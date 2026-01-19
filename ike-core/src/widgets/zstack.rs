use crate::{Builder, LayoutCx, Size, Space, Widget, WidgetMut};

pub struct ZStack {}

impl ZStack {
    pub fn new(cx: &mut impl Builder) -> WidgetMut<'_, Self> {
        cx.build_widget(Self {}).finish()
    }
}

impl Widget for ZStack {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let mut size = space.min;

        for i in 0..cx.children().len() {
            let child_size = cx.layout_nth_child(i, space);
            size = size.max(child_size);
        }

        space.constrain(size)
    }
}
