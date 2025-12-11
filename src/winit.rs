use std::{any::Any, sync::mpsc::Receiver, time::Instant};

use ike_core::{
    Modifiers, Offset, Point, PointerButton, PointerId, ScrollDelta, Size, WindowSizing,
};
use ike_skia::{
    SkiaPainter,
    vulkan::{VulkanContext, VulkanWindow},
};
use ori::AsyncContext;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    Context,
    app::{AnyEffect, UiBuilder},
    context::Event,
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
    let vulkan = VulkanContext::new(&event_loop);

    let mut painter = SkiaPainter::new();
    painter.load_font(
        include_bytes!("InterVariable.ttf"),
        None,
    );

    let (sender, receiver) = std::sync::mpsc::channel();
    let context = Context {
        app: ike_core::AppState::new(),
        proxy: event_loop.create_proxy(),
        entries: Vec::new(),
        sender,

        use_type_names_unsafe: false,
    };

    let view = build(data);

    let mut state = AppState {
        data,

        build,
        view,
        state: None,

        runtime,
        receiver,

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

    painter: SkiaPainter,
    context: Context,
    windows: Vec<WindowState>,

    vulkan: VulkanContext,
}

struct WindowState {
    animate: Option<Instant>,

    vulkan: VulkanWindow,

    id:       ike_core::WindowId,
    window:   Window,
    min_size: Size,
    max_size: Size,
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
        self.update_windows(event_loop);
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
                    self.context.app.animate(window.id, delta_time);

                    if self.context.app.window_needs_animate(window.id) {
                        window.animate = Some(Instant::now());
                        window.window.request_redraw();
                    }
                }

                let desc = self.context.app.get_window(window.id).unwrap();

                let new_window_size = window.vulkan.draw(
                    &mut self.painter,
                    desc.color,
                    window.window.scale_factor() as f32,
                    || window.window.pre_present_notify(),
                    |canvas| self.context.app.draw(window.id, canvas),
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
                        self.context.app.window_resized(window.id, size);
                    }

                    WindowEvent::ScaleFactorChanged { .. } => {
                        (self.context.app).window_scaled(window.id, scale as f32, size);
                    }

                    _ => unreachable!(),
                }
            }

            WindowEvent::Focused(is_focused) => {
                self.context.app.window_focused(window.id, is_focused);
            }

            WindowEvent::CursorEntered { device_id } => {
                let id = PointerId::from_hash(device_id);
                self.context.app.pointer_entered(window.id, id);
            }

            WindowEvent::CursorLeft { device_id } => {
                let id = PointerId::from_hash(device_id);
                self.context.app.pointer_left(window.id, id);
            }

            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let position = position.to_logical(window.window.scale_factor());
                let position = Point::new(position.x, position.y);

                self.context.app.pointer_moved(
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

                self.context.app.pointer_scrolled(
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

                self.context.app.pointer_button(
                    window.id,
                    PointerId::from_hash(device_id),
                    button,
                    pressed,
                );
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = matches!(event.state, ElementState::Pressed);

                self.context.app.key_press(
                    window.id,
                    convert_winit_key(event.logical_key),
                    event.repeat,
                    event.text.as_deref(),
                    pressed,
                );
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

                self.context.app.modifiers_changed(window.id, modifiers);
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => {}
        }

        self.handle_events();
        self.update_windows(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: ()) {
        self.handle_events();
        self.update_windows(event_loop);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.painter.cleanup();
    }
}

impl<T> AppState<'_, T> {
    fn handle_events(&mut self) {
        for event in self.receiver.try_iter() {
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
            }
        }
    }

    fn update_windows(&mut self, event_loop: &ActiveEventLoop) {
        for desc in self.context.app.windows() {
            match self.windows.iter_mut().find(|w| w.id == desc.id()) {
                Some(window) => window.update(desc),

                None => {
                    self.windows.push(WindowState::new(
                        &mut self.vulkan,
                        event_loop,
                        desc,
                    ));
                }
            }
        }

        for window in &mut self.windows {
            if self.context.app.window_needs_animate(window.id) && window.animate.is_none() {
                window.animate = Some(Instant::now());
                window.window.request_redraw();
            }

            if self.context.app.window_needs_draw(window.id) {
                window.window.request_redraw();
            }
        }

        self.windows.retain(|state| {
            let Some(_window) = self.context.app.get_window(state.id) else {
                return false;
            };

            true
        });
    }
}

impl WindowState {
    fn new(
        vulkan: &mut VulkanContext,
        event_loop: &ActiveEventLoop,
        desc: &ike_core::Window,
    ) -> Self {
        use winit::dpi::LogicalSize;

        let size = match desc.sizing {
            WindowSizing::FitContent => LogicalSize::new(
                desc.current_size().width,
                desc.current_size().height,
            ),
            WindowSizing::Resizable { default_size, .. } => {
                LogicalSize::new(default_size.width, default_size.height)
            }
        };

        let min_size = match desc.sizing {
            WindowSizing::FitContent => size,
            WindowSizing::Resizable { min_size, .. } => {
                LogicalSize::new(min_size.width, min_size.height)
            }
        };

        let max_size = match desc.sizing {
            WindowSizing::FitContent => size,
            WindowSizing::Resizable { max_size, .. } => {
                LogicalSize::new(max_size.width, max_size.height)
            }
        };

        let attributes = Window::default_attributes()
            .with_title(&desc.title)
            .with_visible(desc.visible)
            .with_decorations(desc.decorated)
            .with_transparent(true)
            .with_min_inner_size(min_size)
            .with_max_inner_size(max_size)
            .with_inner_size(size)
            .with_cursor(desc.cursor())
            .with_resizable(matches!(
                desc.sizing,
                WindowSizing::Resizable { .. }
            ));

        let window = event_loop.create_window(attributes).unwrap();

        let vulkan = unsafe {
            let physical = window.inner_size();
            VulkanWindow::new(
                vulkan,
                &window,
                physical.width,
                physical.height,
            )
        };

        let (min_size, max_size) = match desc.sizing {
            WindowSizing::FitContent => (Size::ZERO, Size::ZERO),
            WindowSizing::Resizable {
                min_size, max_size, ..
            } => (min_size, max_size),
        };

        Self {
            animate: None,
            id: desc.id(),
            vulkan,
            window,
            min_size,
            max_size,
        }
    }

    fn update(&mut self, desc: &ike_core::Window) {
        if self.window.title() != desc.title {
            self.window.set_title(&desc.title);
        }

        if let WindowSizing::Resizable {
            min_size, max_size, ..
        } = desc.sizing
        {
            if self.min_size != min_size {
                self.window.set_min_inner_size(Some(LogicalSize::new(
                    min_size.width,
                    min_size.height,
                )));
                self.min_size = min_size;
            }

            if self.max_size != max_size {
                self.window.set_max_inner_size(Some(LogicalSize::new(
                    max_size.width,
                    max_size.height,
                )));
                self.max_size = max_size;
            }
        }

        self.window.set_decorations(desc.decorated);
        self.window.set_visible(desc.visible);
        self.window.set_cursor(desc.cursor());
    }

    fn resize(&mut self) {
        let size = self.window.inner_size();

        (self.vulkan).resize(size.width, size.height);
    }
}
