use crate::{BuildCx, Canvas, DrawCx, LayoutCx, Offset, Paragraph, Size, Space, Widget, WidgetMut};

pub struct Label {
    paragraph: Paragraph,
}

impl Label {
    pub fn new(cx: &mut impl BuildCx, paragraph: Paragraph) -> WidgetMut<'_, Self> {
        cx.insert(Self { paragraph })
    }

    pub fn set_text(this: &mut WidgetMut<Self>, paragraph: Paragraph) {
        this.paragraph = paragraph;
        this.request_layout();
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
