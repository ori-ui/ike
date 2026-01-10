mod settings;
mod signal;
mod state;
mod widget_mut;
mod widget_ref;
mod widgets;

use std::{
    ops::Range,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

pub(crate) use state::WorldState;
pub(crate) use widgets::Widgets;

pub use settings::Settings;
pub use signal::{ImeSignal, Signal, WindowUpdate};
pub use widget_mut::WidgetMut;
pub use widget_ref::WidgetRef;
pub use widgets::AnyWidget;

use crate::{
    AnyWidgetId, Builder, Canvas, Color, CursorIcon, Key, Layer, LayerId, Modifiers, Offset,
    Padding, Point, PointerButton, PointerId, Recorder, ScrollDelta, Size, TouchId, Update,
    WidgetId, Window, WindowId, WindowSizing, debug::debug_panic, passes,
};

pub struct World {
    pub(crate) widgets: Widgets,
    pub(crate) state:   WorldState,
}

impl World {
    pub fn new(signaller: Box<dyn Fn(Signal)>, settings: Settings) -> Self {
        Self {
            widgets: Widgets::new(),
            state:   WorldState::new(signaller, settings),
        }
    }
}

impl World {
    pub fn settings(&self) -> &Settings {
        &self.state.settings
    }

    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.state.settings
    }

    pub fn recorder(&mut self) -> &Recorder {
        &self.state.recorder
    }

    pub fn recorder_mut(&mut self) -> &mut Recorder {
        &mut self.state.recorder
    }
}

impl World {
    pub fn create_window(&mut self, contents: WidgetId) -> WindowId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let id = WindowId {
            data: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        };

