use std::{
    mem,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use crate::{
    BuildCx, Canvas, Color, CursorIcon, Modifiers, Size, Tree, Update, WidgetId, Window, WindowId,
    WindowSizing,
};

mod focus;
mod key;
mod layout;
mod pointer;
mod scroll;
mod signal;

pub use signal::{RootSignal, WindowUpdate};

pub struct Root {
    pub(crate) tree:  Tree,
    pub(crate) state: RootState,
}

pub struct RootState {
    pub(crate) windows: Vec<Window>,
    pub(crate) sink:    Box<dyn Fn(RootSignal)>,
}

impl RootState {
    pub(crate) fn signal(&self, signal: RootSignal) {
        (self.sink)(signal);
    }

    pub(crate) fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub(crate) fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    pub(crate) fn windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter()
    }

    pub(crate) fn request_redraw(&self, window: WindowId) {
        self.signal(RootSignal::RequestRedraw(window));
    }

    pub(crate) fn request_animate(&self, window: WindowId) {
        self.signal(RootSignal::RequestAnimate(window));
    }

    pub(crate) fn set_window_cursor(&mut self, window: WindowId, cursor: CursorIcon) {
        if let Some(win) = self.get_window_mut(window) {
            win.cursor = cursor;

            self.signal(RootSignal::UpdateWindow(
                window,
                WindowUpdate::Cursor(cursor),
            ));
        }
    }

    pub(crate) fn set_window_title(&mut self, window: WindowId, title: String) {
        if let Some(win) = self.get_window_mut(window) {
            win.title = title.clone();

            self.signal(RootSignal::UpdateWindow(
                window,
                WindowUpdate::Title(title),
            ));
        }
    }

    pub(crate) fn set_window_sizing(&mut self, window: WindowId, sizing: WindowSizing) {
        if let Some(win) = self.get_window_mut(window) {
            win.sizing = sizing;

            self.signal(RootSignal::UpdateWindow(
                window,
                WindowUpdate::Sizing(sizing),
            ));
        }
    }

    pub(crate) fn set_window_visible(&mut self, window: WindowId, visible: bool) {
        if let Some(win) = self.get_window_mut(window) {
            win.is_visible = visible;

            self.signal(RootSignal::UpdateWindow(
                window,
                WindowUpdate::Visible(visible),
            ));
        }
    }

    pub(crate) fn set_window_decorated(&mut self, window: WindowId, decorated: bool) {
        if let Some(win) = self.get_window_mut(window) {
            win.is_decorated = decorated;

            self.signal(RootSignal::UpdateWindow(
                window,
                WindowUpdate::Decorated(decorated),
            ));
        }
    }

    pub(crate) fn set_window_color(&mut self, window: WindowId, color: Color) {
        if let Some(win) = self.get_window_mut(window) {
            win.color = color;
            self.signal(RootSignal::RequestRedraw(window));
        }
    }
}

impl Root {
    pub fn new(sink: impl Fn(RootSignal) + 'static) -> Self {
        Self {
            tree:  Tree::new(),
            state: RootState {
                windows: Vec::new(),
                sink:    Box::new(sink),
            },
        }
    }

    pub fn create_window(&mut self, contents: WidgetId) -> WindowId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let id = WindowId {
            data: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        };

        self.state.windows.push(Window {
            id,
            anchor: None,
            scale: 1.0,
            pointers: Vec::new(),
            modifiers: Modifiers::empty(),
            size: Size::new(800.0, 600.0),
            properties: Vec::new(),
            cursor: CursorIcon::Default,
            title: String::new(),
            contents,
            sizing: WindowSizing::Resizable {
                default_size: Size::new(800.0, 600.0),
                min_size:     Size::all(0.0),
                max_size:     Size::all(f32::INFINITY),
            },
            is_focused: false,
            is_visible: true,
            is_decorated: true,
            color: Color::WHITE,
        });

        self.state.signal(RootSignal::CreateWindow(id));

        if let Some(mut widget) = self.get_mut(contents) {
            widget.set_window_recursive(Some(id));
        }

