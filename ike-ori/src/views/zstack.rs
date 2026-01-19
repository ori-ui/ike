use ike_core::{Builder, WidgetId, widgets};
use ori::{Action, Elements, Event, Mut, View, ViewMarker, ViewSeq};

use crate::Context;

pub fn zstack<V>(contents: V) -> ZStack<V> {
    ZStack::new(contents)
}

pub struct ZStack<V> {
    contents: V,
}

impl<V> ZStack<V> {
    pub fn new(contents: V) -> Self {
        Self { contents }
    }
}

impl<V> ViewMarker for ZStack<V> {}
impl<T, V> View<Context, T> for ZStack<V>
where
    V: ViewSeq<Context, T, WidgetId>,
{
    type Element = WidgetId<widgets::ZStack>;
    type State = (Vec<WidgetId>, V::State);

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let element = widgets::ZStack::new(cx).id();

        let mut contents = Vec::new();
        let state = self.contents.seq_build(
            &mut ZStackElements::new(element, &mut contents),
            cx,
            data,
        );

        (element, (contents, state))
    }

    fn rebuild(
        self,
        element: Mut<Context, Self::Element>,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.seq_rebuild(
            &mut ZStackElements::new(*element, contents),
            state,
            cx,
            data,
        );
    }

    fn event(
        element: Mut<Context, Self::Element>,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        V::seq_event(
            &mut ZStackElements::new(*element, contents),
            state,
            cx,
            data,
            event,
        )
    }

    fn teardown(element: Self::Element, (mut contents, state): Self::State, cx: &mut Context) {
        V::seq_teardown(
            &mut ZStackElements::new(element, &mut contents),
            state,
            cx,
        );
    }
}

struct ZStackElements<'a> {
    widget:   WidgetId<widgets::ZStack>,
    elements: &'a mut Vec<WidgetId>,
    index:    usize,
}

impl<'a> ZStackElements<'a> {
    fn new(widget: WidgetId<widgets::ZStack>, elements: &'a mut Vec<WidgetId>) -> Self {
        Self {
            widget,
            elements,
            index: 0,
        }
    }
}

impl<'a> Elements<Context, WidgetId> for ZStackElements<'a> {
    fn next(&mut self, _cx: &mut Context) -> Option<&mut WidgetId> {
        let element = self.elements.get_mut(self.index)?;
        self.index += 1;
        Some(element)
    }

    fn insert(&mut self, cx: &mut Context, element: WidgetId) {
        cx.insert_child(self.widget, self.index, element);
        self.elements.insert(self.index, element);
        self.index += 1;
    }

    fn remove(&mut self, cx: &mut Context) -> Option<WidgetId> {
        cx.remove_child(self.widget, self.index);
        Some(self.elements.remove(self.index))
    }

    fn swap(&mut self, cx: &mut Context, offset: usize) {
        self.elements.swap(self.index, self.index + offset);
        cx.swap_children(
            self.widget,
            self.index,
            self.index + offset,
        );
    }
}
