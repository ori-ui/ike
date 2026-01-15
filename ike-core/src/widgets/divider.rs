use crate::{
    Axis, Builder, Canvas, Color, CornerRadius, DrawCx, LayoutCx, Paint, Rect, Size, Space, Widget,
    WidgetMut,
};

pub struct Divider {
    axis:          Axis,
    thickness:     f32,
    inset:         f32,
    padding:       f32,
    corner_radius: CornerRadius,
    color:         Color,
}

impl Divider {
    pub fn new(cx: &mut impl Builder) -> WidgetMut<'_, Self> {
        cx.build_widget(Self {
            axis:          Axis::Horizontal,
            thickness:     1.0,
            inset:         8.0,
            padding:       4.0,
            corner_radius: CornerRadius::all(0.0),
            color:         Color::BLACK,
        })
        .finish()
    }

    pub fn set_axis(this: &mut WidgetMut<Self>, axis: Axis) {
        this.widget.axis = axis;
        this.cx.request_layout();
    }

    pub fn set_thickness(this: &mut WidgetMut<Self>, thickness: f32) {
        this.widget.thickness = thickness;
        this.cx.request_layout();
    }

    pub fn set_inset(this: &mut WidgetMut<Self>, inset: f32) {
        this.widget.inset = inset;
        this.cx.request_layout();
    }

    pub fn set_padding(this: &mut WidgetMut<Self>, padding: f32) {
        this.widget.padding = padding;
        this.cx.request_layout();
    }

    pub fn set_corner_radius(this: &mut WidgetMut<Self>, corner_radius: CornerRadius) {
        this.widget.corner_radius = corner_radius;
        this.cx.request_draw();
    }

    pub fn set_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.color = color;
        this.cx.request_draw();
    }
}

impl Widget for Divider {
    fn layout(&mut self, _cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let (_, minor) = self.axis.unpack_size(space.max);

        self.axis.pack_size(
            self.thickness + self.padding * 2.0,
            minor,
        )
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        let (_, minor) = self.axis.unpack_size(cx.size());

        let length = minor - self.inset * 2.0;
        let rect = Rect::min_size(
            self.axis.pack_point(self.padding, self.inset),
            self.axis.pack_size(self.thickness, length),
        );

        canvas.draw_rect(
            rect,
            self.corner_radius,
            &Paint::from(self.color),
        );
    }
}
