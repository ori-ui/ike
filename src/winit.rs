use std::{
    any::{Any, TypeId},
    pin::Pin,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::Instant,
};

use ike_core::{
    AnyWidgetId, BuildCx, Modifiers, Offset, Point, PointerButton, PointerId, Root, RootSignal,
    ScrollDelta, Size, WidgetId, WindowSizing, WindowUpdate,
};
use ike_skia::{
    SkiaPainter,
    vulkan::{SkiaVulkanContext, SkiaVulkanWindow},
};
use ori::AsyncContext;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use crate::{
    app::{AnyEffect, UiBuilder},
    key::convert_winit_key,
};

pub(crate) fn run<T>(data: &mut T, mut build: UiBuilder<T>) {
    let rt;
    let _rt_guard = if tokio::runtime::Handle::try_current().is_err() {
        rt = Some(tokio::runtime::Runtime::new().unwrap());
        Some(rt.as_ref().unwrap().enter())
    } else {
        None
    };

    let runtime = tokio::runtime::Handle::current();
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let vulkan = SkiaVulkanContext::new(&event_loop);

    let mut painter = SkiaPainter::new();
    painter.load_font(
        include_bytes!("InterVariable.ttf"),
        None,
    );

    let (sender, receiver) = std::sync::mpsc::channel();
    let context = Context {
        root: ike_core::Root::new({
            let sender = sender.clone();

            move |signal| {
                let _ = sender.send(Event::Signal(signal));
            }
        }),
        proxy: event_loop.create_proxy(),
        entries: Vec::new(),
        sender,

        use_type_names_unsafe: false,
    };

    let view = build(data);

    #[cfg(target_os = "linux")]
    let clipboard = {
        use copypasta::{
            wayland_clipboard::create_clipboards_from_external,
            x11_clipboard::{Clipboard, X11ClipboardContext},
        };
        use winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};

        let display_handle = event_loop.display_handle().unwrap();
        if let RawDisplayHandle::Wayland(handle) = display_handle.as_raw() {
            unsafe {
                let (_primary, clipboard) =
                    create_clipboards_from_external(handle.display.as_ptr());
                Box::new(clipboard) as Box<_>
            }
        } else {
            Box::new(X11ClipboardContext::<Clipboard>::new().unwrap()) as Box<_>
        }
    };

    #[cfg(not(target_os = "linux"))]
    let clipboard = Box::new(copypasta::ClipboardContext::new().unwrap());

    let mut state = AppState {
        data,

        build,
        view,
        state: None,

        runtime,
        receiver,
        clipboard,

        painter,
        context,
        windows: Vec::new(),

        vulkan,
    };

    event_loop.run_app(&mut state).unwrap();
}

struct AppState<'a, T> {
    data: &'a mut T,

    build: UiBuilder<T>,
    view:  AnyEffect<T>,
    state: Option<Box<dyn Any>>,

    runtime:  tokio::runtime::Handle,
    receiver: Receiver<Event>,

    clipboard: Box<dyn copypasta::ClipboardProvider>,
    painter:   SkiaPainter,
    context:   Context,
    windows:   Vec<WindowState>,

    vulkan: SkiaVulkanContext,
}

struct WindowState {
    animate: Option<Instant>,

    vulkan: SkiaVulkanWindow,

    id:     ike_core::WindowId,
    window: Window,
}

