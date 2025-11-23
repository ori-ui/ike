use crate::{AnyWidgetId, BuildCx, LayoutCx, Size, Space, Widget, WidgetMut};

pub struct Constrain {
    min_size: Size,
    max_size: Size,
}

impl Constrain {
    pub fn new(cx: &mut impl BuildCx, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        let mut this = cx.insert(Self {
            min_size: Size::all(0.0),
            max_size: Size::all(f32::INFINITY),
        });
        this.add_child(child);
        this
    }

    pub fn set_child(this: &mut WidgetMut<Self>, child: impl AnyWidgetId) {
        this.replace_child(0, child);
    }

    pub fn set_min_size(this: &mut WidgetMut<Self>, min_size: Size) {
        this.min_size = min_size;
        this.request_layout();
    }

    pub fn set_max_size(this: &mut WidgetMut<Self>, max_size: Size) {
        this.max_size = max_size;
        this.request_layout();
    }
}

impl Widget for Constrain {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, mut space: Space) -> Size {
        space.min = space.min.max(self.min_size);
        space.max = space.max.min(self.max_size);
        space.min = space.min.min(space.max);

        cx.layout_child(0, space)
    }
}
