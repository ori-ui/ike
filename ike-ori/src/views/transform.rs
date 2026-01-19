use ike_core::{Builder, Offset, Transition, WidgetId, widgets};
use ori::{Action, Event, Mut, View, ViewMarker};

use crate::Context;

pub fn transform<V>(contents: V) -> Transform<V> {
    Transform::new(contents)
}

pub struct Transform<V> {
    contents:   V,
    properties: Properties,
}

impl<V> Transform<V> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            properties: Properties {
                translation: Offset::ZERO,
                rotation:    0.0,
                scale_x:     1.0,
                scale_y:     1.0,

                translation_transition: Transition::INSTANT,
                rotation_transition:    Transition::INSTANT,
                scale_transition:       Transition::INSTANT,
            },
        }
    }

    pub fn translation(mut self, translation: Offset) -> Self {
        self.properties.translation = translation;
        self
    }

    pub fn rotation(mut self, rotation: f32) -> Self {
        self.properties.rotation = rotation;
        self
    }

    pub fn scale(mut self, scale_x: f32, scale_y: f32) -> Self {
        self.properties.scale_x = scale_x;
        self.properties.scale_y = scale_y;
        self
    }

    pub fn translation_transition(mut self, transition: Transition) -> Self {
        self.properties.translation_transition = transition;
        self
    }

    pub fn rotation_transition(mut self, transition: Transition) -> Self {
        self.properties.rotation_transition = transition;
        self
    }

    pub fn scale_transition(mut self, transition: Transition) -> Self {
        self.properties.scale_transition = transition;
        self
    }

    pub fn transition(self, transition: Transition) -> Self {
        self.translation_transition(transition)
            .rotation_transition(transition)
            .scale_transition(transition)
    }
}

pub struct Properties {
    translation: Offset,
    rotation:    f32,
    scale_x:     f32,
    scale_y:     f32,

    translation_transition: Transition,
    rotation_transition:    Transition,
    scale_transition:       Transition,
}

impl<V> ViewMarker for Transform<V> {}
impl<T, V> View<Context, T> for Transform<V>
where
    V: crate::View<T>,
{
    type Element = WidgetId<widgets::Transform>;
    type State = (Properties, V::Element, V::State);

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.contents.build(cx, data);

        let mut widget = widgets::Transform::new(cx, element);

        widgets::Transform::set_translation(&mut widget, self.properties.translation);
        widgets::Transform::set_rotation(&mut widget, self.properties.rotation);
        widgets::Transform::set_scale(
            &mut widget,
            self.properties.scale_x,
            self.properties.scale_y,
        );

        widgets::Transform::set_translation_transition(
            &mut widget,
            self.properties.translation_transition,
        );
        widgets::Transform::set_rotation_transition(
            &mut widget,
            self.properties.rotation_transition,
        );
        widgets::Transform::set_scale_transition(
            &mut widget,
            self.properties.scale_transition,
        );

        (
            widget.id(),
            (self.properties, element, state),
        )
    }

    fn rebuild(
        self,
        element: Mut<Context, Self::Element>,
        (properties, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.rebuild(contents, state, cx, data);

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.properties.translation != properties.translation {
            widgets::Transform::set_translation(&mut widget, self.properties.translation);
        }

        if self.properties.rotation != properties.rotation {
            widgets::Transform::set_rotation(&mut widget, self.properties.rotation);
        }

        if self.properties.scale_x != properties.scale_x
            || self.properties.scale_y != properties.scale_y
        {
            widgets::Transform::set_scale(
                &mut widget,
                self.properties.scale_x,
                self.properties.scale_y,
            );
        }

        widgets::Transform::set_translation_transition(
            &mut widget,
            self.properties.translation_transition,
        );
        widgets::Transform::set_rotation_transition(
            &mut widget,
            self.properties.rotation_transition,
        );
        widgets::Transform::set_scale_transition(
            &mut widget,
            self.properties.scale_transition,
        );

        *properties = self.properties;
    }

    fn event(
        _element: Mut<Context, Self::Element>,
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
