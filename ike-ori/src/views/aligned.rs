use ike_core::{AnyWidgetId, Builder, WidgetId, widgets};
use ori::{Action, Event, View, ViewMarker};

use crate::Context;

pub fn align<V>(x: f32, y: f32, contents: V) -> Aligned<V> {
    Aligned { contents, x, y }
}

pub fn top_left<V>(contents: V) -> Aligned<V> {
    align(0.0, 0.0, contents)
}

pub fn top<V>(contents: V) -> Aligned<V> {
    align(0.5, 0.0, contents)
}

pub fn top_right<V>(contents: V) -> Aligned<V> {
    align(1.0, 0.0, contents)
}

pub fn left<V>(contents: V) -> Aligned<V> {
    align(0.0, 0.5, contents)
}

pub fn center<V>(contents: V) -> Aligned<V> {
    align(0.5, 0.5, contents)
}

pub fn right<V>(contents: V) -> Aligned<V> {
    align(1.0, 0.5, contents)
}

pub fn bottom_left<V>(contents: V) -> Aligned<V> {
    align(0.0, 1.0, contents)
}

pub fn bottom<V>(contents: V) -> Aligned<V> {
    align(0.5, 1.0, contents)
}

pub fn bottom_right<V>(contents: V) -> Aligned<V> {
    align(1.0, 1.0, contents)
}

pub struct Aligned<V> {
    contents: V,
    x:        f32,
    y:        f32,
}

impl<V> ViewMarker for Aligned<V> {}
impl<T, V> View<Context, T> for Aligned<V>
where
    V: crate::View<T>,
{
    type Element = WidgetId<widgets::Aligned>;
    type State = (V::Element, V::State);

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let element = widgets::Aligned::new(cx, self.x, self.y, contents.upcast());

        (element.id(), (contents, state))
    }

    fn rebuild(
        self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.rebuild(contents, state, cx, data);

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        widgets::Aligned::set_alignment(&mut widget, self.x, self.y);
    }

    fn event(
        _element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        V::event(contents, state, cx, data, event)
    }

    fn teardown(element: Self::Element, (contents, state): Self::State, cx: &mut Context) {
        V::teardown(contents, state, cx);
        cx.remove_widget(element);
    }
}
