use std::{
    any::Any,
    ffi::{self, CString},
    pin::Pin,
    ptr::{self, NonNull},
    sync::{OnceLock, mpsc::Receiver},
    time::Instant,
};

mod callbacks;
mod context;
mod log;

pub use context::Context;
use jni::{
    JNIEnv, JavaVM,
    objects::{JObject, JValue},
};
pub(crate) use log::MakeAndroidWriter;

use ike_core::{
    ImeSignal, Point, PointerButton, PointerId, RootSignal, Size, WindowId, WindowUpdate,
};
use ori::AsyncContext;
use raw_window_handle::{AndroidNdkWindowHandle, DisplayHandle, RawWindowHandle, WindowHandle};

use crate::{
    android::context::Proxy,
    app::{AnyEffect, UiBuilder},
};

static NATIVE_ACTIVITY: OnceLock<NativeActivity> = OnceLock::new();

#[derive(Clone, Copy)]
struct NativeActivity(NonNull<ndk_sys::ANativeActivity>);

unsafe impl Send for NativeActivity {}
unsafe impl Sync for NativeActivity {}

#[doc(hidden)]
pub fn android_main(native_activity: *mut ffi::c_void, main: fn()) {
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
    }));

    tracing::debug!("app starting");

    let mut native_activity = NonNull::new(native_activity.cast())
        .expect("native_activity should never be null, but was null");

    unsafe {
        callbacks::register_callbacks(native_activity.as_mut());
    }

    let _ = NATIVE_ACTIVITY.set(NativeActivity(native_activity.cast()));

    std::thread::spawn(main);
}

pub fn run<T>(data: &mut T, build: UiBuilder<T>) {
    let native_activity = *NATIVE_ACTIVITY
        .get()
        .expect("native activity should be set");

    let mut event_loop = EventLoop::new(native_activity.0, data, build);
    event_loop.run();
}

struct EventLoop<'a, T> {
    data:  &'a mut T,
    build: UiBuilder<T>,
    view:  AnyEffect<T>,
    state: Box<dyn Any>,

    runtime: tokio::runtime::Handle,
    context: Context,

    native_activity: NonNull<ndk_sys::ANativeActivity>,
    looper:          *mut ndk_sys::ALooper,
    receiver:        Receiver<Event>,
    jvm:             JavaVM,

    choreographer: Option<NonNull<ndk_sys::AChoreographer>>,
    is_rendering:  bool,
    wants_render:  bool,
    vulkan:        ike_skia::vulkan::SkiaVulkanContext,
    painter:       ike_skia::SkiaPainter,

    scale_factor: f32,

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
    surface: ike_skia::vulkan::SkiaVulkanSurface,
}

enum Event {
    WindowCreated(*mut ndk_sys::ANativeWindow),
    WindowDestroyed,
    WindowRedraw,
    WindowResized,
    WindowFocusChanged(bool),

    RenderFinished,

    InputQueueCreated(*mut ndk_sys::AInputQueue),
    InputQueueDestroyed(*mut ndk_sys::AInputQueue),

    Signal(RootSignal),
    Rebuild,
    Event(ori::Event),
    Future(Pin<Box<dyn Future<Output = ()> + Send>>),
}

unsafe impl Send for Event {}

impl<'a, T> EventLoop<'a, T> {
    fn new(
        native_activity: NonNull<ndk_sys::ANativeActivity>,
        data: &'a mut T,
        mut build: UiBuilder<T>,
    ) -> Self {
        let rt;
        let _rt_guard = if tokio::runtime::Handle::try_current().is_err() {
            rt = Some(tokio::runtime::Runtime::new().unwrap());
            Some(rt.as_ref().unwrap().enter())
        } else {
            None
        };

        let runtime = tokio::runtime::Handle::current();
        let display_handle = DisplayHandle::android();
        let vulkan_context = ike_skia::vulkan::SkiaVulkanContext::new(display_handle);
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

        let looper = unsafe {
            ndk_sys::ALooper_prepare(ndk_sys::ALOOPER_PREPARE_ALLOW_NON_CALLBACKS as i32)
        };

        let proxy = Proxy::new(sender, looper);
        let mut context = Context::new(proxy);

        let mut view = build(data);
        let (_, state) = view.any_build(&mut context, data);

        let jvm = unsafe { JavaVM::from_raw(native_activity.as_ref().vm).unwrap() };

        Self {
            data,
            build,
            view,
            state,

            runtime,
            context,

            native_activity,
            looper,
            receiver,
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
        }
    }