        self.state.windows.push(Window {
            id,
            layers: Rc::new(vec![Layer {
                id:       Layer::next_id(),
                widget:   contents,
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
            insets: Padding::all(0.0),
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

        self.state.emit_signal(Signal::CreateWindow(id));
        passes::hierarchy::set_window(self, contents, Some(id));

        id
    }

    pub fn remove_window(&mut self, window: WindowId) {
        if let Some(index) = self.state.windows.iter().position(|w| w.id == window) {
            let window = self.state.windows.remove(index);

            for layer in window.layers() {
                self.remove_widget(layer.widget);
            }
        }

        self.state.emit_signal(Signal::RemoveWindow(window));
    }

    pub fn set_window_widget(&mut self, window: WindowId, widget: WidgetId) {
        passes::hierarchy::set_window(self, widget, Some(window));

        if let Some(window) = self.state.window(window)
            && let Some(layer) = window.layers.first().cloned()
        {
            self.remove_widget(layer.widget);
        }

        if let Some(window) = self.state.window_mut(window)
            && let Some(layer) = window.get_base_layer_mut()
        {
            layer.widget = widget;
        }
    }

    pub fn add_layer(
        &mut self,
        window: WindowId,
        position: Point,
        widget: impl AnyWidgetId,
    ) -> LayerId {
        let widget = widget.upcast();

        passes::hierarchy::set_window(self, widget, Some(window));

        let id = Layer::next_id();
        let layer = Layer {
            id,
            widget,
            position,
            size: self
                .widget(widget)
                .map_or(Size::ZERO, |widget| widget.cx.size()),
        };

        if let Some(window) = self.state.window_mut(window) {
            let layers = Rc::make_mut(&mut window.layers);
            layers.push(layer);
        }

        if let Some(mut widget) = self.widget_mut(widget) {
            debug_assert!(
                widget.cx.parent().is_none(),
                "layer widgets mut be orphans",
            );

            widget.cx.state.transform.offset = Offset {
                x: position.x,
                y: position.y,
            };

            widget.cx.request_compose();
        }

        id
    }

    pub fn set_layer_position(&mut self, window: WindowId, layer: LayerId, position: Point) {
        if let Some(window) = self.state.window_mut(window)
            && let Some(layer) = window.get_layer_mut(layer)
        {
            layer.position = position;
            let widget = layer.widget;

            if let Some(mut widget) = self.widget_mut(widget) {
                widget.cx.state.transform.offset = Offset {
                    x: position.x,
                    y: position.y,
                };

                widget.cx.request_compose();
            }
        }
    }

    pub fn set_layer_widget(&mut self, window: WindowId, layer: LayerId, widget: impl AnyWidgetId) {
        let widget = widget.upcast();

        passes::hierarchy::set_window(self, widget, Some(window));

        if let Some(window) = self.state.window_mut(window)
            && let Some(layer) = window.get_layer_mut(layer)
        {
            layer.widget = widget;

            if let Some(mut widget) = self.widget_mut(widget) {
                widget.cx.request_compose();
            }
        }
    }

    pub fn remove_layer(&mut self, window: WindowId, layer: LayerId) {
        if let Some(window) = self.state.window_mut(window) {
            let layers = Rc::make_mut(&mut window.layers);

            if let Some(index) = layers.iter().position(|l| l.id == layer) {
                debug_assert!(index > 0, "cannot remove base layer");

                let layer = layers.remove(index);
                self.remove_widget(layer.widget);
            } else {
                debug_panic!("tried to remove invalid layer");
            }
        }

        self.state.request_redraw(window);
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

    pub fn window_resized(&mut self, window: WindowId, new_size: Size) {
        let window_id = window;

        let Some(window) = self.window_mut(window) else {
            return;
        };

        window.size = new_size;

        let update = Update::WindowResized(new_size);
        passes::update::window(self, window_id, &update);
    }

    pub fn window_scaled(&mut self, window: WindowId, new_size: Size, new_scale: f32) {
        let window_id = window;

        let Some(window) = self.window_mut(window) else {
            return;
        };

        window.size = new_size;
        window.scale = new_scale;

        let update = Update::WindowResized(new_size);
        passes::update::window(self, window_id, &update);

        let update = Update::WindowScaled(new_scale);
        passes::update::window(self, window_id, &update);
    }

    pub fn window_inset(&mut self, window: WindowId, insets: Padding) {
        let window_id = window;

        let Some(window) = self.window_mut(window) else {
            return;
        };

        window.insets = insets;

        let update = Update::WindowInset(insets);
        passes::update::window(self, window_id, &update);
    }

    pub fn window_focused(&mut self, window: WindowId, is_focused: bool) {
        let window_id = window;

        let Some(window) = self.state.window_mut(window_id) else {
            return;
        };

        window.is_focused = is_focused;

        passes::update::window(
            self,
            window_id,
            &Update::WindowFocused(is_focused),
        );

        if let Some(window) = self.state.window_mut(window_id)
            && let Some(focused) = window.focused
            && let Some(widget) = self.widget(focused)
            && widget.cx.hierarchy.accepts_text()
        {
            widget.cx.world.emit_signal(Signal::Ime(ImeSignal::Start));
        }
    }
}

impl World {
    pub fn pointer_entered(&mut self, window: WindowId, pointer: PointerId) -> bool {
        passes::pointer::entered(self, window, pointer)
    }

    pub fn pointer_left(&mut self, window: WindowId, pointer: PointerId) -> bool {
        passes::pointer::left(self, window, pointer)
    }

    pub fn pointer_moved(&mut self, window: WindowId, pointer: PointerId, position: Point) -> bool {
        passes::pointer::moved(self, window, pointer, position)
    }

    pub fn pointer_pressed(
        &mut self,
        window: WindowId,
        pointer: PointerId,
        button: PointerButton,
        pressed: bool,
    ) -> bool {
        passes::pointer::pressed(self, window, pointer, button, pressed)
    }

    pub fn pointer_scrolled(
        &mut self,
        window: WindowId,
        pointer: PointerId,
        delta: ScrollDelta,
    ) -> bool {
        passes::pointer::scrolled(self, window, pointer, delta)
    }
}

impl World {
    pub fn modifiers_changed(&mut self, window: WindowId, modifiers: Modifiers) -> bool {
        passes::key::modifiers_changed(self, window, modifiers)
    }

    pub fn key_pressed(
        &mut self,
        window: WindowId,
        key: Key,
        repeat: bool,
        text: Option<&str>,
        pressed: bool,
    ) -> bool {
        passes::key::pressed(self, window, key, repeat, text, pressed)
    }
}

impl World {
    pub fn text_pasted(&mut self, window: WindowId, text: String) -> bool {
        passes::text::pasted(self, window, text)
    }

    pub fn ime_commit_text(&mut self, window: WindowId, text: String) -> bool {
        passes::text::ime_commit(self, window, text)
    }

    pub fn ime_select(&mut self, window: WindowId, selection: Range<usize>) -> bool {
        passes::text::ime_select(self, window, selection)
    }
}

impl World {
    pub fn touch_down(&mut self, window: WindowId, touch: TouchId, position: Point) -> bool {
        passes::touch::down(self, window, touch, position)
    }

    pub fn touch_up(&mut self, window: WindowId, touch: TouchId, position: Point) -> bool {
        passes::touch::up(self, window, touch, position)
    }

    pub fn touch_move(&mut self, window: WindowId, touch: TouchId, position: Point) -> bool {
        passes::touch::moved(self, window, touch, position)
    }
}

impl World {
    pub fn animate(&mut self, window: WindowId, delta_time: Duration) {
        passes::animate::animate_window(self, window, delta_time);
    }

    pub fn draw(&mut self, window: WindowId, canvas: &mut dyn Canvas) -> Option<Size> {
        let size = passes::layout::layout_window(self, window, canvas.painter());
        passes::compose::compose_window(self, window);
        passes::pointer::update_window_hovered(self, window);
        passes::draw::draw_window(self, window, canvas);
        passes::record::record_window(self, window, canvas);

        if self.settings().debug.recorder_overlay {
            passes::debug::recorder_overlay_window(self, window, canvas);
        }

        size
    }
}

impl World {
    #[track_caller]
    pub(crate) fn widget(&self, id: WidgetId) -> Option<WidgetRef<'_>> {
        match self.get_widget(id) {
            Some(widget) => Some(widget),
            None => {
                debug_panic!("invalid widget {id:?}");

                None
            }
        }
    }

    #[track_caller]
    pub(crate) fn widget_mut(&mut self, id: WidgetId) -> Option<WidgetMut<'_>> {
        match self.get_widget_mut(id) {
            Some(widget) => Some(widget),
            None => {
                debug_panic!("invalid widget {id:?}");

                None
            }
        }
    }
}

impl Builder for World {
    fn world(&self) -> &World {
        self
    }

    fn world_mut(&mut self) -> &mut World {
        self
    }
}
