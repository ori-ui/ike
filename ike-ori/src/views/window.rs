use ike_core::{
    AnyWidgetId, Builder, Color, Key, KeyEvent, KeyPressEvent, Modifiers, PointerButton,
    PointerButtonEvent, PointerEvent, Size, WindowId, WindowSizing,
};
use ori::{Action, Event, NoElement, Provider, Proxied, Proxy, View, ViewId, ViewMarker};

use crate::Palette;

pub fn window<V, T>(contents: V) -> Window<V, T> {
    Window::new(contents)
}

type KeyFilter = Box<dyn FnMut(&KeyEvent) -> bool>;
type PointerFilter = Box<dyn FnMut(&PointerEvent) -> bool>;
type OnKeyEvent<T> = Box<dyn FnMut(&mut T, &KeyEvent) -> Action>;
type OnPointerEvent<T> = Box<dyn FnMut(&mut T, &PointerEvent) -> Action>;
type OnKey<T> = Box<dyn FnMut(&mut T, &KeyPressEvent) -> Action>;
type OnPointer<T> = Box<dyn FnMut(&mut T, &PointerButtonEvent) -> Action>;

pub struct Window<V, T> {
    contents:       V,
    title:          String,
    sizing:         WindowSizing,
    visible:        bool,
    decorated:      bool,
    color:          Option<Color>,
    key_filter:     Option<KeyFilter>,
    pointer_filter: Option<PointerFilter>,
    on_key:         Option<OnKeyEvent<T>>,
    on_pointer:     Option<OnPointerEvent<T>>,
    on_keys:        Vec<(Key, Modifiers, OnKey<T>)>,
    on_pointers:    Vec<(PointerButton, OnPointer<T>)>,
}

