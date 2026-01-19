use std::mem;

use ike_core::{
    AnyWidgetId, Axis, Builder, WidgetId,
    widgets::{self, Align, Justify},
};
use ori::{Action, Element, Elements, Event, Super, View, ViewMarker, ViewSeq};

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
    contents:   V,
    properties: Properties,
}

impl<V> Stack<V> {
    pub fn new(axis: Axis, contents: V) -> Self {
        Self {
            contents,
            properties: Properties {
                axis,
                justify: Justify::Start,
                align: Align::Center,
                gap: 0.0,
            },
        }
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.properties.justify = justify;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.properties.align = align;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.properties.gap = gap;
        self
    }
}

pub struct Properties {
    axis:    Axis,
    justify: Justify,
    align:   Align,
    gap:     f32,
}

impl<V> ViewMarker for Stack<V> {}
impl<T, V> View<Context, T> for Stack<V>
where
    V: ViewSeq<Context, T, Flex<WidgetId>>,
{
    type Element = WidgetId<widgets::Stack>;
    type State = (
        Properties,
        Vec<Flex<WidgetId>>,
        V::State,
    );

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let element = {
            let mut widget = widgets::Stack::new(cx);

            widgets::Stack::set_axis(&mut widget, self.properties.axis);
            widgets::Stack::set_justify(&mut widget, self.properties.justify);
            widgets::Stack::set_align(&mut widget, self.properties.align);
            widgets::Stack::set_gap(&mut widget, self.properties.gap);

            widget.id()
        };

        let mut elements = Vec::new();
        let states = self.contents.seq_build(
            &mut FlexElements::new(element, &mut elements),
            cx,
            data,
        );

        (
            element,
            (self.properties, elements, states),
        )
    }

    fn rebuild(
        self,
        element: &mut Self::Element,
        (properties, elements, states): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.seq_rebuild(
            &mut FlexElements::new(*element, elements),
            states,
            cx,
            data,
        );

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.properties.axis != properties.axis {
            widgets::Stack::set_axis(&mut widget, self.properties.axis);
        }

        if self.properties.justify != properties.justify {
            widgets::Stack::set_justify(&mut widget, self.properties.justify);
        }

        if self.properties.align != properties.align {
            widgets::Stack::set_align(&mut widget, self.properties.align);
        }

        if self.properties.gap != properties.gap {
            widgets::Stack::set_gap(&mut widget, self.properties.gap);
        }

        *properties = self.properties;
    }

    fn event(
        element: &mut Self::Element,
        (_properties, elements, states): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        V::seq_event(
            &mut FlexElements::new(*element, elements),
            states,
            cx,
            data,
            event,
        )
    }

    fn teardown(
        element: Self::Element,
        (_properties, mut elements, states): Self::State,
        cx: &mut Context,
    ) {
        V::seq_teardown(
            &mut FlexElements::new(element, &mut elements),
            states,
            cx,
        );

        cx.remove_widget(element);
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
impl<T, V> View<Context, T> for Flex<V>
where
    V: crate::View<T>,
{
    type Element = Flex<V::Element>;
    type State = V::State;

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.contents.build(cx, data);
        let element = Flex {
            contents: element,
            flex:     self.flex,
            tight:    self.tight,
        };

        (element, state)
    }

    fn rebuild(
        self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        (self.contents).rebuild(&mut element.contents, state, cx, data);
        element.flex = self.flex;
        element.tight = self.tight;
    }

    fn event(
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        V::event(
            &mut element.contents,
            state,
            cx,
            data,
            event,
        )
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut Context) {
        V::teardown(element.contents, state, cx);
    }
}

struct FlexElements<'a> {
    widget:   WidgetId<widgets::Stack>,
    elements: &'a mut Vec<Flex<WidgetId>>,
    index:    usize,
}

impl<'a> FlexElements<'a> {
    fn new(widget: WidgetId<widgets::Stack>, elements: &'a mut Vec<Flex<WidgetId>>) -> Self {
        Self {
            widget,
            elements,
            index: 0,
        }
    }
}

impl<'a> Elements<Context, Flex<WidgetId>> for FlexElements<'a> {
    fn next(&mut self, cx: &mut Context) -> Option<&mut Flex<WidgetId>> {
        let element = self.elements.get_mut(self.index)?;

        if let Ok(mut widget) = cx.get_widget_mut(self.widget) {
            widgets::Stack::set_flex(
                &mut widget,
                self.index,
                element.flex,
                element.tight,
            );
        }

        self.index += 1;
        Some(element)
    }

    fn insert(&mut self, cx: &mut Context, element: Flex<WidgetId>) {
        cx.insert_child(
            self.widget,
            self.index,
            element.contents,
        );

        if let Ok(mut widget) = cx.get_widget_mut(self.widget) {
            widgets::Stack::set_flex(
                &mut widget,
                self.index,
                element.flex,
                element.tight,
            );
        }

        self.elements.insert(self.index, element);
        self.index += 1;
    }

    fn remove(&mut self, cx: &mut Context) -> Option<Flex<WidgetId>> {
        cx.remove_child(self.widget, self.index);
        Some(self.elements.remove(self.index))
    }

    fn swap(&mut self, cx: &mut Context, offset: usize) {
        cx.swap_children(
            self.widget,
            self.index,
            self.index + offset,
        );

        self.elements.swap(self.index, self.index + offset);
    }
}

impl<V> Element<Context> for Flex<V> {
    type Mut<'a>
        = &'a mut Self
    where
        V: 'a;
}

impl<T> Super<Context, WidgetId<T>> for Flex<WidgetId>
where
    T: ?Sized,
{
    fn replace(cx: &mut Context, this: &mut Self, other: WidgetId<T>) -> Self {
        cx.replace_widget(this.contents, other);
        this.contents = other.upcast();
        this.flex = 0.0;
        this.tight = false;
        *this
    }

    fn upcast(_cx: &mut Context, sub: WidgetId<T>) -> Self {
        Flex {
            contents: sub.upcast(),
            flex:     0.0,
            tight:    false,
        }
    }

    fn downcast(self) -> WidgetId<T> {
        self.contents.downcast()
    }

    fn downcast_with<U>(this: &mut Self, f: impl FnOnce(&mut WidgetId<T>) -> U) -> U {
        let mut widget = this.downcast();
        let output = f(&mut widget);
        this.contents = widget.upcast();
        output
    }
}

impl<T> Super<Context, Flex<WidgetId<T>>> for Flex<WidgetId>
where
    T: ?Sized,
{
    fn replace(cx: &mut Context, this: &mut Flex<WidgetId>, other: Flex<WidgetId<T>>) -> Self {
        cx.replace_widget(this.contents, other.contents);
        mem::replace(this, Flex::upcast(cx, other))
    }

    fn upcast(_cx: &mut Context, sub: Flex<WidgetId<T>>) -> Self {
        Self {
            contents: sub.contents.upcast(),
            flex:     sub.flex,
            tight:    sub.tight,
        }
    }

    fn downcast(self) -> Flex<WidgetId<T>> {
        Flex {
            contents: self.contents.downcast(),
            flex:     self.flex,
            tight:    self.tight,
        }
    }

    fn downcast_with<U>(this: &mut Self, f: impl FnOnce(&mut Flex<WidgetId<T>>) -> U) -> U {
        let mut widget = this.downcast();
        let output = f(&mut widget);
        this.contents = widget.contents.upcast();
        this.flex = widget.flex;
        this.tight = widget.tight;
        output
    }
}
