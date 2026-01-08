use crate::{
    AnyWidgetId, BorderWidth, BuildCx, Canvas, Color, CornerRadius, DrawCx, LayoutCx, Padding,
    Paint, Painter, Size, Space, Widget, WidgetMut,
};

pub struct Container {
    padding:          Padding,
    border_width:     BorderWidth,
    corner_radius:    CornerRadius,
    background_color: Color,
    border_color:     Color,
}

impl Container {
    pub fn new(cx: &mut impl BuildCx, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        cx.build_widget(Container {
            padding:          Padding::all(8.0),
            border_width:     BorderWidth::all(1.0),
            corner_radius:    CornerRadius::all(8.0),
            background_color: Color::rgb(0.9, 0.9, 0.9),
            border_color:     Color::BLACK,
        })
        .with_child(child)
        .finish()
    }

    pub fn set_padding(this: &mut WidgetMut<Self>, padding: Padding) {
        this.widget.padding = padding;
        this.cx.request_layout();
    }

    pub fn set_border_width(this: &mut WidgetMut<Self>, border_width: BorderWidth) {
        this.widget.border_width = border_width;
        this.cx.request_layout();
    }

    pub fn set_corner_radius(this: &mut WidgetMut<Self>, corner_radius: CornerRadius) {
        this.widget.corner_radius = corner_radius;
        this.cx.request_draw();
    }

    pub fn set_background_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.background_color = color;
        this.cx.request_draw();
    }

    pub fn set_border_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.border_color = color;
        this.cx.request_draw();
    }
}

impl Widget for Container {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size {
        let space = space.shrink(self.padding.size() + self.border_width.size());
        let size = cx.layout_child(0, painter, space);

        cx.place_child(
            0,
            self.padding.offset() + self.border_width.offset(),
        );

        size + self.padding.size() + self.border_width.size()
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_rect(
            cx.rect(),
            self.corner_radius,
            &Paint::from(self.background_color),
        );

        canvas.draw_border(
            cx.rect(),
            self.border_width,
            self.corner_radius,
            &Paint::from(self.border_color),
        );
    }
}