impl<T> ApplicationHandler for AppState<'_, T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let (_, state) = ori::View::build(
            &mut self.view,
            &mut self.context,
            self.data,
        );

        self.state = Some(state);
        self.handle_events(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.windows.iter_mut().find(|w| w.window.id() == window_id) else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                if let Some(animate) = window.animate.take() {
                    let delta_time = animate.elapsed();
                    self.context.root.animate(window.id, delta_time);
                }

                let desc = self.context.root.get_window(window.id).unwrap();

                let new_window_size = window.vulkan.draw(
                    &mut self.painter,
                    desc.color(),
                    window.window.scale_factor() as f32,
                    || window.window.pre_present_notify(),
                    |canvas| self.context.root.draw(window.id, canvas),
                );

                if let Some(size) = new_window_size.flatten() {
                    let size = LogicalSize::new(size.width, size.height);

                    window.window.set_min_inner_size(Some(size));
                    window.window.set_max_inner_size(Some(size));
                }
            }

            WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
                window.resize();

                let scale = window.window.scale_factor();
                let size = window.window.inner_size().to_logical(scale);

                let size = Size::new(size.width, size.height);

                match event {
                    WindowEvent::Resized(..) => {
                        self.context.root.window_resized(window.id, size);
                    }

                    WindowEvent::ScaleFactorChanged { .. } => {
                        (self.context.root).window_scaled(window.id, scale as f32, size);
                    }

                    _ => unreachable!(),
                }
            }

            WindowEvent::Focused(is_focused) => {
                self.context.root.window_focused(window.id, is_focused);
            }

            WindowEvent::CursorEntered { device_id } => {
                let id = PointerId::from_hash(device_id);
                self.context.root.pointer_entered(window.id, id);
            }

            WindowEvent::CursorLeft { device_id } => {
                let id = PointerId::from_hash(device_id);
                self.context.root.pointer_left(window.id, id);
            }

            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let position = position.to_logical(window.window.scale_factor());
                let position = Point::new(position.x, position.y);

                self.context.root.pointer_moved(
                    window.id,
                    PointerId::from_hash(device_id),
                    position,
                );
            }

            WindowEvent::MouseWheel {
                device_id, delta, ..
            } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => ScrollDelta::Line(Offset::new(x, y)),
                    MouseScrollDelta::PixelDelta(delta) => ScrollDelta::Pixel(
                        Offset::new(delta.x as f32, delta.y as f32)
                            / window.window.scale_factor() as f32,
                    ),
                };

                self.context.root.pointer_scrolled(
                    window.id,
                    delta,
                    PointerId::from_hash(device_id),
                );
            }

            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let button = match button {
                    MouseButton::Left => PointerButton::Primary,
                    MouseButton::Right => PointerButton::Secondary,
                    MouseButton::Middle => PointerButton::Tertiary,
                    MouseButton::Back => PointerButton::Backward,
                    MouseButton::Forward => PointerButton::Forward,
                    MouseButton::Other(i) => PointerButton::Other(i),
                };

                let pressed = matches!(state, ElementState::Pressed);

                self.context.root.pointer_pressed(
                    window.id,
                    PointerId::from_hash(device_id),
                    button,
                    pressed,
                );
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let action_mod = match self.context.root.get_window(window.id) {
                    Some(window) if cfg!(target_os = "macos") => window.modifiers().meta(),
                    Some(window) => window.modifiers().ctrl(),
                    None => false,
                };

                if matches!(
                    event.logical_key,
                    winit::keyboard::Key::Character(ref c) if c == "v",
                ) && event.state.is_pressed()
                    && action_mod
                    && cfg!(any(
                        target_os = "linux",
                        target_os = "macos",
                        target_os = "windows"
                    ))
                {
                    if let Ok(text) = self.clipboard.get_contents() {
                        self.context.root.text_pasted(window.id, text);
                    }
                } else {
                    self.context.root.key_pressed(
                        window.id,
                        convert_winit_key(event.logical_key),
                        event.repeat,
                        event.text.as_deref(),
                        event.state.is_pressed(),
                    );
                }
            }

            WindowEvent::ModifiersChanged(mods) => {
                let mut modifiers = Modifiers::empty();

                if mods.state().shift_key() {
                    modifiers |= Modifiers::SHIFT;
                }

                if mods.state().control_key() {
                    modifiers |= Modifiers::CONTROL;
                }

                if mods.state().alt_key() {
                    modifiers |= Modifiers::ALT;
                }

                if mods.state().super_key() {
                    modifiers |= Modifiers::META;
                }

                self.context.root.modifiers_changed(window.id, modifiers);
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => {}
        }

        self.handle_events(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: ()) {
        self.handle_events(event_loop);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.painter.cleanup();
    }
}