        id
    }

    pub fn remove_window(&mut self, window: WindowId) {
        if let Some(window) = self.get_window(window) {
            self.remove(window.contents);
        }

        self.state.windows.retain(|w| w.id() != window);
        self.state.signal(RootSignal::RemoveWindow(window));
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.state.get_window(id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.state.get_window_mut(id)
    }

    pub fn windows(&self) -> impl Iterator<Item = &Window> {
        self.state.windows()
    }

    #[must_use]
    pub fn set_window_contents(
        &mut self,
        window: WindowId,
        contents: WidgetId,
    ) -> Option<WidgetId> {
        if let Some(mut widget) = self.get_mut(contents) {
            widget.set_window_recursive(Some(window));
            widget.request_layout();
        };

        if let Some(window) = self.get_window_mut(window) {
            Some(mem::replace(
                &mut window.contents,
                contents,
            ))
        } else {
            None
        }
    }

    pub fn set_window_title(&mut self, window: WindowId, title: String) {
        self.state.set_window_title(window, title);
    }

    pub fn set_window_sizing(&mut self, window: WindowId, sizing: WindowSizing) {
        self.state.set_window_sizing(window, sizing);
    }

    pub fn set_window_visible(&mut self, window: WindowId, visible: bool) {
        self.state.set_window_visible(window, visible);
    }

    pub fn set_window_decorated(&mut self, window: WindowId, decorated: bool) {
        self.state.set_window_decorated(window, decorated);
    }

    pub fn set_window_color(&mut self, window: WindowId, color: Color) {
        self.state.set_window_color(window, color);
    }

    #[must_use]
    pub fn draw(&mut self, window: WindowId, canvas: &mut dyn Canvas) -> Option<Size> {
        let window_id = window;
        let window = self.state.get_window(window)?;
        let sizing = window.sizing;
        let contents = window.contents;
        let pointer_position = window.pointers.first().map(|p| p.position);

        let new_window_size = {
            let mut widget = self.tree.get_mut(&mut self.state, contents).unwrap();
            layout::layout_window(&mut widget, window_id, canvas.painter())
        };

        if let Some(position) = pointer_position {
            pointer::update_hovered(self, window_id, contents, position);
        }

        let mut widget = self.tree.get_mut(&mut self.state, contents).unwrap();

        {
            let _span = tracing::info_span!("draw");
            widget.draw_recursive(canvas);
        }

        {
            let _span = tracing::info_span!("draw_over");
            widget.draw_over_recursive(canvas);
        }

        matches!(sizing, WindowSizing::FitContent).then_some(new_window_size)
    }

    pub fn animate(&mut self, window: WindowId, delta_time: Duration) {
        let Some(window) = self.state.get_window_mut(window) else {
            return;
        };

        let _span = tracing::info_span!("animate");

        let contents = window.contents;
        let mut widget = self.tree.get_mut(&mut self.state, contents).unwrap();
        widget.animate_recursive(delta_time);
    }

    pub fn window_focused(&mut self, window: WindowId, is_focused: bool) {
        let Some(window) = self.state.get_window_mut(window) else {
            return;
        };

        window.is_focused = is_focused;

        let contents = window.contents;
        let mut widget = self.tree.get_mut(&mut self.state, contents).unwrap();
        let update = Update::WindowFocused(is_focused);
        widget.update_recursive(update);
    }

    pub fn window_resized(&mut self, window: WindowId, new_size: Size) {
        let Some(window) = self.state.get_window_mut(window) else {
            return;
        };

        window.size = new_size;

        let contents = window.contents;
        let mut widget = self.tree.get_mut(&mut self.state, contents).unwrap();
        let update = Update::WindowResized(new_size);
        widget.update_recursive(update);
        widget.request_layout();
    }

    pub fn window_scaled(&mut self, window: WindowId, new_scale: f32, new_size: Size) {
        let Some(window) = self.state.get_window_mut(window) else {
            return;
        };

        window.scale = new_scale;
        window.size = new_size;

        let contents = window.contents;
        let mut widget = self.tree.get_mut(&mut self.state, contents).unwrap();
        let update = Update::WindowScaleChanged(new_scale);
        widget.update_recursive(update);
    }
}

impl BuildCx for Root {
    fn root(&self) -> &Root {
        self
    }

    fn root_mut(&mut self) -> &mut Root {
        self
    }
}
