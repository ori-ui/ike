use crate::{
    AnyWidgetId, BorderWidth, BuildCx, Canvas, Color, CornerRadius, DrawCx, LayoutCx, Padding,
    Paint, Size, Space, Widget, WidgetId,
};

pub struct Container {
    padding: Padding,
    border_width: BorderWidth,
    corner_radius: CornerRadius,
    background: Color,
    border_color: Color,
}

impl Container {
    pub fn new(cx: &mut impl BuildCx, child: impl AnyWidgetId) -> WidgetId<Self> {
        let this = cx.insert(Container {
            padding: Padding::all(8.0),
            border_width: BorderWidth::all(1.0),
            corner_radius: CornerRadius::all(8.0),
            background: Color::rgb(0.9, 0.9, 0.9),
            border_color: Color::BLACK,
        });

        cx.add_child(this, child);

        this
    }

    pub fn set_child(cx: &mut impl BuildCx, id: WidgetId<Self>, child: impl AnyWidgetId) {
        cx.replace_child(id, 0, child);
    }

    pub fn set_padding(cx: &mut impl BuildCx, id: WidgetId<Self>, padding: Padding) {
        cx.get_mut(id).padding = padding;
        cx.request_layout(id);
    }

    pub fn set_border_width(cx: &mut impl BuildCx, id: WidgetId<Self>, border_width: BorderWidth) {
        cx.get_mut(id).border_width = border_width;
        cx.request_layout(id);
    }

    pub fn set_corner_radius(
        cx: &mut impl BuildCx,
        id: WidgetId<Self>,
        corner_radius: CornerRadius,
    ) {
        cx.get_mut(id).corner_radius = corner_radius;
        cx.request_draw(id);
    }

    pub fn set_background(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        cx.get_mut(id).background = color;
        cx.request_draw(id);
    }

    pub fn set_border_color(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        cx.get_mut(id).border_color = color;
        cx.request_draw(id);
    }
}

impl Widget for Container {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let space = space.shrink(self.padding.size());
        let size = cx.layout_child(0, space);

        cx.place_child(0, self.padding.offset());

        size + self.padding.size()
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_rect(cx.rect(), self.corner_radius, &Paint::from(self.background));
        canvas.draw_border(
            cx.rect(),
            self.border_width,
            self.corner_radius,
            &Paint::from(self.border_color),
        );
    }

    fn accepts_pointer() -> bool {
        false
    }
}
