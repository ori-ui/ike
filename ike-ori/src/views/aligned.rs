use ike_core::{AnyWidgetId, BuildCx, WidgetId, widgets};

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

impl<V> ori::ViewMarker for Aligned<V> {}
impl<C, T, V> ori::View<C, T> for Aligned<V>
where
    C: BuildCx,
    V: ori::View<C, T, Element: AnyWidgetId>,
{
    type Element = WidgetId<widgets::Aligned>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let element = widgets::Aligned::new(cx, self.x, self.y, contents.upcast());

        (element.id(), (contents, state))
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

        if self.x != old.x || self.y != old.y {
            widgets::Aligned::set_alignment(&mut widget, self.x, self.y);
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
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.contents.event(contents, state, cx, data, event);

        if !cx.is_child(*element, *contents) {
            cx.set_child(*element, 0, *contents);
        }

        action
    }
}
