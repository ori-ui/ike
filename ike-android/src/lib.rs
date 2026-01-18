#![warn(clippy::unwrap_used)]

use std::{
    any::Any,
    ffi::{self, CString},
    fmt, io,
    pin::Pin,
    ptr::{self, NonNull},
    sync::{
        Arc, OnceLock,
        mpsc::{Receiver, Sender, channel},
    },
    time::Instant,
};

mod callbacks;
mod context;
mod ime;
mod input;
mod log;
mod native;
mod window;

use jni::JavaVM;
pub use log::MakeAndroidWriter;

use ike_core::{Padding, Signal, Size, WindowId, WindowUpdate, World};
use ori::Proxied;
use parking_lot::Mutex;
use raw_window_handle::DisplayHandle;

use crate::{
    context::Proxy,
    ime::{Ime, ImeEvent},
    input::InputQueueEvent,
    window::WindowEvent,
};

static GLOBAL_STATE: OnceLock<ActivityState> = OnceLock::new();

struct ActivityState {
    activity: NonNull<ndk_sys::ANativeActivity>,
    receiver: Receiver<Event>,
    sender:   Sender<Event>,
    waker:    Mutex<Option<Box<dyn Fn() + Send>>>,

    ime: Ime,
}

fn send_event(event: Event) {
    let global_state = GLOBAL_STATE.get().expect("GLOBAL_STATE has not been set");
    let _ = global_state.sender.send(event);

    if let Some(ref wake) = *global_state.waker.lock() {
        wake();
    }
}

unsafe impl Send for ActivityState {}
unsafe impl Sync for ActivityState {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("vulkan error: {0}")]
    Vulkan(#[from] ike_skia::vulkan::Error),

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("jni error: {0}")]
    Jni(#[from] jni::errors::Error),
}

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

    let (sender, receiver) = channel();

    let state = ActivityState {
        activity: native_activity,
        receiver,
        sender,
        waker: Default::default(),

        ime: Ime::new(),
    };

    GLOBAL_STATE
        .set(state)
        .unwrap_or_else(|_| panic!("android main should never called twice"));

    unsafe {
        callbacks::register_callbacks(native_activity.as_mut());

        let jvm = JavaVM::from_raw(native_activity.as_ref().vm).expect("failed getting jvm");
        native::init(&jvm, native_activity).expect("failed to register callbacks");
    }

    std::thread::spawn(move || {
        let exit_code = main().report();

        if exit_code == std::process::ExitCode::SUCCESS {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    });
}

