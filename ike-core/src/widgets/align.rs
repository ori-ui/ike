use crate::{AnyWidgetId, Builder, LayoutCx, Offset, Size, Space, Widget, WidgetMut};

pub struct Aligned {
    x: f32,
    y: f32,
}

impl Aligned {
    pub fn new(
        cx: &mut impl Builder,
        x: f32,
        y: f32,
        contents: impl AnyWidgetId,
    ) -> WidgetMut<'_, Self> {
        cx.build_widget(Aligned { x, y })
            .with_child(contents)
            .finish()
    }

    pub fn set_alignment(this: &mut WidgetMut<Self>, x: f32, y: f32) {
        this.widget.x = x;
        this.widget.y = y;
        this.cx.request_layout();
    }
}

impl Widget for Aligned {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let child_size = {
            let mut space = space;
            space.min = Size::ZERO;

            cx.layout_nth_child(0, space)
        };

        let mut size = space.constrain(child_size);

        if space.max.width.is_finite() {
            size.width = space.max.width;
        }

        if space.max.height.is_finite() {
            size.height = space.max.height;
        }

        let excess_width = size.width - child_size.width;
        let excess_height = size.height - child_size.height;

        let position = Offset::new(
            excess_width * self.x,
            excess_height * self.y,
        );

        cx.place_nth_child(0, position);

        size
    }
}
