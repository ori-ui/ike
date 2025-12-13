use crate::{
    BuildCx, Canvas, DrawCx, LayoutCx, Offset, Painter, Paragraph, Point, Rect, Size, Space,
    Widget, WidgetMut,
};

pub struct Label {
    paragraph: Paragraph,
}

impl Label {
    pub fn new(cx: &mut impl BuildCx, paragraph: Paragraph) -> WidgetMut<'_, Self> {
        cx.insert(Self { paragraph })
    }

    pub fn set_text(this: &mut WidgetMut<Self>, paragraph: Paragraph) {
        this.widget.paragraph = paragraph;
        this.cx.request_layout();
    }
}

impl Widget for Label {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size {
        let size = painter.measure_text(&self.paragraph, space.max.width);
        let size = space.constrain(size);
        cx.set_clip(Rect::min_size(Point::ORIGIN, size));

        size
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_text(
            &self.paragraph,
            cx.width(),
            Offset::all(0.0),
        );
    }
}
