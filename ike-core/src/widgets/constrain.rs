use std::time::Duration;

use crate::{
    AnyWidgetId, BuildCx, LayoutCx, Painter, Size, Space, Transition, Transitioned, UpdateCx,
    Widget, WidgetMut,
};

pub struct Constrain {
    min_size: Transitioned<Size>,
    max_size: Transitioned<Size>,
}

impl Constrain {
    pub fn new(cx: &mut impl BuildCx, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        let mut this = cx.insert(Self {
            min_size: Transitioned::new(Size::all(0.0), Transition::INSTANT),
            max_size: Transitioned::new(
                Size::all(f32::INFINITY),
                Transition::INSTANT,
            ),
        });
        this.add_child(child);
        this
    }

    pub fn set_child(this: &mut WidgetMut<Self>, child: impl AnyWidgetId) {
        let child = this.replace_child(0, child);
        this.cx.remove(child);
    }

    pub fn set_min_size(this: &mut WidgetMut<Self>, min_size: Size) {
        this.cx.request_layout();

        if this.widget.min_size.begin(min_size) {
            this.cx.request_animate();
        }
    }

    pub fn set_max_size(this: &mut WidgetMut<Self>, max_size: Size) {
        this.cx.request_layout();

        if this.widget.max_size.begin(max_size) {
            this.cx.request_animate();
        }
    }

    pub fn set_min_size_transition(this: &mut WidgetMut<Self>, transition: Transition) {
        this.widget.min_size.set_transition(transition);
    }

    pub fn set_max_size_transition(this: &mut WidgetMut<Self>, transition: Transition) {
        this.widget.max_size.set_transition(transition);
    }
}

impl Widget for Constrain {
    fn layout(
        &mut self,
        cx: &mut LayoutCx<'_>,
        painter: &mut dyn Painter,
        mut space: Space,
    ) -> Size {
        if self.min_size.width.is_finite() || space.max.width.is_finite() {
            space.min.width = space.min.width.max(self.min_size.width);
        }

        if self.min_size.height.is_finite() || space.max.height.is_finite() {
            space.min.height = space.min.height.max(self.min_size.height);
        }

        space.max = space.max.min(*self.max_size);
        space.min = space.min.min(space.max);

        cx.layout_child(0, painter, space)
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        cx.request_layout();

        if self.min_size.animate(dt) || self.max_size.animate(dt) {
            cx.request_animate();
            cx.set_pixel_perfect(false);
        } else {
            cx.set_pixel_perfect(true);
        }
    }
}
