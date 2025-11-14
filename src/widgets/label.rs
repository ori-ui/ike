use crate::{Canvas, DrawCx, LayoutCx, Size, Space, Widget, WidgetId, widget::BuildCx};

pub struct Label {
    text: String,
}

impl Label {
    pub fn new(cx: &mut BuildCx<'_>, text: impl ToString) -> WidgetId<Self> {
        cx.insert(Self {
            text: text.to_string(),
        })
    }
}

impl Widget for Label {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        space.min
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.text(&self.text);
    }
}
