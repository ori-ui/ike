use std::{
    any::Any,
    ffi::{self, CString},
    fmt,
    pin::Pin,
    ptr::{self, NonNull},
    sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
        mpsc::Receiver,
    },
    time::Instant,
};

mod callbacks;
mod context;
mod ime;
mod input;
mod log;
mod window;

use jni::JavaVM;
pub use log::MakeAndroidWriter;

use ike_core::{Root, RootSignal, WindowId, WindowUpdate};
use ori::Proxyable;
use raw_window_handle::DisplayHandle;

use crate::{
    context::Proxy,
    ime::{Ime, ImeEvent},
    input::InputQueueEvent,
    window::WindowEvent,
};

static NATIVE_ACTIVITY: AtomicPtr<ndk_sys::ANativeActivity> = AtomicPtr::new(ptr::null_mut());

#[derive(thiserror::Error, Debug)]
pub enum Error {}

#[doc(hidden)]
pub fn android_main<T>(native_activity: *mut ffi::c_void, main: fn() -> T)
where
    T: std::process::Termination + 'static,
{
    std::panic::set_hook(Box::new(|info| {
        if let Ok(message) = CString::new(info.to_string()) {
            unsafe {
                ndk_sys::__android_log_print(
                    ndk_sys::android_LogPriority::ANDROID_LOG_FATAL.0 as i32,
                    c"rust".as_ptr(),
                    message.as_ptr(),
                );
            }
        }

        std::process::abort();
    }));

    tracing::debug!("app starting");

    let mut native_activity = NonNull::new(native_activity.cast())
        .expect("native_activity should never be null, but was null");

    unsafe {
        callbacks::register_callbacks(native_activity.as_mut());
    }

    NATIVE_ACTIVITY.store(
        native_activity.as_ptr(),
        Ordering::SeqCst,
    );

    std::thread::spawn(move || {
        let exit_code = main().report();

        if exit_code == std::process::ExitCode::SUCCESS {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    });
}

pub fn run<T>(data: &mut T, mut build: ike_ori::UiBuilder<T>) {
    let native_activity = NonNull::new(NATIVE_ACTIVITY.load(Ordering::SeqCst))
        .expect("native activity should be set");

    let rt;
    let _rt_guard = if tokio::runtime::Handle::try_current().is_err() {
        rt = Some(tokio::runtime::Runtime::new().unwrap());
        Some(rt.as_ref().unwrap().enter())
    } else {
        None
    };

    let runtime = tokio::runtime::Handle::current();
    let display_handle = DisplayHandle::android();
    let vulkan_context = ike_skia::vulkan::Context::new(display_handle).unwrap();
    let painter = ike_skia::SkiaPainter::new();

    let scale_factor = unsafe {
        let config = ndk_sys::AConfiguration_new();

        ndk_sys::AConfiguration_fromAssetManager(
            config,
            native_activity.as_ref().assetManager,
        );

        ndk_sys::AConfiguration_getDensity(config) as f32 / 160.0
    };

    let (sender, receiver) = std::sync::mpsc::channel::<Event>();

    let looper =
        unsafe { ndk_sys::ALooper_prepare(ndk_sys::ALOOPER_PREPARE_ALLOW_NON_CALLBACKS as i32) };

    let proxy = Proxy::new(sender, looper);

    let root = Root::new({
        let proxy = proxy.clone();

        move |signal| proxy.send(Event::Signal(signal))
    });
    let mut context = ike_ori::Context::new(root, Arc::new(proxy.clone()));

    let mut view = build(data);
    let (_, state) = view.any_build(&mut context, data);

    let (jvm, ime) = unsafe {
        let jvm = JavaVM::from_raw(native_activity.as_ref().vm).unwrap();
        let ime = ime::init(&jvm, native_activity, proxy.clone()).unwrap();

        (jvm, ime)
    };

    let mut event_loop = EventLoop {
        data,
        build,
        view,
        state,

        runtime,
        context,
        proxy,

        native_activity,
        looper,
        receiver,
        jvm,
        ime,

        choreographer: None,
        is_rendering: false,
        wants_render: false,
        vulkan: vulkan_context,
        painter,

        scale_factor,

        input_queue: None,

        animate: None,
        window: WindowState::Pending {
            id:      None,
            updates: Vec::new(),
        },
    };

    event_loop.run()
}

struct EventLoop<'a, T> {
    data:  &'a mut T,
    build: ike_ori::UiBuilder<T>,
    view:  ike_ori::AnyEffect<T>,
    state: Box<dyn Any>,

    runtime: tokio::runtime::Handle,
    context: ike_ori::Context,
    proxy:   Proxy,

    native_activity: NonNull<ndk_sys::ANativeActivity>,
    looper:          *mut ndk_sys::ALooper,
    receiver:        Receiver<Event>,
    jvm:             JavaVM,
    ime:             &'static Ime,

    choreographer: Option<NonNull<ndk_sys::AChoreographer>>,
    is_rendering:  bool,
    wants_render:  bool,
    vulkan:        ike_skia::vulkan::Context,
    painter:       ike_skia::SkiaPainter,
    scale_factor:  f32,

    input_queue: Option<*mut ndk_sys::AInputQueue>,

    animate: Option<Instant>,
    window:  WindowState,
}