impl<T> AppState<'_, T> {
    fn handle_events(&mut self, event_loop: &ActiveEventLoop) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                Event::Rebuild => {
                    let mut view = (self.build)(self.data);
                    ori::View::rebuild(
                        &mut view,
                        &mut ori::NoElement,
                        self.state.as_mut().unwrap(),
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
                        self.state.as_mut().unwrap(),
                        &mut self.context,
                        self.data,
                        &mut event,
                    );

                    self.context.send_action(action);
                }

                Event::Spawn(future) => {
                    self.runtime.spawn(future);
                }

                Event::Signal(signal) => self.handle_signal(event_loop, signal),
            }
        }
    }

    fn handle_signal(&mut self, event_loop: &ActiveEventLoop, signal: RootSignal) {
        match signal {
            RootSignal::RequestRedraw(id) => {
                if let Some(window) = self.windows.iter().find(|w| w.id == id) {
                    window.window.request_redraw();
                }
            }

            RootSignal::RequestAnimate(id) => {
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.animate = Some(Instant::now());
                    window.window.request_redraw();
                }
            }

            RootSignal::ClipboardSet(text) => {
                let _ = self.clipboard.set_contents(text);
            }

            RootSignal::CreateWindow(id) => {
                if let Some(window) = self.context.root.get_window(id) {
                    let window = WindowState::new(&mut self.vulkan, event_loop, window);
                    self.windows.push(window);
                }
            }

            RootSignal::RemoveWindow(id) => {
                self.windows.retain(|w| w.id != id);
            }

            RootSignal::UpdateWindow(id, update) => {
                let Some(win) = self.windows.iter_mut().find(|w| w.id == id) else {
                    return;
                };

                match update {
                    WindowUpdate::Title(title) => win.window.set_title(&title),

                    WindowUpdate::Sizing(sizing) => {
                        if let WindowSizing::Resizable {
                            min_size, max_size, ..
                        } = sizing
                        {
                            win.window.set_min_inner_size(Some(LogicalSize::new(
                                min_size.width,
                                min_size.height,
                            )));

                            win.window.set_max_inner_size(Some(LogicalSize::new(
                                max_size.width,
                                max_size.height,
                            )));
                        }
                    }

                    WindowUpdate::Visible(visible) => {
                        win.window.set_visible(visible);
                    }

                    WindowUpdate::Decorated(decorated) => {
                        win.window.set_decorations(decorated);
                    }

                    WindowUpdate::Cursor(cursor) => {
                        win.window.set_cursor(cursor);
                    }
                }
            }
        }
    }
}

impl WindowState {
    fn new(
        vulkan: &mut SkiaVulkanContext,
        event_loop: &ActiveEventLoop,
        desc: &ike_core::Window,
    ) -> Self {
        use winit::dpi::LogicalSize;

        let size = match desc.sizing() {
            WindowSizing::FitContent => LogicalSize::new(
                desc.current_size().width,
                desc.current_size().height,
            ),
            WindowSizing::Resizable { default_size, .. } => {
                LogicalSize::new(default_size.width, default_size.height)
            }
        };

        let min_size = match desc.sizing() {
            WindowSizing::FitContent => size,
            WindowSizing::Resizable { min_size, .. } => LogicalSize::new(
                min_size.width.max(1.0),
                min_size.height.max(1.0),
            ),
        };

        let max_size = match desc.sizing() {
            WindowSizing::FitContent => size,
            WindowSizing::Resizable { max_size, .. } => LogicalSize::new(
                max_size.width.max(1.0),
                max_size.height.max(1.0),
            ),
        };

        let attributes = Window::default_attributes()
            .with_title(desc.title())
            .with_visible(desc.is_visible())
            .with_decorations(desc.is_decorated())
            .with_transparent(true)
            .with_min_inner_size(min_size)
            .with_max_inner_size(max_size)
            .with_inner_size(size)
            .with_cursor(desc.cursor())
            .with_resizable(matches!(
                desc.sizing(),
                WindowSizing::Resizable { .. }
            ));

        let window = event_loop.create_window(attributes).unwrap();

        let vulkan = unsafe {
            let physical = window.inner_size();
            SkiaVulkanWindow::new(
                vulkan,
                &window,
                physical.width,
                physical.height,
            )
        };

        Self {
            animate: None,
            id: desc.id(),
            vulkan,
            window,
        }
    }

