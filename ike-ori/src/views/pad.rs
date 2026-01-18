use ike_core::{AnyWidgetId, Builder, Padding, WidgetId, widgets};
use ori::{Action, Event, View, ViewMarker};

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
impl<C, T, V> View<C, T> for Pad<V>
where
    C: Builder,
    V: View<C, T, Element: AnyWidgetId>,
{
    type Element = WidgetId<widgets::Pad>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let mut widget = widgets::Pad::new(cx, contents);

        widgets::Pad::set_padding(&mut widget, self.padding);

        (widget.id(), (contents, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
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

        if !cx.is_child(*element, *contents) {
            cx.set_child(*element, 0, *contents);
        }

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.padding != old.padding {
            widgets::Pad::set_padding(&mut widget, self.padding);
        }
    }

    fn teardown(&mut self, element: Self::Element, (contents, state): Self::State, cx: &mut C) {
        self.contents.teardown(contents, state, cx);
        cx.remove_widget(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let action = self.contents.event(contents, state, cx, data, event);

        if !cx.is_child(*element, *contents) {
            cx.set_child(*element, 0, *contents);
        }

        action
    }
}
