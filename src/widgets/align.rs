use crate::{AnyWidgetId, BuildCx, LayoutCx, Offset, Size, Space, Widget, WidgetMut};

pub struct Aligned {
    x: f32,
    y: f32,
}

impl Aligned {
    pub fn new(
        cx: &mut impl BuildCx,
        x: f32,
        y: f32,
        content: impl AnyWidgetId,
    ) -> WidgetMut<'_, Self> {
        let mut this = cx.insert(Aligned { x, y });
        this.add_child(content);
        this
    }

    pub fn set_child(this: &mut WidgetMut<Self>, child: impl AnyWidgetId) {
        this.replace_child(0, child);
    }

    pub fn set_alignment(this: &mut WidgetMut<Self>, x: f32, y: f32) {
        this.x = x;
        this.y = y;
        this.request_layout();
    }
}

impl Widget for Aligned {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let child_size = cx.layout_child(0, space);

        let mut size = child_size;

        if space.max.width.is_finite() {
            size.width = space.max.width;
        }

        if space.max.height.is_finite() {
            size.height = space.max.height;
        }

        let excess_width = size.width - child_size.width;
        let excess_height = size.height - child_size.height;

        let position = Offset::new(excess_width * self.x, excess_height * self.y);

        cx.place_child(0, position);

        size
    }
}
