use ike_core::{AnyWidgetId, Builder, Transition, WidgetId, widgets};
use ori::{Action, Event, View, ViewMarker};

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
impl<C, T, V> View<C, T> for SafeArea<V>
where
    C: Builder,
    V: View<C, T, Element: AnyWidgetId>,
{
    type Element = WidgetId<widgets::SafeArea>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let mut widget = widgets::SafeArea::new(cx, contents);

        widgets::SafeArea::set_transition(&mut widget, self.transition);

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

        let Some(mut widget) = cx.get_widget_mut(*element) else {
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
        cx: &mut C,
        data: &mut T,
    ) {
        self.contents.teardown(contents, state, cx, data);
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