pub fn run<T>(
    data: &mut T,
    mut build: ike_ori::UiBuilder<T>,
    settings: ike_core::Settings,
) -> Result<(), Error> {
    let global_state = GLOBAL_STATE
        .get()
        .expect("android_main should have been called");

    let rt;
    let _rt_guard = if tokio::runtime::Handle::try_current().is_err() {
        rt = Some(tokio::runtime::Runtime::new()?);
        rt.as_ref().map(|rt| rt.enter())
    } else {
        None
    };

    let runtime = tokio::runtime::Handle::current();
    let display_handle = DisplayHandle::android();
    let vulkan_context = ike_skia::vulkan::Context::new(display_handle)?;
    let mut painter = ike_skia::SkiaPainter::new();

    painter.load_font(
        include_bytes!("../../fonts/InterVariable.ttf"),
        None,
    );

    let scale_factor = unsafe {
        let config = ndk_sys::AConfiguration_new();

        ndk_sys::AConfiguration_fromAssetManager(
            config,
            global_state.activity.as_ref().assetManager,
        );

        ndk_sys::AConfiguration_getDensity(config) as f32 / 160.0
    };

    let looper =
        unsafe { ndk_sys::ALooper_prepare(ndk_sys::ALOOPER_PREPARE_ALLOW_NON_CALLBACKS as i32) };

    let proxy = Proxy::new(global_state.sender.clone(), looper);
    let signaller = Box::new({
        let proxy = proxy.clone();

        move |signal| proxy.send(Event::Signal(signal))
    });

    let mut context = ike_ori::Context {
        world:     World::new(signaller, settings),
        proxy:     Arc::new(proxy.clone()),
        resources: ike_ori::Resources::new(),
    };

    *global_state.waker.lock() = Some(Box::new({
        let proxy = proxy.clone();
        move || proxy.wake()
    }));

    let mut view = build(data);
    let (_, state) = view.any_build(&mut context, data);

    let jvm = unsafe { JavaVM::from_raw(global_state.activity.as_ref().vm)? };

    let mut event_loop = EventLoop {
        data,
        build,
        view,
        state,

        runtime,
        context,
        proxy,

        global_state,
        native_activity: global_state.activity,
        looper,
        jvm,

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

    event_loop.run();

    Ok(())
}

struct EventLoop<'a, T> {
    data:  &'a mut T,
    build: ike_ori::UiBuilder<T>,
    view:  ike_ori::AnyEffect<T>,
    state: Box<dyn Any>,

    runtime: tokio::runtime::Handle,
    context: ike_ori::Context,
    proxy:   Proxy,

    global_state:    &'static ActivityState,
    native_activity: NonNull<ndk_sys::ANativeActivity>,
    looper:          *mut ndk_sys::ALooper,
    jvm:             JavaVM,

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
    id:      Option<WindowId>,
    android: *mut ndk_sys::ANativeWindow,
    surface: ike_skia::vulkan::Surface,
    focused: bool,
    width:   u32,
    height:  u32,
    insets:  Padding,
}

enum Event {
    Resumed,

    InputQueue(InputQueueEvent),
    Window(WindowEvent),
    Ime(ImeEvent),
    Signal(Signal),
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
    fn ime(&self) -> &Ime {
        &self.global_state.ime
    }

    fn run(&mut self) {
        unsafe {
            let proxy = Box::new(self.proxy.clone());
            self.native_activity.as_mut().instance = Box::into_raw(proxy).cast();
        }

        unsafe {
            self.choreographer = NonNull::new(ndk_sys::AChoreographer_getInstance());
        }

        loop {
            let global_state = GLOBAL_STATE
                .get()
                .expect("android_main should have been called");

            while let Ok(event) = global_state.receiver.try_recv() {
                self.handle_event(event);
            }

            self.handle_input_events();

            unsafe {
                ndk_sys::ALooper_pollOnce(
                    -1,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
            }
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
                    (),
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
                    (),
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

    fn handle_signal(&mut self, signal: Signal) {
        match signal {
            Signal::RequestRedraw { .. } => {
                self.proxy.send(Event::Window(WindowEvent::Redraw));
            }

            Signal::RequestAnimate { start, .. } => {
                if self.animate.is_none() {
                    self.animate = Some(start);
                    self.proxy.send(Event::Window(WindowEvent::Redraw));
                }
            }

            Signal::Mutate(f) => {
                f(&mut self.context.world);
            }

            Signal::ClipboardSet(..) => {}

            Signal::CreateWindow(window_id) => match self.window {
                WindowState::Pending { ref mut id, .. } if id.is_none() => {
                    *id = Some(window_id);
                }

                WindowState::Open(ref mut window) if window.id.is_none() => {
                    let size = Size::new(
                        window.width as f32 / self.scale_factor,
                        window.height as f32 / self.scale_factor,
                    );

                    window.id = Some(window_id);
                    (self.context.world).window_scaled(window_id, size, self.scale_factor);
                    (self.context.world).window_focused(window_id, window.focused);
                    (self.context.world).window_inset(window_id, window.insets);
                }

                _ => {
                    tracing::error!("android only supportes one window!");
                }
            },

            Signal::RemoveWindow(..) => {}

            Signal::UpdateWindow(_, update) => {
                if let WindowState::Open(ref mut window) = self.window {
                    window.handle_update(update);
                }
            }

            Signal::Ime(ime) => self.handle_ime_signal(ime),
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
