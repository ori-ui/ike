use std::time::Duration;

use crate::{
    Affine, AnyWidgetId, Builder, ComposeCx, LayoutCx, Offset, Point, Size, Space, Transition,
    Transitioned, UpdateCx, Widget, WidgetMut,
};

pub struct Transform {
    translation: Transitioned<Offset>,
    rotation:    Transitioned<f32>,
    scale_x:     Transitioned<f32>,
    scale_y:     Transitioned<f32>,
}

impl Transform {
    pub fn new(cx: &mut impl Builder, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        cx.build_widget(Self {
            translation: Transitioned::new(Offset::ZERO, Transition::INSTANT),
            rotation:    Transitioned::new(0.0, Transition::INSTANT),
            scale_x:     Transitioned::new(1.0, Transition::INSTANT),
            scale_y:     Transitioned::new(1.0, Transition::INSTANT),
        })
        .with_child(child)
        .finish()
    }

    pub fn set_translation(this: &mut WidgetMut<'_, Self>, translation: Offset) {
        this.cx.request_compose();

        if this.widget.translation.begin(translation) {
            this.cx.request_animate();
        }
    }

    pub fn set_rotation(this: &mut WidgetMut<'_, Self>, rotation: f32) {
        this.cx.request_compose();

        if this.widget.rotation.begin(rotation) {
            this.cx.request_animate();
        }
    }

    pub fn set_scale(this: &mut WidgetMut<'_, Self>, scale_x: f32, scale_y: f32) {
        this.cx.request_compose();

        if this.widget.scale_x.begin(scale_x) | this.widget.scale_y.begin(scale_y) {
            this.cx.request_animate();
        }
    }

    pub fn set_translation_transition(this: &mut WidgetMut<'_, Self>, transition: Transition) {
        this.widget.translation.set_transition(transition);
    }

    pub fn set_rotation_transition(this: &mut WidgetMut<'_, Self>, transition: Transition) {
        this.widget.rotation.set_transition(transition);
    }

    pub fn set_scale_transition(this: &mut WidgetMut<'_, Self>, transition: Transition) {
        this.widget.scale_x.set_transition(transition);
        this.widget.scale_y.set_transition(transition);
    }
}

impl Widget for Transform {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        cx.layout_nth_child(0, space)
    }

    fn compose(&mut self, cx: &mut ComposeCx<'_>) {
        let transform = Affine::translate(Point::ORIGIN - cx.rect().center())
            * Affine::scale(*self.scale_x, *self.scale_y)
            * Affine::rotate(*self.rotation)
            * Affine::translate(*self.translation)
            * Affine::translate(cx.rect().center() - Point::ORIGIN);

        cx.place_nth_child(0, transform);
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        cx.request_compose();

        if self.translation.animate(dt)
            | self.rotation.animate(dt)
            | self.scale_x.animate(dt)
            | self.scale_y.animate(dt)
        {
            cx.request_animate();
            cx.set_subpixel(true);
        } else {
            cx.set_subpixel(false);
        }
    }
}
