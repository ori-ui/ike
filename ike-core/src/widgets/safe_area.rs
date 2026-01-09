use std::time::Duration;

use crate::{
    AnyWidgetId, Builder, LayoutCx, Padding, Size, Space, Transition, Transitioned, Update,
    UpdateCx, Widget, WidgetMut,
};

pub struct SafeArea {
    insets: Transitioned<Padding>,
}

impl SafeArea {
    pub fn new(cx: &mut impl Builder, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        cx.build_widget(SafeArea {
            insets: Transitioned::new(Padding::all(0.0), Transition::INSTANT),
        })
        .with_child(child)
        .finish()
    }

    pub fn set_transition(this: &mut WidgetMut<Self>, transition: Transition) {
        this.widget.insets.set_transition(transition);
    }
}

impl Widget for SafeArea {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let space = self.insets.layout_down(cx, space);
        let size = cx.layout_child(0, space);
        cx.place_child(0, self.insets.offset());
        self.insets.layout_up(cx, size)
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        cx.request_layout();

        if self.insets.animate(dt) {
            cx.request_animate();
            cx.set_subpixel(true);
        } else {
            cx.set_subpixel(false);
        }
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        if let Update::WindowInset(insets) = update {
            cx.request_layout();

            if self.insets.begin(insets) {
                cx.request_animate();
            }
        }
    }
}
