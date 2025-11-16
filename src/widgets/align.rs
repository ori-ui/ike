use crate::{BuildCx, LayoutCx, Point, Size, Space, Widget, WidgetId};

pub struct Aligned {
    x: f32,
    y: f32,
}

impl Aligned {
    pub fn new(
        cx: &mut BuildCx<'_>,
        x: f32,
        y: f32,
        content: WidgetId<impl Widget>,
    ) -> WidgetId<Self> {
        let this = cx.insert(Aligned { x, y });

        cx.add_child(&this, content);

        this
    }

    pub fn set_child(cx: &mut BuildCx<'_>, id: &WidgetId<Self>, child: WidgetId<impl Widget>) {
        cx.replace_child(id, 0, child);
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

        let position = Point::new(excess_width * self.x, excess_height * self.y);

        cx.place_child(0, position);

        size
    }
}
