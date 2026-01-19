use ike_core::{Builder, Size, Transition, WidgetId, widgets};
use ori::{Action, Event, View, ViewMarker};

use crate::Context;

pub fn constrain<V>(contents: V) -> Constrain<V> {
    Constrain::new(contents)
}

pub fn min_size<V>(min_size: impl Into<Size>, contents: V) -> Constrain<V> {
    Constrain::new(contents).min_size(min_size.into())
}

pub fn max_size<V>(max_size: impl Into<Size>, contents: V) -> Constrain<V> {
    Constrain::new(contents).max_size(max_size.into())
}

pub fn size<V>(size: impl Into<Size>, contents: V) -> Constrain<V> {
    Constrain::new(contents).size(size.into())
}

pub fn min_width<V>(min_width: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).min_width(min_width)
}

pub fn min_height<V>(min_height: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).min_height(min_height)
}

pub fn max_width<V>(max_width: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).max_width(max_width)
}

pub fn max_height<V>(max_height: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).max_height(max_height)
}

pub fn width<V>(width: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).width(width)
}

pub fn height<V>(height: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).height(height)
}

pub fn fill<V>(contents: V) -> Constrain<V> {
    Constrain::new(contents).size(Size::all(f32::INFINITY))
}

pub fn fill_width<V>(contents: V) -> Constrain<V> {
    Constrain::new(contents).width(f32::INFINITY)
}

pub fn fill_height<V>(contents: V) -> Constrain<V> {
    Constrain::new(contents).height(f32::INFINITY)
}

pub struct Constrain<V> {
    contents:   V,
    properties: Properties,
}

impl<V> Constrain<V> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            properties: Properties {
                min_size:            Size::all(0.0),
                max_size:            Size::all(f32::INFINITY),
                min_size_transition: Transition::INSTANT,
                max_size_transition: Transition::INSTANT,
            },
        }
    }

    pub fn min_size_transition(mut self, transition: Transition) -> Self {
        self.properties.min_size_transition = transition;
        self
    }

    pub fn max_size_transition(mut self, transition: Transition) -> Self {
        self.properties.max_size_transition = transition;
        self
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.properties.min_size_transition = transition;
        self.properties.max_size_transition = transition;
        self
    }

    pub fn min_size(mut self, min_size: Size) -> Self {
        self.properties.min_size = min_size;
        self
    }

    pub fn max_size(mut self, max_size: Size) -> Self {
        self.properties.max_size = max_size;
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.properties.min_size.width = min_width;
        self
    }

    pub fn min_height(mut self, min_height: f32) -> Self {
        self.properties.min_size.height = min_height;
        self
    }

    pub fn max_width(mut self, max_width: f32) -> Self {
        self.properties.max_size.width = max_width;
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.properties.max_size.height = max_height;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.properties.min_size = size;
        self.properties.max_size = size;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.properties.min_size.width = width;
        self.properties.max_size.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.properties.min_size.height = height;
        self.properties.max_size.height = height;
        self
    }
}

pub struct Properties {
    min_size:            Size,
    max_size:            Size,
    min_size_transition: Transition,
    max_size_transition: Transition,
}

impl<V> ViewMarker for Constrain<V> {}
impl<T, V> View<Context, T> for Constrain<V>
where
    V: crate::View<T>,
{
    type Element = WidgetId<widgets::Constrain>;
    type State = (Properties, V::Element, V::State);

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let mut widget = widgets::Constrain::new(cx, contents);

        widgets::Constrain::set_min_size(&mut widget, self.properties.min_size);
        widgets::Constrain::set_max_size(&mut widget, self.properties.max_size);
        widgets::Constrain::set_min_size_transition(
            &mut widget,
            self.properties.min_size_transition,
        );
        widgets::Constrain::set_max_size_transition(
            &mut widget,
            self.properties.max_size_transition,
        );

        (
            widget.id(),
            (self.properties, contents, state),
        )
    }

    fn rebuild(
        self,
        element: &mut Self::Element,
        (properties, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.rebuild(contents, state, cx, data);

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.properties.min_size != properties.min_size {
            widgets::Constrain::set_min_size(&mut widget, self.properties.min_size);
        }

        if self.properties.max_size != properties.max_size {
            widgets::Constrain::set_max_size(&mut widget, self.properties.max_size);
        }

        if self.properties.min_size_transition != properties.min_size_transition {
            widgets::Constrain::set_min_size_transition(
                &mut widget,
                self.properties.min_size_transition,
            );
        }

        if self.properties.max_size_transition != properties.max_size_transition {
            widgets::Constrain::set_max_size_transition(
                &mut widget,
                self.properties.max_size_transition,
            );
        }
    }

    fn event(
        _element: &mut Self::Element,
        (_properties, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        V::event(contents, state, cx, data, event)
    }

    fn teardown(
        element: Self::Element,
        (_properties, contents, state): Self::State,
        cx: &mut Context,
    ) {
        V::teardown(contents, state, cx);
        cx.remove_widget(element);
    }
}
