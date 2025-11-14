use std::any::Any;

use crate::{
    Canvas, Point, WidgetId,
    math::{Size, Space},
    tree::Tree,
};

pub trait Widget: Any {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size;

    fn draw(&mut self, cx: &mut DrawCx<'_>, scene: &mut dyn Canvas);
}

pub struct WidgetState {
    pub(crate) size: Size,
}

impl WidgetState {
    pub fn new() -> Self {
        Self {
            size: Size::new(0.0, 0.0),
        }
    }
}

pub struct LayoutCx<'a> {
    pub(crate) tree: &'a mut Tree,
    pub(crate) state: &'a mut WidgetState,
}

pub struct DrawCx<'a> {
    pub(crate) tree: &'a mut Tree,
    pub(crate) state: &'a WidgetState,
}

pub struct BuildCx<'a> {
    pub(crate) tree: &'a mut Tree,
}

impl LayoutCx<'_> {
    pub fn layout_child<T>(&mut self, id: &WidgetId<T>, space: Space) -> Size
    where
        T: ?Sized + Widget,
    {
        self.tree.with(id, |tree, widget, state| {
            let mut cx = LayoutCx { tree, state };

            let size = widget.layout(&mut cx, space);
            state.size = size;

            size
        })
    }

    pub fn place_child<T>(&mut self, _id: &WidgetId<T>, _point: Point) {}
}

impl BuildCx<'_> {
    pub fn insert<T>(&mut self, widget: T) -> WidgetId<T>
    where
        T: Widget,
    {
        self.tree.insert(widget)
    }
}

macro_rules! impl_contexts {
    ($cx:ty { $($tt:tt)* }) => {
        impl $cx {
            $($tt)*
        }
    };
    ($cx:ty, $($cxs:ty),* $(,)* { $($tt:tt)* }) => {
        impl_contexts!($cx { $($tt)* });
        impl_contexts!($($cxs),* { $($tt)* });
    }
}

impl_contexts! {
    LayoutCx<'_>,
    DrawCx<'_> {

    }
}
