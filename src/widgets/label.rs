use crate::{BuildCx, Canvas, DrawCx, LayoutCx, Offset, Paragraph, Size, Space, Widget, WidgetId};

pub struct Label {
    paragraph: Paragraph,
}

impl Label {
    pub fn new(cx: &mut impl BuildCx, paragraph: Paragraph) -> WidgetId<Self> {
        cx.insert(Self { paragraph })
    }

    pub fn set_paragraph(cx: &mut impl BuildCx, id: WidgetId<Self>, paragraph: Paragraph) {
        cx.get_mut(id).paragraph = paragraph;
        cx.request_layout(id);
    }
}

impl Widget for Label {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        cx.fonts()
            .measure(&self.paragraph, space.max.width)
            .min(space.max)
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_text(&self.paragraph, cx.width(), Offset::all(0.0));
    }

    fn accepts_pointer() -> bool {
        false
    }
}
