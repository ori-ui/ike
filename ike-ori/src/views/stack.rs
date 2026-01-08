use ike_core::{
    AnyWidgetId, Axis, BuildCx, WidgetId,
    widgets::{self, Align, Justify},
};
use ori::{ElementSeq, View, ViewMarker};

use crate::context::Context;

pub fn stack<V>(axis: Axis, contents: V) -> Stack<V> {
    Stack::new(axis, contents)
}

pub fn hstack<V>(contents: V) -> Stack<V> {
    stack(Axis::Horizontal, contents)
}

pub fn vstack<V>(contents: V) -> Stack<V> {
    stack(Axis::Vertical, contents)
}

pub struct Stack<V> {
    contents: V,
    axis:     Axis,
    justify:  Justify,
    align:    Align,
    gap:      f32,
}

impl<V> Stack<V> {
    pub fn new(axis: Axis, contents: V) -> Self {
        Self {
            contents,
            axis,
            justify: Justify::Start,
            align: Align::Center,
            gap: 0.0,
        }
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

impl<V> ViewMarker for Stack<V> {}
impl<C, T, V> View<C, T> for Stack<V>
where
    C: BuildCx,
    V: ori::ViewSeq<C, T, Flex<WidgetId>>,
{
    type Element = WidgetId<widgets::Stack>;
    type State = (V::Elements, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (children, states) = self.contents.seq_build(cx, data);

        let widget = {
            let mut widget = widgets::Stack::new(cx);

            widgets::Stack::set_axis(&mut widget, self.axis);
            widgets::Stack::set_justify(&mut widget, self.justify);
            widgets::Stack::set_align(&mut widget, self.align);
            widgets::Stack::set_gap(&mut widget, self.gap);

            widget.id()
        };

        for child in children.iter() {
            cx.add_child(widget, child.contents);
        }

        update_flex(cx, widget, &children);

        (widget, (children, states))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (children, states): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.seq_rebuild(
            children,
            states,
            cx,
            data,
            &mut old.contents,
        );

        update_children(cx, *element, children);

        let Some(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.axis != old.axis {
            widgets::Stack::set_axis(&mut widget, self.axis);
        }

        if self.justify != old.justify {
            widgets::Stack::set_justify(&mut widget, self.justify);
        }

        if self.align != old.align {
            widgets::Stack::set_align(&mut widget, self.align);
        }

        if self.gap != old.gap {
            widgets::Stack::set_gap(&mut widget, self.gap);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (children, states): Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        self.contents.seq_teardown(children, states, cx, data);
        cx.remove_widget(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (children, states): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.contents.seq_event(children, states, cx, data, event);

        update_children(cx, *element, children);

        action
    }
}

fn update_children(
    cx: &mut impl BuildCx,
    widget: WidgetId<widgets::Stack>,
    children: &mut impl ElementSeq<Flex<WidgetId>>,
) {
    for child in children.iter() {
        if !cx.is_parent(widget, child.contents) {
            cx.add_child(widget, child.contents);
        }
    }

    for (i, child) in children.iter().enumerate() {
        let children = cx.children(widget);

        if children[i] != child.contents {
            let index = children
                .iter()
                .position(|c| *c == child.contents)
                .expect("child should not have been removed");

            cx.swap_children(widget, i, index);
        }
    }

    let cur_len = cx.children(widget).len();
    let new_len = children.count();

    for _ in new_len..cur_len {
        let child = *cx
            .children(widget)
            .last()
            .expect("there should be at least one child");

        cx.remove_widget(child);
    }

    update_flex(cx, widget, children);
}

fn update_flex(
    cx: &mut impl BuildCx,
    widget: WidgetId<widgets::Stack>,
    children: &impl ElementSeq<Flex<WidgetId>>,
) {
    if let Some(mut widget) = cx.get_widget_mut(widget) {
        for (i, child) in children.iter().enumerate() {
            let (flex, tight) = widget.widget.get_flex(i);

            if flex != child.flex || tight != child.tight {
                widgets::Stack::set_flex(&mut widget, i, child.flex, child.tight);
            }
        }
    }
}

pub fn flex<V>(contents: V) -> Flex<V> {
    Flex::new(true, contents)
}

pub fn expand<V>(contents: V) -> Flex<V> {
    Flex::new(false, contents)
}

#[derive(Clone, Copy)]
pub struct Flex<V> {
    contents: V,
    flex:     f32,
    tight:    bool,
}

impl<V> Flex<V> {
    pub fn new(tight: bool, contents: V) -> Self {
        Self {
            contents,
            flex: 1.0,
            tight,
        }
    }

    pub fn amount(mut self, amount: f32) -> Self {
        self.flex = amount;
        self
    }
}

impl<V> ViewMarker for Flex<V> {}
impl<C, T, V> View<C, T> for Flex<V>
where
    V: View<C, T>,
{
    type Element = Flex<V::Element>;
    type State = V::State;

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.contents.build(cx, data);
        let element = Flex {
            contents: element,
            flex:     self.flex,
            tight:    self.tight,
        };

        (element, state)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.rebuild(
            &mut element.contents,
            state,
            cx,
            data,
            &mut old.contents,
        );
    }

    fn teardown(&mut self, element: Self::Element, state: Self::State, cx: &mut C, data: &mut T) {
        self.contents.teardown(element.contents, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        self.contents.event(
            &mut element.contents,
            state,
            cx,
            data,
            event,
        )
    }
}

impl<S> ori::Super<Context, S> for Flex<WidgetId>
where
    S: AnyWidgetId,
{
    fn upcast(_cx: &mut Context, sub: S) -> Self {
        Flex {
            contents: sub.upcast(),
            flex:     0.0,
            tight:    false,
        }
    }

    fn downcast(self) -> S {
        self.contents.downcast()
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut S) -> T) -> T {
        self.contents.downcast_with(f)
    }
}

impl<S> ori::Super<Context, Flex<S>> for Flex<WidgetId>
where
    S: AnyWidgetId,
{
    fn upcast(_cx: &mut Context, sub: Flex<S>) -> Self {
        Self {
            contents: sub.contents.upcast(),
            flex:     sub.flex,
            tight:    sub.tight,
        }
    }

    fn downcast(self) -> Flex<S> {
        Flex {
            contents: self.contents.downcast(),
            flex:     self.flex,
            tight:    self.tight,
        }
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut Flex<S>) -> T) -> T {
        let mut flex: Flex<S> = self.downcast();
        let output = f(&mut flex);
        self.contents = flex.contents.upcast();
        self.flex = flex.flex;
        output
    }
}
