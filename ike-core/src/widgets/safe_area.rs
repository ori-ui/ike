use crate::{
    AnyWidgetId, Builder, LayoutCx, Offset, Point, Size, Space, Update, UpdateCx, Widget, WidgetMut,
};

#[non_exhaustive]
pub struct SafeArea {}

impl SafeArea {
    pub fn new(cx: &mut impl Builder, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        cx.build_widget(SafeArea {}).with_child(child).finish()
    }
}

impl Widget for SafeArea {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, mut space: Space) -> Size {
        if let Some(window) = cx.get_window()
            && let Some(safe_area) = window.safe_area()
        {
            space.max = space.max.min(safe_area.size());
            space.min = space.min.min(safe_area.size());

            let size = cx.layout_child(0, space);
            cx.place_child(0, safe_area.top_left() - Point::ORIGIN);
            size
        } else {
            let size = cx.layout_child(0, space);
            cx.place_child(0, Offset::ZERO);
            size
        }
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        if let Update::WindowSafeAreaChanged(..) = update {
            cx.request_layout();
        }
    }
}
