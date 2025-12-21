#![warn(clippy::unwrap_used)]

use std::{
    any::Any,
    io,
    pin::Pin,
    sync::{Arc, mpsc::Receiver},
    time::Instant,
};

use ike_core::{
    Modifiers, Offset, Point, PointerButton, PointerId, RootSignal, ScrollDelta, Size,
    WindowSizing, WindowUpdate,
};
use ike_skia::{SkiaPainter, vulkan::Surface};
use ori::Proxyable;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    error::{EventLoopError, OsError},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowId},
};

use crate::proxy::Proxy;

mod key;
mod proxy;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("vulkan error: {0}")]
    Vulkan(#[from] ike_skia::vulkan::Error),

    #[error("os error: {0}")]
    Os(#[from] OsError),

    #[error("event loop: {0}")]
    EventLoop(#[from] EventLoopError),

    #[error("handle error: {0}")]
    Handle(#[from] HandleError),

    #[error("clipboard error: {0}")]
    Clipboard(Box<dyn std::error::Error + Send + Sync>),

    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

pub fn run<T>(data: &mut T, mut build: ike_ori::UiBuilder<T>) -> Result<(), Error> {
    let rt;
    let _rt_guard = if tokio::runtime::Handle::try_current().is_err() {
        rt = Some(tokio::runtime::Runtime::new()?);
        rt.as_ref().map(|rt| rt.enter())
    } else {
        None
    };

    let runtime = tokio::runtime::Handle::current();
    let event_loop = EventLoop::with_user_event().build()?;
    let display_handle = event_loop.display_handle()?;
    let vulkan = ike_skia::vulkan::Context::new(display_handle)?;

    let mut painter = SkiaPainter::new();
    painter.load_font(
        include_bytes!("../../fonts/InterVariable.ttf"),
        None,
    );

    let (sender, receiver) = std::sync::mpsc::channel();
    let root = ike_core::Root::new({
        let sender = sender.clone();

        move |signal| {
            let _ = sender.send(Event::Signal(signal));
        }
    });
    let proxy = Proxy::new(sender, event_loop.create_proxy());
    let context = ike_ori::Context::new(root, Arc::new(proxy));

    let view = build(data);

    #[cfg(target_os = "linux")]
    let clipboard = {
        use copypasta::{
            wayland_clipboard::create_clipboards_from_external,
            x11_clipboard::{Clipboard, X11ClipboardContext},
        };
        use winit::raw_window_handle::RawDisplayHandle;

        if let RawDisplayHandle::Wayland(handle) = display_handle.as_raw() {
            unsafe {
                let display = handle.display.as_ptr();
                let (_primary, clipboard) = create_clipboards_from_external(display);
                Box::new(clipboard) as Box<_>
            }
        } else {
            let clipboard = X11ClipboardContext::<Clipboard>::new().map_err(Error::Clipboard)?;
            Box::new(clipboard) as Box<_>
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
        result: Ok(()),
    };

    event_loop.run_app(&mut state)?;

    state.result
}

enum Event {
    Rebuild,
    Event(ori::Event),
    Spawn(Pin<Box<dyn Future<Output = ()> + Send>>),
    Signal(RootSignal),
}

struct AppState<'a, T> {
    data: &'a mut T,

    build: ike_ori::UiBuilder<T>,
    view:  ike_ori::AnyEffect<T>,
    state: Option<Box<dyn Any>>,

    runtime:  tokio::runtime::Handle,
    receiver: Receiver<Event>,

    clipboard: Box<dyn copypasta::ClipboardProvider>,
    painter:   SkiaPainter,
    context:   ike_ori::Context,
    windows:   Vec<WindowState>,

    vulkan: ike_skia::vulkan::Context,
    result: Result<(), Error>,
}

struct WindowState {
    animate: Option<Instant>,

    surface: Surface,

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
            tracing::error!(
                window = ?window_id,
                event = ?event,
                "window event received for invalid window",
            );

            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                if let Some(animate) = window.animate.take() {
                    let delta_time = animate.elapsed();
                    self.context.animate(window.id, delta_time);
                }

                let Some(desc) = self.context.get_window(window.id) else {
                    tracing::error!("window redraw request before it has been created");
                    return;
                };

                let Ok(new_window_size) = window.surface.draw(
                    &mut self.painter,
                    desc.color(),
                    window.window.scale_factor() as f32,
                    || window.window.pre_present_notify(),
                    |canvas| self.context.draw(window.id, canvas),
                ) else {
                    tracing::error!("drawing failed");
                    return;
                };

                if let Some(size) = new_window_size.flatten() {
                    let size = LogicalSize::new(size.width, size.height);

                    window.window.set_min_inner_size(Some(size));
                    window.window.set_max_inner_size(Some(size));
                }
            }

            WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
                if let Err(err) = window.resize() {
                    tracing::error!("failed to resize window: {err}");
                    return;
                };

                let scale = window.window.scale_factor();
                let size = window.window.inner_size().to_logical(scale);

                let scale = scale as f32;
                let size = Size::new(size.width, size.height);

                match event {
                    WindowEvent::Resized(..) => {
                        self.context.window_resized(window.id, size);
                    }

                    WindowEvent::ScaleFactorChanged { .. } => {
                        self.context.window_scaled(window.id, scale, size);
                    }

                    _ => unreachable!(),
                }
            }

            WindowEvent::Focused(is_focused) => {
                self.context.window_focused(window.id, is_focused);
            }

            WindowEvent::CursorEntered { device_id } => {
                let pointer_id = PointerId::from_hash(device_id);
                self.context.pointer_entered(window.id, pointer_id);
            }

            WindowEvent::CursorLeft { device_id } => {
                let pointer_id = PointerId::from_hash(device_id);
                self.context.pointer_left(window.id, pointer_id);
            }

            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let pointer_id = PointerId::from_hash(device_id);
                let position = position.to_logical(window.window.scale_factor());
                let position = Point::new(position.x, position.y);

                self.context.pointer_moved(window.id, pointer_id, position);
            }

            WindowEvent::MouseWheel {
                device_id, delta, ..
            } => {
                let pointer_id = PointerId::from_hash(device_id);
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => ScrollDelta::Line(Offset::new(x, y)),
                    MouseScrollDelta::PixelDelta(delta) => ScrollDelta::Pixel(
                        Offset::new(delta.x as f32, delta.y as f32)
                            / window.window.scale_factor() as f32,
                    ),
                };

                self.context.pointer_scrolled(window.id, pointer_id, delta);
            }

            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let pointer_id = PointerId::from_hash(device_id);
                let pressed = matches!(state, ElementState::Pressed);
                let button = match button {
                    MouseButton::Left => PointerButton::Primary,
                    MouseButton::Right => PointerButton::Secondary,
                    MouseButton::Middle => PointerButton::Tertiary,
                    MouseButton::Back => PointerButton::Backward,
                    MouseButton::Forward => PointerButton::Forward,
                    MouseButton::Other(i) => PointerButton::Other(i),
                };

                (self.context).pointer_pressed(window.id, pointer_id, button, pressed);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let action_mod = match self.context.get_window(window.id) {
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
                        self.context.text_pasted(window.id, text);
                    }
                } else {
                    self.context.key_pressed(
                        window.id,
                        key::convert_winit_key(event.logical_key),
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

                self.context.modifiers_changed(window.id, modifiers);
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
            if let Err(error) = self.handle_event(event_loop, event) {
                tracing::error!("{error}");
            }
        }
    }

    fn handle_event(&mut self, event_loop: &ActiveEventLoop, event: Event) -> Result<(), Error> {
        match event {
            Event::Rebuild => {
                if let Some(ref mut state) = self.state {
                    let mut view = (self.build)(self.data);
                    ori::View::rebuild(
                        &mut view,
                        &mut ori::NoElement,
                        state,
                        &mut self.context,
                        self.data,
                        &mut self.view,
                    );

                    self.view = view;
                }
            }

            Event::Event(mut event) => {
                if let Some(ref mut state) = self.state {
                    let action = ori::View::event(
                        &mut self.view,
                        &mut ori::NoElement,
                        state,
                        &mut self.context,
                        self.data,
                        &mut event,
                    );

                    self.context.send_action(action);
                }
            }

            Event::Spawn(future) => {
                self.runtime.spawn(future);
            }

            Event::Signal(signal) => {
                self.handle_signal(event_loop, signal)?;
            }
        }

        Ok(())
    }

    fn handle_signal(
        &mut self,
        event_loop: &ActiveEventLoop,
        signal: RootSignal,
    ) -> Result<(), Error> {
        match signal {
            RootSignal::RequestRedraw(id) => {
                if let Some(window) = self.windows.iter().find(|w| w.id == id) {
                    window.window.request_redraw();
                }
            }

            RootSignal::RequestAnimate(id, last_frame) => {
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id)
                    && window.animate.is_none()
                {
                    window.animate = Some(last_frame);
                    window.window.request_redraw();
                }
            }

            RootSignal::ClipboardSet(text) => {
                let _ = self.clipboard.set_contents(text);
            }

            RootSignal::CreateWindow(id) => {
                if let Some(window) = self.context.get_window(id) {
                    let window = WindowState::new(&mut self.vulkan, event_loop, window)?;
                    self.windows.push(window);
                }
            }

            RootSignal::RemoveWindow(id) => {
                self.windows.retain(|w| w.id != id);
            }

            RootSignal::UpdateWindow(id, update) => {
                let Some(win) = self.windows.iter_mut().find(|w| w.id == id) else {
                    return Ok(());
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

            RootSignal::Ime(..) => {}
        }

        Ok(())
    }
}

impl WindowState {
    fn new(
        vulkan: &mut ike_skia::vulkan::Context,
        event_loop: &ActiveEventLoop,
        desc: &ike_core::Window,
    ) -> Result<Self, Error> {
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

        let window = event_loop.create_window(attributes)?;

        let surface = unsafe {
            let physical = window.inner_size();
            Surface::new(
                vulkan,
                window.display_handle()?,
                window.window_handle()?,
                physical.width,
                physical.height,
            )?
        };

        Ok(Self {
            id: desc.id(),
            animate: None,
            surface,
            window,
        })
    }

    fn resize(&mut self) -> Result<(), Error> {
        let size = self.window.inner_size();
        self.surface.resize(size.width, size.height)?;
        Ok(())
    }
}
