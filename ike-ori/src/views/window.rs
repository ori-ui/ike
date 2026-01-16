use ike_core::{AnyWidgetId, Builder, Color, KeyEvent, PointerEvent, Size, WindowId, WindowSizing};
use ori::{Action, Event, NoElement, Provider, Proxied, Proxy, View, ViewMarker};

use crate::Palette;

pub fn window<V, T>(contents: V) -> Window<V, T> {
    Window::new(contents)
}

#[allow(
    clippy::type_complexity,
    reason = "it's more clear to have the long types"
)]
pub struct Window<V, T> {
    contents:       V,
    title:          String,
    sizing:         WindowSizing,
    visible:        bool,
    decorated:      bool,
    color:          Option<Color>,
    key_filter:     Option<Box<dyn FnMut(&KeyEvent) -> bool>>,
    pointer_filter: Option<Box<dyn FnMut(&PointerEvent) -> bool>>,
    on_key:         Option<Box<dyn FnMut(&mut T, &KeyEvent) -> Action>>,
    on_pointer:     Option<Box<dyn FnMut(&mut T, &PointerEvent) -> Action>>,
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

    pub fn on_key<A>(
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

    pub fn on_pointer<A>(
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
}

enum WindowEvent {
    Key(WindowId, KeyEvent),
    Pointer(WindowId, PointerEvent),
}

impl<V, T> ViewMarker for Window<V, T> {}
impl<C, T, V> View<C, T> for Window<V, T>
where
    C: Builder + Provider + Proxied,
    V: View<C, T, Element: AnyWidgetId>,
{
    type Element = NoElement;
    type State = (WindowId, V::Element, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let palette = cx.get_or_default::<Palette>();
        let id = cx.world_mut().create_window(contents.upcast());

        cx.set_window_title(id, self.title.clone());
        cx.set_window_sizing(id, self.sizing);
        cx.set_window_visible(id, self.visible);
        cx.set_window_decorated(id, self.decorated);
        cx.set_window_color(
            id,
            self.color.unwrap_or(palette.background),
        );

        let proxy = cx.proxy();

        if let Some(window) = cx.world_mut().get_window_mut(id) {
            let proxy = proxy.cloned();
            let mut filter = self.key_filter.take();

            window.set_on_key(Box::new(move |event| {
                if let Some(ref mut filter) = filter
                    && filter(event)
                {
                    proxy.event(Event::new(
                        WindowEvent::Key(id, event.clone()),
                        None,
                    ));
                    true
                } else {
                    false
                }
            }));
        }

        if let Some(window) = cx.world_mut().get_window_mut(id) {
            let mut filter = self.pointer_filter.take();

            window.set_on_pointer(Box::new(move |event| {
                if let Some(ref mut filter) = filter
                    && filter(event)
                {
                    proxy.event(Event::new(
                        WindowEvent::Pointer(id, event.clone()),
                        None,
                    ));
                    true
                } else {
                    false
                }
            }));
        }

        (NoElement, (id, contents, state))
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        (id, contents, state): &mut Self::State,
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

        let palette = cx.get_or_default::<Palette>();

        if let Some(window) = cx.world().get_window(*id)
            && let Some(layer) = window.layers().first()
            && layer.widget() != contents.upcast()
        {
            cx.set_window_base_layer(*id, *contents);
        }

        if self.title != old.title {
            cx.set_window_title(*id, self.title.clone());
        }

        if self.sizing != old.sizing {
            cx.set_window_sizing(*id, self.sizing);
        }

        if self.visible != old.visible {
            cx.set_window_visible(*id, self.visible);
        }

        if self.decorated != old.decorated {
            cx.set_window_decorated(*id, self.decorated);
        }

        if self.color != old.color {
            cx.set_window_color(
                *id,
                self.color.unwrap_or(palette.background),
            );
        }
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        (window, _, _): Self::State,
        cx: &mut C,
        _data: &mut T,
    ) {
        cx.world_mut().remove_window(window);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        (id, contents, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        match event.get() {
            Some(WindowEvent::Key(target, event)) if target == id => {
                return if let Some(ref mut on_key) = self.on_key {
                    on_key(data, event)
                } else {
                    Action::new()
                };
            }

            Some(WindowEvent::Pointer(target, event)) if target == id => {
                return if let Some(ref mut on_pointer) = self.on_pointer {
                    on_pointer(data, event)
                } else {
                    Action::new()
                };
            }

            _ => {}
        }

        let action = self.contents.event(contents, state, cx, data, event);

        if let Some(window) = cx.world().get_window(*id)
            && let Some(layer) = window.layers().first()
            && layer.widget() != contents.upcast()
        {
            cx.set_window_base_layer(*id, *contents);
        }

        action
    }
}
