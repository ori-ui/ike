use std::{
    collections::HashSet,
    hash::BuildHasherDefault,
    mem,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use crate::{
    Arena, BuildCx, Canvas, Color, CursorIcon, Modifiers, Point, Recorder, Size, Update, WidgetId,
    WidgetMut, WidgetRef, Window, WindowId, WindowSizing, debug::debug_panic, event::TouchSettings,
    window::Layer,
};

mod focus;
mod key;
mod layout;
mod pointer;
mod query;
mod scroll;
mod signal;
mod text;
mod touch;

use seahash::SeaHasher;
pub use signal::{ImeSignal, Signal, WindowUpdate};

pub struct World {
    pub(crate) arena: Arena,
    pub(crate) state: WorldState,
}

pub struct WorldState {
    pub(crate) windows:        Vec<Window>,
    pub(crate) propagate:      HashSet<WidgetId, BuildHasherDefault<SeaHasher>>,
    pub(crate) sink:           Box<dyn Fn(Signal)>,
    pub(crate) trace:          bool,
    pub(crate) touch_settings: TouchSettings,
    pub(crate) recorder:       Recorder,
}

impl WorldState {
    pub(crate) fn signal(&self, signal: Signal) {
        tracing::trace!(
            signal = ?signal,
            "root signal emitted",
        );

        (self.sink)(signal);
    }

    pub(crate) fn window(&self, id: WindowId) -> Option<&Window> {
        match self.get_window(id) {
            Some(window) => Some(window),
            None => {
                debug_panic!("invalid window `{id:?}`");
                None
            }
        }
    }

    pub(crate) fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        match self.get_window_mut(id) {
            Some(window) => Some(window),
            None => {
                debug_panic!("invalid window `{id:?}`");
                None
            }
        }
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    pub(crate) fn windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter()
    }

    pub(crate) fn request_propagate(&mut self, widget: WidgetId) {
        self.propagate.insert(widget);
    }

    pub(crate) fn request_redraw(&self, window: WindowId) {
        self.signal(Signal::RequestRedraw(window));
    }

    pub(crate) fn request_animate(&self, window: WindowId) {
        self.signal(Signal::RequestAnimate(
            window,
            Instant::now(),
        ));
    }

    pub(crate) fn set_window_cursor(&mut self, window: WindowId, cursor: CursorIcon) {
        if let Some(win) = self.window_mut(window) {
            win.cursor = cursor;

            self.signal(Signal::UpdateWindow(
                window,
                WindowUpdate::Cursor(cursor),
            ));
        }
    }

    pub(crate) fn set_window_title(&mut self, window: WindowId, title: String) {
        if let Some(win) = self.window_mut(window) {
            win.title = title.clone();

            self.signal(Signal::UpdateWindow(
                window,
                WindowUpdate::Title(title),
            ));
        }
    }

    pub(crate) fn set_window_sizing(&mut self, window: WindowId, sizing: WindowSizing) {
        if let Some(win) = self.window_mut(window) {
            win.sizing = sizing;

            self.signal(Signal::UpdateWindow(
                window,
                WindowUpdate::Sizing(sizing),
            ));
        }
    }

    pub(crate) fn set_window_visible(&mut self, window: WindowId, visible: bool) {
        if let Some(win) = self.window_mut(window) {
            win.is_visible = visible;

            self.signal(Signal::UpdateWindow(
                window,
                WindowUpdate::Visible(visible),
            ));
        }
    }

    pub(crate) fn set_window_decorated(&mut self, window: WindowId, decorated: bool) {
        if let Some(win) = self.window_mut(window) {
            win.is_decorated = decorated;

            self.signal(Signal::UpdateWindow(
                window,
                WindowUpdate::Decorated(decorated),
            ));
        }
    }

    pub(crate) fn set_window_color(&mut self, window: WindowId, color: Color) {
        if let Some(win) = self.window_mut(window) {
            win.color = color;
            self.signal(Signal::RequestRedraw(window));
        }
    }
}

impl World {
    pub fn new(sink: impl Fn(Signal) + 'static) -> Self {
        Self {
            arena: Arena::new(),
            state: WorldState {
                windows:        Vec::new(),
                propagate:      HashSet::default(),
                sink:           Box::new(sink),
                trace:          false,
                touch_settings: TouchSettings::default(),
                recorder:       Recorder::new(),
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
            layers: Rc::new(vec![Layer {
                root:     contents,
                size:     Size::new(800.0, 600.0),
                position: Point::ORIGIN,
            }]),

            modifiers: Modifiers::empty(),
            pointers: Vec::new(),
            touches: Vec::new(),

            focused: None,

            properties: Vec::new(),

            scale: 1.0,
            size: Size::new(800.0, 600.0),
            is_visible: true,
            is_focused: false,
            is_decorated: true,

            cursor: CursorIcon::Default,
            title: String::new(),
            sizing: WindowSizing::Resizable {
                default_size: Size::new(800.0, 600.0),
                min_size:     Size::all(0.0),
                max_size:     Size::all(f32::INFINITY),
            },
            color: Color::WHITE,
        });

        self.state.signal(Signal::CreateWindow(id));

        if let Some(mut widget) = self.widget_mut(contents) {
            widget.cx.set_window_recursive(Some(id));
        }

        id
    }

    pub fn remove_window(&mut self, window: WindowId) {
        if let Some(index) = self.state.windows.iter().position(|w| w.id == window) {
            let window = self.state.windows.remove(index);

            for layer in window.layers() {
                self.remove_widget(layer.root);
            }
        }

        self.state.signal(Signal::RemoveWindow(window));
    }

