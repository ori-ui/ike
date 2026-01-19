use ike_core::{Builder, Padding, WidgetId, widgets};
use ori::{Action, Event, View, ViewMarker};

use crate::Context;

pub fn pad<V>(padding: impl Into<Padding>, contents: V) -> Pad<V> {
    Pad::new(padding, contents)
}

pub struct Pad<V> {
    contents: V,
    padding:  Padding,
}

impl<V> Pad<V> {
    pub fn new(padding: impl Into<Padding>, contents: V) -> Self {
        Self {
            contents,
            padding: padding.into(),
        }
    }
}

impl<V> ViewMarker for Pad<V> {}
impl<T, V> View<Context, T> for Pad<V>
where
    V: crate::View<T>,
{
    type Element = WidgetId<widgets::Pad>;
    type State = (Padding, V::Element, V::State);

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let mut widget = widgets::Pad::new(cx, contents);

        widgets::Pad::set_padding(&mut widget, self.padding);

        (
            widget.id(),
            (self.padding, contents, state),
        )
    }

    fn rebuild(
        self,
        element: &mut Self::Element,
        (padding, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.rebuild(contents, state, cx, data);

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.padding != *padding {
            *padding = self.padding;
            widgets::Pad::set_padding(&mut widget, self.padding);
        }
    }

    fn event(
        _element: &mut Self::Element,
        (_padding, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        V::event(contents, state, cx, data, event)
    }

    fn teardown(
        element: Self::Element,
        (_padding, contents, state): Self::State,
        cx: &mut Context,
    ) {
        V::teardown(contents, state, cx);
        cx.remove_widget(element);
    }
}
