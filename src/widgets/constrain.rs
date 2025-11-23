use crate::{AnyWidgetId, BuildCx, LayoutCx, Size, Space, Widget, WidgetId};

pub struct Constrain {
    min_size: Size,
    max_size: Size,
}

impl Constrain {
    pub fn new(cx: &mut impl BuildCx, child: impl AnyWidgetId) -> WidgetId<Self> {
        let this = cx.insert(Self {
            min_size: Size::all(0.0),
            max_size: Size::all(f32::INFINITY),
        });

        cx.add_child(this, child);

        this
    }

    pub fn set_child(cx: &mut impl BuildCx, id: WidgetId<Self>, child: impl AnyWidgetId) {
        cx.replace_child(id, 0, child);
    }

    pub fn set_min_size(cx: &mut impl BuildCx, id: WidgetId<Self>, min_size: Size) {
        cx.get_mut(id).min_size = min_size;
        cx.request_layout(id);
    }

    pub fn set_max_size(cx: &mut impl BuildCx, id: WidgetId<Self>, max_size: Size) {
        cx.get_mut(id).max_size = max_size;
        cx.request_layout(id);
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