#[allow(clippy::large_enum_variant)]
enum WindowState {
    Open(Window),
    Pending {
        id:      Option<WindowId>,
        updates: Vec<WindowUpdate>,
    },
}

struct Window {
    id:      WindowId,
    android: *mut ndk_sys::ANativeWindow,
    surface: ike_skia::vulkan::Surface,
}

enum Event {
    Resumed,

    InputQueue(InputQueueEvent),
    Window(WindowEvent),
    Ime(ImeEvent),
    Signal(RootSignal),
    Rebuild,
    Event(ori::Event),
    Future(Pin<Box<dyn Future<Output = ()> + Send>>),
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resumed => write!(f, "Resumed"),
            Self::InputQueue(arg0) => f.debug_tuple("InputQueue").field(arg0).finish(),
            Self::Window(arg0) => f.debug_tuple("Window").field(arg0).finish(),
            Self::Ime(arg0) => f.debug_tuple("Ime").field(arg0).finish(),
            Self::Signal(arg0) => f.debug_tuple("Signal").field(arg0).finish(),
            Self::Rebuild => write!(f, "Rebuild"),
            Self::Event(arg0) => f.debug_tuple("Event").field(arg0).finish(),
            Self::Future(_) => f.debug_tuple("Future").finish(),
        }
    }
}

impl<'a, T> EventLoop<'a, T> {
    fn run(&mut self) {
        unsafe {
            let proxy = Box::new(self.proxy.clone());
            self.native_activity.as_mut().instance = Box::into_raw(proxy).cast();
        }

        unsafe {
            self.choreographer = NonNull::new(ndk_sys::AChoreographer_getInstance());
        }

        loop {
            unsafe {
                ndk_sys::ALooper_pollOnce(
                    -1,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
            }

            while let Ok(event) = self.receiver.try_recv() {
                self.handle_event(event);
            }

            self.handle_input_events();
        }
    }

    fn handle_event(&mut self, event: Event) {
        tracing::trace!(?event, "android event");

        match event {
            Event::Resumed => {}

            Event::InputQueue(event) => self.handle_input_queue_event(event),
            Event::Window(event) => self.handle_window_event(event),
            Event::Ime(event) => self.handle_ime_event(event),
            Event::Signal(signal) => self.handle_signal(signal),

            Event::Rebuild => {
                let mut view = (self.build)(self.data);
                ori::View::rebuild(
                    &mut view,
                    &mut ori::NoElement,
                    &mut self.state,
                    &mut self.context,
                    self.data,
                    &mut self.view,
                );
                self.view = view;
            }

            Event::Event(mut event) => {
                let action = ori::View::event(
                    &mut self.view,
                    &mut ori::NoElement,
                    &mut self.state,
                    &mut self.context,
                    self.data,
                    &mut event,
                );

                self.context.send_action(action);
            }

            Event::Future(future) => {
                self.runtime.spawn(future);
            }
        }
    }

    fn handle_signal(&mut self, signal: RootSignal) {
        match signal {
            RootSignal::RequestRedraw(..) => {
                self.proxy.send(Event::Window(WindowEvent::Redraw));
            }

            RootSignal::RequestAnimate(_, last_frame) => {
                if self.animate.is_none() {
                    self.animate = Some(last_frame);
                    self.proxy.send(Event::Window(WindowEvent::Redraw));
                }
            }

            RootSignal::ClipboardSet(..) => {}

            RootSignal::CreateWindow(window_id) => match self.window {
                WindowState::Pending { ref mut id, .. } if id.is_none() => {
                    *id = Some(window_id);
                }

                _ => {
                    tracing::error!("android only supportes one window!");
                }
            },

            RootSignal::RemoveWindow(..) => {}

            RootSignal::UpdateWindow(_, update) => {
                if let WindowState::Open(ref mut window) = self.window {
                    window.handle_update(update);
                }
            }

            RootSignal::Ime(ime) => self.handle_ime_signal(ime),
        }
    }
}

impl Window {
    fn handle_update(&mut self, update: WindowUpdate) {
        match update {
            WindowUpdate::Title(..) => {}
            WindowUpdate::Sizing(..) => {}
            WindowUpdate::Visible(..) => {}
            WindowUpdate::Decorated(..) => {}
            WindowUpdate::Cursor(..) => {}
        }
    }
}
