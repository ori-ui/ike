use crate::{LayoutCx, Paragraph, Size, Space, TextLayoutLine, Widget};

pub struct TextArea {
    paragraph: Paragraph,
    lines:     Vec<TextLayoutLine>,
}

impl Widget for TextArea {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        self.lines = cx.fonts().layout(&self.paragraph, space.max.width);
        //let mut size = cx.fonts().measure(&self.paragraph, space.max.width);

        todo!()
    }
}
