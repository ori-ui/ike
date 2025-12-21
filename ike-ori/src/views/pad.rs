use ike_core::{BuildCx, Padding, WidgetId, widgets};

use crate::{Context, View};

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

impl<V> ori::ViewMarker for Pad<V> {}
impl<T, V> ori::View<Context, T> for Pad<V>
where
    V: View<T>,
{
    type Element = WidgetId<widgets::Pad>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let mut widget = widgets::Pad::new(cx, contents);

        widgets::Pad::set_padding(&mut widget, self.padding);

        (widget.id(), (contents, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut Context,
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

        let Some(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if !widget.cx.is_child(*contents) {
            widgets::Pad::set_child(&mut widget, *contents);
        }

        if self.padding != old.padding {
            widgets::Pad::set_padding(&mut widget, self.padding);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (contents, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.teardown(contents, state, cx, data);
        cx.remove_widget(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.contents.event(contents, state, cx, data, event);

        if let Some(mut widget) = cx.get_widget_mut(*element)
            && !widget.cx.is_child(*contents)
        {
            widgets::Pad::set_child(&mut widget, *contents);
        };

        action
    }
}