    fn resize(&mut self) {
        let size = self.window.inner_size();

        (self.vulkan).resize(size.width, size.height);
    }
}

pub struct Context {
    root:    Root,
    proxy:   EventLoopProxy<()>,
    entries: Vec<Entry>,
    sender:  Sender<Event>,

    use_type_names_unsafe: bool,
}

impl Context {
    pub fn create_window(&mut self, contents: impl AnyWidgetId) -> ike_core::WindowId {
        self.root.create_window(contents.upcast())
    }

    pub fn remove_window(&mut self, window: ike_core::WindowId) {
        self.root.remove_window(window)
    }

    #[allow(dead_code)]
    pub(crate) fn is_using_type_names(&self) -> bool {
        self.use_type_names_unsafe
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn use_type_names(&mut self, enabled: bool) {
        self.use_type_names_unsafe = enabled;
    }
}

struct Entry {
    value:     Box<dyn Any>,
    type_id:   TypeId,
    type_name: &'static str,
}

#[derive(Clone)]
pub struct Proxy {
    sender: Sender<Event>,
    proxy:  EventLoopProxy<()>,
}

enum Event {
    Rebuild,
    Event(ori::Event),
    Spawn(Pin<Box<dyn Future<Output = ()> + Send>>),
    Signal(RootSignal),
}

impl BuildCx for Context {
    fn root(&self) -> &Root {
        &self.root
    }

    fn root_mut(&mut self) -> &mut Root {
        &mut self.root
    }
}

impl ori::BaseElement for Context {
    type Element = WidgetId;
}

impl ori::AsyncContext for Context {
    type Proxy = Proxy;

    fn proxy(&mut self) -> Self::Proxy {
        Proxy {
            sender: self.sender.clone(),
            proxy:  self.proxy.clone(),
        }
    }
}

impl ori::ProviderContext for Context {
    fn push_context<T: Any>(&mut self, context: Box<T>) {
        self.entries.push(Entry {
            value:     context,
            type_id:   TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        })
    }

    fn pop_context<T: Any>(&mut self) -> Option<Box<T>> {
        self.entries.pop()?.value.downcast().ok()
    }

    fn get_context<T: Any>(&self) -> Option<&T> {
        let entry = match self.use_type_names_unsafe {
            true => self
                .entries
                .iter()
                .rfind(|e| e.type_name == std::any::type_name::<T>())?,
            false => self
                .entries
                .iter()
                .rfind(|e| e.type_id == TypeId::of::<T>())?,
        };

        Some(unsafe { &*(entry.value.as_ref() as *const _ as *const T) })
    }

    fn get_context_mut<T: Any>(&mut self) -> Option<&mut T> {
        let entry = match self.use_type_names_unsafe {
            true => self
                .entries
                .iter_mut()
                .rfind(|e| e.type_name == std::any::type_name::<T>())?,
            false => self
                .entries
                .iter_mut()
                .rfind(|e| e.type_id == TypeId::of::<T>())?,
        };

        Some(unsafe { &mut *(entry.value.as_mut() as *mut _ as *mut T) })
    }
}

impl ori::Proxy for Proxy {
    fn clone(&self) -> Arc<dyn ori::Proxy> {
        Arc::new(Clone::clone(self))
    }

    fn rebuild(&self) {
        let _ = self.sender.send(Event::Rebuild);
        let _ = self.proxy.send_event(());
    }

    fn event(&self, event: ori::Event) {
        let _ = self.sender.send(Event::Event(event));
        let _ = self.proxy.send_event(());
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        let _ = self.sender.send(Event::Spawn(future));
        let _ = self.proxy.send_event(());
    }
}
