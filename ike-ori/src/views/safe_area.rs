use ike_core::{Builder, Transition, WidgetId, widgets};
use ori::{Action, Event, View, ViewMarker};

use crate::Context;

pub fn safe_area<V>(contents: V) -> SafeArea<V> {
    SafeArea::new(contents)
}

pub struct SafeArea<V> {
    contents: V,

    transition: Transition,
}

impl<V> SafeArea<V> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            transition: Transition::ease(0.1),
        }
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.transition = transition;
        self
    }
}

impl<V> ViewMarker for SafeArea<V> {}
impl<T, V> View<Context, T> for SafeArea<V>
where
    V: crate::View<T>,
{
    type Element = WidgetId<widgets::SafeArea>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let mut widget = widgets::SafeArea::new(cx, contents);

        widgets::SafeArea::set_transition(&mut widget, self.transition);

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

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.transition != old.transition {
            widgets::SafeArea::set_transition(&mut widget, self.transition);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (contents, state): Self::State,
        cx: &mut Context,
    ) {
        self.contents.teardown(contents, state, cx);
        cx.remove_widget(element);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        self.contents.event(contents, state, cx, data, event)
    }
}