impl<V, T> Window<V, T> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            title: String::new(),
            sizing: WindowSizing::Resizable {
                default_size: Size::new(800.0, 600.0),
                min_size:     Size::all(0.0),
                max_size:     Size::all(f32::INFINITY),
            },
            visible: true,
            decorated: true,
            color: None,
            key_filter: None,
            pointer_filter: None,
            on_key: None,
            on_pointer: None,
            on_keys: Vec::new(),
            on_pointers: Vec::new(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn sizing(mut self, sizing: WindowSizing) -> Self {
        self.sizing = sizing;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.sizing = WindowSizing::Resizable {
            default_size: Size::new(width, height),
            min_size:     Size::new(width, height),
            max_size:     Size::new(width, height),
        };

        self
    }

    pub fn fit_content(mut self) -> Self {
        self.sizing = WindowSizing::FitContent;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    /// Register an [`Action`] callback for when a `key` is pressed with `modifiers` held.
    ///
    /// This is useful for registering keyboard shortcuts. Multiple keys can be set for one window.
    pub fn on_key<A>(
        mut self,
        key: impl Into<Key>,
        modifiers: Modifiers,
        mut on_key: impl FnMut(&mut T, &KeyPressEvent) -> A + 'static,
    ) -> Self
    where
        A: Into<Action>,
    {
        self.on_keys.push((
            key.into(),
            modifiers,
            Box::new(move |data, event| on_key(data, event).into()),
        ));
        self
    }

    /// Register an [`Action`] callback for when a pointer `button` is pressed.
    ///
    /// This is useful for registering shortcuts, like pressing [`PointerButton::Backward`] or
    /// [`PointerButton::Forward`]. Multiple pointer buttons can be set for one window.
    pub fn on_pointer_button<A>(
        mut self,
        button: PointerButton,
        mut on_button: impl FnMut(&mut T, &PointerButtonEvent) -> A + 'static,
    ) -> Self
    where
        A: Into<Action>,
    {
        self.on_pointers.push((
            button,
            Box::new(move |data, event| on_button(data, event).into()),
        ));
        self
    }

    /// Regster an [`Action`] callback for when a [`KeyEvent`] is emitted.
    ///
    /// `filter` allows for choosing which events to handle, this matters for platforms like
    /// `android` where the operating system needs to know whether a key press like _audio down_ or
    /// _back_ is handled by the application.
    pub fn on_key_event<A>(
        mut self,
        filter: impl FnMut(&KeyEvent) -> bool + 'static,
        mut on_key: impl FnMut(&mut T, &KeyEvent) -> A + 'static,
    ) -> Self
    where
        A: Into<Action>,
    {
        self.key_filter = Some(Box::new(filter));
        self.on_key = Some(Box::new(move |data, event| {
            on_key(data, event).into()
        }));
        self
    }

    /// Regster an [`Action`] callback for when a [`PointerEvent`] is emitted.
    pub fn on_pointer_event<A>(
        mut self,
        filter: impl FnMut(&PointerEvent) -> bool + 'static,
        mut on_pointer: impl FnMut(&mut T, &PointerEvent) -> A + 'static,
    ) -> Self
    where
        A: Into<Action>,
    {
        self.pointer_filter = Some(Box::new(filter));
        self.on_pointer = Some(Box::new(move |data, event| {
            on_pointer(data, event).into()
        }));
        self
    }

    fn register_on_key(
        &mut self,
        cx: &mut (impl Builder + Proxied),
        window_id: WindowId,
        view_id: ViewId,
    ) {
        let proxy = cx.proxy();
        if let Some(window) = cx.world_mut().get_window_mut(window_id) {
            let mut filter = self.key_filter.take();
            let keys: Vec<_> = self
                .on_keys
                .iter()
                .map(|(key, mods, _)| (key.clone(), *mods))
                .collect();

            window.set_on_key(Box::new(move |event| {
                let is_key = match event {
                    KeyEvent::Down(press) => keys.iter().any(|(key, mods)| {
                        let is_key = press.key == *key;
                        let is_mods = press.modifiers == *mods;

                        is_key && is_mods
                    }),
                    _ => false,
                };

                if is_key || filter.as_mut().is_some_and(|filter| filter(event)) {
                    proxy.event(Event::new(
                        WindowEvent::Key(event.clone()),
                        view_id,
                    ));

                    true
                } else {
                    false
                }
            }));
        }

        let proxy = cx.proxy();
        if let Some(window) = cx.world_mut().get_window_mut(window_id) {
            let buttons: Vec<_> = self.on_pointers.iter().map(|(btn, _)| *btn).collect();
            let mut filter = self.pointer_filter.take();

            window.set_on_pointer(Box::new(move |event| {
                let is_button = match event {
                    PointerEvent::Down(press) => buttons.contains(&press.button),
                    _ => false,
                };

                if is_button || filter.as_mut().is_some_and(|filter| filter(event)) {
                    proxy.event(Event::new(
                        WindowEvent::Pointer(event.clone()),
                        view_id,
                    ));
                    true
                } else {
                    false
                }
            }));
        }
    }
}

enum WindowEvent {
    Key(KeyEvent),
    Pointer(PointerEvent),
}

impl<V, T> ViewMarker for Window<V, T> {}
impl<C, T, V> View<C, T> for Window<V, T>
where
    C: Builder + Provider + Proxied,
    V: View<C, T, Element: AnyWidgetId>,
{
    type Element = NoElement;
    type State = (WindowId, ViewId, V::Element, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let palette = cx.get_or_default::<Palette>();
        let window_id = cx.world_mut().create_window(contents.upcast());

        let color = self.color.unwrap_or(palette.background);

        cx.set_window_title(window_id, self.title.clone());
        cx.set_window_sizing(window_id, self.sizing);
        cx.set_window_visible(window_id, self.visible);
        cx.set_window_decorated(window_id, self.decorated);
        cx.set_window_color(window_id, color);

        let view_id = ViewId::next();
        self.register_on_key(cx, window_id, view_id);
        (
            NoElement,
            (window_id, view_id, contents, state),
        )
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        (window_id, view_id, contents, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.rebuild(
            contents,
            state,
            cx,
            data,
            &mut old.contents,
        );

        self.register_on_key(cx, *window_id, *view_id);

        let palette = cx.get_or_default::<Palette>();

        if let Some(window) = cx.world().get_window(*window_id)
            && let Some(layer) = window.layers().first()
            && layer.widget() != contents.upcast()
        {
            cx.set_window_base_layer(*window_id, *contents);
        }

        if self.title != old.title {
            cx.set_window_title(*window_id, self.title.clone());
        }

        if self.sizing != old.sizing {
            cx.set_window_sizing(*window_id, self.sizing);
        }

        if self.visible != old.visible {
            cx.set_window_visible(*window_id, self.visible);
        }

        if self.decorated != old.decorated {
            cx.set_window_decorated(*window_id, self.decorated);
        }

        if self.color != old.color {
            let color = self.color.unwrap_or(palette.background);
            cx.set_window_color(*window_id, color);
        }
    }

    fn teardown(&mut self, _element: Self::Element, (window, _, _, _): Self::State, cx: &mut C) {
        cx.world_mut().remove_window(window);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        (window_id, view_id, contents, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        match event.take_targeted(*view_id) {
            Some(WindowEvent::Key(event)) => {
                for (key, mods, on_key) in &mut self.on_keys {
                    if let KeyEvent::Down(ref event) = event
                        && event.key == *key
                        && event.modifiers == *mods
                    {
                        return on_key(data, event);
                    }
                }

                return match self.on_key {
                    Some(ref mut on_key) => on_key(data, &event),
                    None => Action::new(),
                };
            }

            Some(WindowEvent::Pointer(event)) => {
                for (button, on_button) in &mut self.on_pointers {
                    if let PointerEvent::Down(ref event) = event
                        && event.button == *button
                    {
                        return on_button(data, event);
                    }
                }

                return match self.on_pointer {
                    Some(ref mut on_pointer) => on_pointer(data, &event),
                    None => Action::new(),
                };
            }

            _ => {}
        }

        let action = self.contents.event(contents, state, cx, data, event);

        if let Some(window) = cx.world().get_window(*window_id)
            && let Some(layer) = window.layers().first()
            && layer.widget() != contents.upcast()
        {
            cx.set_window_base_layer(*window_id, *contents);
        }

        action
    }
}
