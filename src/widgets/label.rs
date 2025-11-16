use crate::{
    Canvas, DrawCx, LayoutCx, Offset, Size, Space, Widget, WidgetId,
    text::{Paragraph, TextAlign, TextStyle, TextWrap},
    widget::BuildCx,
};

pub struct Label {
    paragraph: Paragraph,
}

impl Label {
    pub fn new(cx: &mut BuildCx<'_>, text: impl ToString) -> WidgetId<Self> {
        let mut paragraph = Paragraph::new(16.0, TextAlign::Start, TextWrap::None);

        paragraph.push(
            text.to_string(),
            TextStyle {
                font_size: 16.0,
                font_family: String::from("Noto Sans"),
            },
        );

        cx.insert(Self { paragraph })
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
}
