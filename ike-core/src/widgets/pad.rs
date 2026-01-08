use crate::{AnyWidgetId, BuildCx, LayoutCx, Padding, Painter, Size, Space, Widget, WidgetMut};

pub struct Pad {
    padding: Padding,
}

impl Pad {
    pub fn new(cx: &mut impl BuildCx, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        cx.build_widget(Pad {
            padding: Padding::all(8.0),
        })
        .with_child(child)
        .finish()
    }

    pub fn set_padding(this: &mut WidgetMut<Self>, padding: Padding) {
        this.widget.padding = padding;
        this.cx.request_layout();
    }
}

impl Widget for Pad {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size {
        let space = space.shrink(self.padding.size());
        let size = cx.layout_child(0, painter, space);
        cx.place_child(0, self.padding.offset());
        size + self.padding.size()
    }
}
