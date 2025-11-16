use std::any::Any;

use crate::{
    Canvas, Point, WidgetId,
    canvas::Fonts,
    math::{Affine, Rect, Size, Space},
    tree::Tree,
};

pub trait Widget: Any {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size;

    fn draw(&mut self, cx: &mut DrawCx<'_>, scene: &mut dyn Canvas) {
        let _ = cx;
        let _ = scene;
    }

    fn on_pointer_event(&mut self) {}

    fn accepts_pointer() -> bool
    where
        Self: Sized,
    {
        true
    }

    fn accepts_focus() -> bool
    where
        Self: Sized,
    {
        false
    }

    fn accepts_text() -> bool
    where
        Self: Sized,
    {
        false
    }
}

pub struct WidgetState {
    pub(crate) affine: Affine,
    pub(crate) size: Size,
    pub(crate) parent: Option<WidgetId>,
    pub(crate) children: Vec<WidgetId>,

    pub(crate) accepts_pointer: bool,
    pub(crate) accepts_focus: bool,
    pub(crate) accepts_text: bool,
}

impl WidgetState {
    pub fn new<T: Widget>() -> Self {
        Self {
            affine: Affine::IDENTITY,
            size: Size::new(0.0, 0.0),
            parent: None,
            children: Vec::new(),

            accepts_focus: T::accepts_focus(),
            accepts_pointer: T::accepts_pointer(),
            accepts_text: T::accepts_text(),
        }
    }
}

pub struct LayoutCx<'a> {
    pub(crate) fonts: &'a mut dyn Fonts,
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
    pub fn fonts(&mut self) -> &mut dyn Fonts {
        self.fonts
    }

    pub fn children(&self) -> &[WidgetId] {
        &self.state.children
    }

    pub fn layout_child(&mut self, i: usize, space: Space) -> Size {
        let child = &self.state.children[i];

        self.tree.with(child, |tree, widget, state| {
            let mut cx = LayoutCx {
                fonts: self.fonts,
                tree,
                state,
            };

            let size = widget.layout(&mut cx, space);
            state.size = size;

            size
        })
    }

    pub fn place_child(&mut self, i: usize, point: Point) {
        let child = &self.state.children[i];

        let state = self.tree.widget_state_mut(child);
        state.affine.offset = point - Point::all(0.0);
    }
}

impl BuildCx<'_> {
    pub fn insert<T>(&mut self, widget: T) -> WidgetId<T>
    where
        T: Widget,
    {
        self.tree.insert(widget)
    }

    pub fn add_child<T, U>(&mut self, parent: &WidgetId<T>, child: WidgetId<U>)
    where
        T: ?Sized,
        U: ?Sized,
    {
        self.tree.add_child(parent, child);
    }

    pub fn replace_child<T, U>(&mut self, parent: &WidgetId<T>, index: usize, child: WidgetId<U>)
    where
        T: ?Sized,
        U: ?Sized,
    {
        self.tree.replace_child(parent, index, child);
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
    DrawCx<'_> {
        pub fn size(&self) -> Size {
            self.state.size
        }

        pub fn width(&self) -> f32 {
            self.size().width
        }

        pub fn height(&self) -> f32 {
            self.size().height
        }

        pub fn rect(&self) -> Rect {
            Rect::min_size(Point::all(0.0), self.size())
        }
    }
}
