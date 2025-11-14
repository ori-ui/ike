use crate::{
    Canvas,
    math::{Size, Space},
    tree::WidgetId,
    widget::{BuildCx, DrawCx, LayoutCx, Widget},
};

pub struct Button {
    content: WidgetId,
}

impl Button {
    pub fn new(cx: &mut BuildCx<'_>, content: WidgetId<impl Widget>) -> WidgetId<Self> {
        cx.insert(Button {
            content: content.cast(),
        })
    }
}

impl Widget for Button {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        cx.layout_child(&self.content, space)
    }

    fn draw(&mut self, _cx: &mut DrawCx<'_>, _canvas: &mut dyn Canvas) {}
}
