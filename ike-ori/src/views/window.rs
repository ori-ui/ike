use ike_core::{AnyWidgetId, Builder, Color, Size, WindowId, WindowSizing};
use ori::{Action, Event, NoElement, Providable, View, ViewMarker};

use crate::Palette;

pub fn window<V>(contents: V) -> Window<V> {
    Window::new(contents)
}

pub struct Window<V> {
    contents:  V,
    title:     String,
    sizing:    WindowSizing,
    visible:   bool,
    decorated: bool,
    color:     Option<Color>,
}

impl<V> Window<V> {
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
}

impl<V> ViewMarker for Window<V> {}
impl<C, T, V> View<C, T> for Window<V>
where
    C: Builder + Providable,
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
