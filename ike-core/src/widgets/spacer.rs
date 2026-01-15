use crate::{Builder, LayoutCx, Size, Space, Widget, WidgetMut};

pub struct Spacer {
    size: Size,
}

impl Spacer {
    pub fn new(cx: &mut impl Builder) -> WidgetMut<'_, Self> {
        cx.build_widget(Self { size: Size::ZERO }).finish()
    }

    pub fn set_size(this: &mut WidgetMut<Self>, size: Size) {
        this.widget.size = size;
        this.cx.request_layout();
    }
}

impl Widget for Spacer {
    fn layout(&mut self, _cx: &mut LayoutCx<'_>, space: Space) -> Size {
        space.constrain(self.size)
    }
}
