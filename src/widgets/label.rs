use crate::{
    BuildCx, Canvas, Color, DrawCx, LayoutCx, Offset, Size, Space, Widget, WidgetId,
    text::{Paragraph, TextAlign, TextStyle, TextWrap},
};

pub struct Label {
    paragraph: Paragraph,
}

impl Label {
    pub fn new(cx: &mut impl BuildCx, text: impl ToString) -> WidgetId<Self> {
        let mut paragraph = Paragraph::new(16.0, TextAlign::Start, TextWrap::None);

        paragraph.push(
            text.to_string(),
            TextStyle {
                font_size: 16.0,
                font_family: String::from("Noto Sans"),
                color: Color::BLACK,
            },
        );

        cx.insert(Self { paragraph })
    }

    pub fn set_text(cx: &mut impl BuildCx, id: WidgetId<Self>, text: impl ToString) {
        let label = cx.get_mut(id);
        let (_, style) = label.paragraph.sections().next().unwrap();
        let style = style.clone();

        label.paragraph.clear();
        label.paragraph.push(text.to_string(), style);

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
