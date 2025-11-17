use crate::{
    BuildCx, Canvas, Color, CornerRadius, DrawCx, EventCx, LayoutCx, Paint, Point, PointerEvent,
    Size, Space, Widget, WidgetId,
};

pub struct Button {
    padding: f32,
}

impl Button {
    pub fn new(cx: &mut BuildCx<'_>, child: WidgetId<impl Widget>) -> WidgetId<Self> {
        let this = cx.insert(Button { padding: 10.0 });

        cx.add_child(&this, child);

        this
    }

    pub fn set_child(cx: &mut BuildCx<'_>, id: &WidgetId<Self>, child: WidgetId<impl Widget>) {
        cx.replace_child(id, 0, child);
    }
}

impl Widget for Button {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let space = space.shrink(Size::all(self.padding * 2.0));
        let size = cx.layout_child(0, space);

        cx.place_child(0, Point::all(self.padding));

        size + self.padding * 2.0
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        let color = if cx.is_hovered() {
            Color::RED
        } else {
            Color::GREEN
        };

        canvas.draw_rect(cx.rect(), CornerRadius::all(8.0), &Paint::from(color));
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, _event: &PointerEvent) -> bool {
        cx.request_draw();

        true
    }
}