    fn run(&mut self) {
        unsafe {
            let proxy = Box::new(self.context.proxy.clone());
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

            if let Some(queue) = self.input_queue {
                unsafe {
                    let mut event = ptr::null_mut();
                    while ndk_sys::AInputQueue_getEvent(queue, &mut event) >= 0 {
                        let handled = self.handle_input_event(event);
                        ndk_sys::AInputQueue_finishEvent(queue, event, handled as i32);
                    }
                }
            }
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::WindowCreated(android) => {
                let window_handle = unsafe {
                    let window = ptr::NonNull::new(android).unwrap().cast();
                    WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(
                        AndroidNdkWindowHandle::new(window),
                    ))
                };

                let width = unsafe { ndk_sys::ANativeWindow_getWidth(android) };
                let height = unsafe { ndk_sys::ANativeWindow_getHeight(android) };

                tracing::debug!(width, height, "window created");

                let vulkan = unsafe {
                    ike_skia::vulkan::SkiaVulkanSurface::new(
                        &mut self.vulkan,
                        DisplayHandle::android(),
                        window_handle,
                        width as u32,
                        height as u32,
                    )
                };

                if let WindowState::Pending {
                    id: Some(id),
                    ref mut updates,
                } = self.window
                {
                    let mut window = Window {
                        id,
                        android,
                        surface: vulkan,
                    };

                    for update in updates.drain(..) {
                        window.handle_update(update);
                    }

                    self.window = WindowState::Open(window);
                } else {
                    tracing::error!("android must have at least one window!");
                }
            }

            Event::WindowDestroyed => {
                match self.window {
                    WindowState::Open(ref window) => {
                        self.window = WindowState::Pending {
                            id:      Some(window.id),
                            updates: Vec::new(),
                        };
                    }

                    WindowState::Pending { .. } => {}
                };
            }

            Event::WindowRedraw => {
                if let WindowState::Open(ref mut window) = self.window {
                    if self.is_rendering {
                        self.wants_render = true;
                        return;
                    }

                    if let Some(animate) = self.animate.take() {
                        let delta_time = animate.elapsed();
                        self.context.root.animate(window.id, delta_time);
                    }

                    let color = self.context.root.get_window(window.id).unwrap().color();

                    if let Some(choreographer) = self.choreographer {
                        unsafe {
                            ndk_sys::AChoreographer_postFrameCallback(
                                choreographer.as_ptr(),
                                Some(frame_callback),
                                (&mut self.context.proxy) as *mut _ as *mut _,
                            );
                        };
                    }

                    self.is_rendering = true;

                    window.surface.draw(
                        &mut self.painter,
                        color,
                        self.scale_factor,
                        || {},
                        |canvas| self.context.root.draw(window.id, canvas),
                    );
                }
            }

            Event::WindowResized => {
                if let WindowState::Open(ref mut window) = self.window {
                    let width = unsafe { ndk_sys::ANativeWindow_getWidth(window.android) };
                    let height = unsafe { ndk_sys::ANativeWindow_getHeight(window.android) };

                    window.surface.resize(width as u32, height as u32);
                    let size = Size::new(
                        width as f32 / self.scale_factor,
                        height as f32 / self.scale_factor,
                    );

                    (self.context.root).window_scaled(window.id, self.scale_factor, size);
                }
            }

            Event::WindowFocusChanged(focused) => {
                if let WindowState::Open(ref mut window) = self.window {
                    (self.context.root).window_focused(window.id, focused);
                }
            }

            Event::RenderFinished => {
                self.is_rendering = false;

                if self.wants_render {
                    self.wants_render = false;
                    self.context.proxy.send(Event::WindowRedraw);
                }
            }

            Event::InputQueueCreated(queue) => {
                tracing::trace!(
                    looper = ?self.looper,
                    queue = ?queue,
                    "attaching looper to input queue",
                );

                unsafe {
                    ndk_sys::AInputQueue_attachLooper(
                        queue,
                        self.looper,
                        42069,
                        None,
                        ptr::null_mut(),
                    );
                };

                self.input_queue = Some(queue);
            }

            Event::InputQueueDestroyed(queue) => {
                tracing::trace!(
                    queue = ?queue,
                    "input queue destroyed",
                );

                self.input_queue = None;
            }

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

    fn handle_input_event(&mut self, event: *mut ndk_sys::AInputEvent) -> bool {
        let WindowState::Open(ref window) = self.window else {
            return false;
        };

        let kind = unsafe { ndk_sys::AInputEvent_getType(event) };

        match kind as u32 {
            ndk_sys::AINPUT_EVENT_TYPE_MOTION => self.handle_motion_event(window.id, event),
            _ => false,
        }
    }

    fn handle_motion_event(
        &mut self,
        window_id: WindowId,
        event: *mut ndk_sys::AInputEvent,
    ) -> bool {
        let action = unsafe { ndk_sys::AMotionEvent_getAction(event) as u32 };
        let index = ((action & ndk_sys::AMOTION_EVENT_ACTION_POINTER_INDEX_MASK)
            >> ndk_sys::AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT) as usize;
        let action = action & ndk_sys::AMOTION_EVENT_ACTION_MASK;

        match action {
            ndk_sys::AMOTION_EVENT_ACTION_DOWN | ndk_sys::AMOTION_EVENT_ACTION_POINTER_DOWN => {
                let x = unsafe { ndk_sys::AMotionEvent_getX(event, index) };
                let y = unsafe { ndk_sys::AMotionEvent_getY(event, index) };
                let point = Point::new(
                    x / self.scale_factor,
                    y / self.scale_factor,
                );

                let id = unsafe { ndk_sys::AMotionEvent_getPointerId(event, index) };
                let id = PointerId::from_u64(id as u64);

                tracing::trace!(
                    index,
                    point = ?point,
                    "down event",
                );

                let entered = self.context.root.pointer_entered(window_id, id);
                let moved = self.context.root.pointer_moved(window_id, id, point);
                let pressed = self.context.root.pointer_pressed(
                    window_id,
                    id,
                    PointerButton::Primary,
                    true,
                );

                entered || moved || pressed
            }

            ndk_sys::AMOTION_EVENT_ACTION_MOVE => {
                let count = unsafe { ndk_sys::AMotionEvent_getPointerCount(event) };
                let mut handled = false;

                for i in 0..count {
                    let x = unsafe { ndk_sys::AMotionEvent_getX(event, i) };
                    let y = unsafe { ndk_sys::AMotionEvent_getY(event, i) };
                    let point = Point::new(
                        x / self.scale_factor,
                        y / self.scale_factor,
                    );

                    let id = unsafe { ndk_sys::AMotionEvent_getPointerId(event, i) };
                    let id = PointerId::from_u64(id as u64);

                    tracing::trace!(
                        index,
                        point = ?point,
                        "move event",
                    );

                    handled |= self.context.root.pointer_moved(window_id, id, point);
                }

                handled
            }

            ndk_sys::AMOTION_EVENT_ACTION_UP | ndk_sys::AMOTION_EVENT_ACTION_POINTER_UP => {
                let id = unsafe { ndk_sys::AMotionEvent_getPointerId(event, index) };
                let id = PointerId::from_u64(id as u64);

                tracing::trace!(index, "up event");

                let pressed = self.context.root.pointer_pressed(
                    window_id,
                    id,
                    PointerButton::Primary,
                    false,
                );
                let left = self.context.root.pointer_left(window_id, id);

                pressed || left
            }

            _ => false,
        }
    }

    fn handle_signal(&mut self, signal: RootSignal) {
        match signal {
            RootSignal::RequestRedraw(..) => {
                self.context.proxy.send(Event::WindowRedraw);
            }

            RootSignal::RequestAnimate(..) => {
                self.animate = Some(Instant::now());
                self.context.proxy.send(Event::WindowRedraw);
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

            RootSignal::Ime(ime) => match ime {
                ImeSignal::Start => {
                    if let Ok(mut env) = self.jvm.attach_current_thread() {
                        let _ = self.show_soft_input(&mut env);
                        tracing::debug!("show soft input");
                    }
                }

                ImeSignal::End => {
                    tracing::info!("end ime");
                }

                ImeSignal::Moved(_) => {}
            },
        }
    }

    fn show_soft_input(&self, env: &mut JNIEnv<'_>) -> jni::errors::Result<bool> {
        let view = self.rust_view(env)?;
        let imm = self.input_method_manager(env, &view)?;

        env.call_method(
            &imm,
            "showSoftInput",
            "(Landroid/view/View;I)Z",
            &[(&view).into(), 0.into()],
        )?
        .z()
    }

    fn input_method_manager<'local>(
        &self,
        env: &mut JNIEnv<'local>,
        rust_view: &JObject<'local>,
    ) -> jni::errors::Result<JObject<'local>> {
        env.get_field(
            rust_view,
            "inputMethodManager",
            "Landroid/view/inputmethod/InputMethodManager;",
        )?
        .l()
    }

    fn rust_view<'local>(&self, env: &mut JNIEnv<'local>) -> jni::errors::Result<JObject<'local>> {
        let activity = unsafe { JObject::from_raw(self.native_activity.as_ref().clazz) };

        env.get_field(
            activity,
            "rustView",
            "Lorg/ori/RustView;",
        )?
        .l()
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

unsafe extern "C" fn frame_callback(_frame_time_nanos: i64, data: *mut ffi::c_void) {
    let proxy = unsafe { &*data.cast::<Proxy>() };
    proxy.send(Event::RenderFinished);
}