    pub(crate) fn window(&self, id: WindowId) -> Option<&Window> {
        self.state.window(id)
    }

    pub(crate) fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.state.window_mut(id)
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

    pub fn set_window_layer(&mut self, window: WindowId, contents: WidgetId) {
        if let Some(mut widget) = self.widget_mut(contents) {
            widget.cx.set_window_recursive(Some(window));
            widget.cx.request_layout();
        }

        if let Some(window) = self.window_mut(window) {
            let layers = Rc::make_mut(&mut window.layers);

            if let Some(layer) = layers.get_mut(0) {
                let prev = mem::replace(&mut layer.root, contents);
                self.remove_widget(prev);
            }
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

    fn handle_updates(&mut self) {
        for widget in mem::take(&mut self.state.propagate) {
            self.propagate_state(widget);
        }
    }

    fn propagate_state(&mut self, widget: WidgetId) {
        let mut current = Some(widget);

        while let Some(widget) = current
            && let Some(mut widget) = self.widget_mut(widget)
        {
            widget.cx.update_state();
            current = widget.cx.parent();
        }
    }

    #[must_use]
    pub fn draw(&mut self, window: WindowId, canvas: &mut dyn Canvas) -> Option<Size> {
        self.handle_updates();

        let new_window_size = layout::layout_window(self, window, canvas.painter());
        layout::compose_window(self, window);
        pointer::update_window_hovered(self, window);

        let window = self.state.window(window)?;

        for layer in window.layers.clone().iter() {
            let Some(mut widget) = self.arena.get_mut(&mut self.state, layer.root) else {
                debug_panic!("invalid widget");
                continue;
            };

            {
                let _span = tracing::info_span!("draw");
                widget.draw_recursive(canvas);
            }

            {
                let _span = tracing::info_span!("record");
                widget.record_recursive(canvas);
            }
        }

        self.state.recorder.frame(&self.arena);

        new_window_size
    }

    pub fn animate(&mut self, window: WindowId, delta_time: Duration) {
        self.handle_updates();

        let window_id = window;
        let Some(window) = self.state.window(window) else {
            tracing::error!(
                window = ?window_id,
                "`animate` called a on window that doesn't exist",
            );

            return;
        };

        let _span = tracing::info_span!("animate");

        for layer in window.layers.clone().iter() {
            if let Some(mut widget) = self.arena.get_mut(&mut self.state, layer.root) {
                widget.animate_recursive(delta_time);
            }
        }
    }

    pub fn window_focused(&mut self, window: WindowId, is_focused: bool) {
        self.handle_updates();

        let window_id = window;
        let Some(window) = self.state.window_mut(window_id) else {
            tracing::error!(
                window = ?window_id,
                "`window_focused` called on a window that doesn't exist",
            );

            return;
        };

        window.is_focused = is_focused;

        for layer in window.layers.clone().iter() {
            if let Some(mut widget) = self.get_widget_mut(layer.root) {
                let update = Update::WindowFocused(is_focused);
                widget.update_recursive(update);
            }
        }

        let Some(window) = self.state.window(window_id) else {
            return;
        };

        // if the window was left in an ime session, we need start it again
        if let Some(focused) = window.focused
            && let Some(widget) = self.widget(focused)
            && widget.cx.state.accepts_text
            && is_focused
        {
            widget.cx.world.signal(Signal::Ime(ImeSignal::Start));
        }
    }

    pub fn window_resized(&mut self, window: WindowId, new_size: Size) {
        self.handle_updates();

        let window_id = window;
        let Some(window) = self.state.window_mut(window_id) else {
            tracing::error!(
                window = ?window_id,
                "`window_resized` called on a window that doesn't exist",
            );

            return;
        };

        window.size = new_size;

        for layer in window.layers.clone().iter() {
            if let Some(mut widget) = self.arena.get_mut(&mut self.state, layer.root) {
                let update = Update::WindowResized(new_size);
                widget.update_recursive(update);
                widget.cx.request_layout();
            }
        }
    }

    pub fn window_scaled(&mut self, window: WindowId, new_scale: f32, new_size: Size) {
        self.handle_updates();

        let window_id = window;
        let Some(window) = self.state.window_mut(window_id) else {
            tracing::error!(
                window = ?window_id,
                "`window_scaled` called on a window that doesn't exist",
            );

            return;
        };

        window.scale = new_scale;
        window.size = new_size;

        for layer in window.layers.clone().iter() {
            if let Some(mut widget) = self.arena.get_mut(&mut self.state, layer.root) {
                let update = Update::WindowScaleChanged(new_scale);
                widget.update_recursive(update);
                widget.cx.request_layout();
            }
        }
    }

    fn widget(&self, id: WidgetId) -> Option<WidgetRef<'_>> {
        match self.get_widget(id) {
            Some(widget) => Some(widget),
            None => {
                debug_panic!("invalid widget {id:?}");

                None
            }
        }
    }

    fn widget_mut(&mut self, id: WidgetId) -> Option<WidgetMut<'_>> {
        match self.get_widget_mut(id) {
            Some(widget) => Some(widget),
            None => {
                debug_panic!("invalid widget {id:?}");

                None
            }
        }
    }
}

impl BuildCx for World {
    fn world(&self) -> &World {
        self
    }

    fn world_mut(&mut self) -> &mut World {
        self
    }
}
