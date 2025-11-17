use crate::{
    Fonts, Point, Size, Space, Tree, Widget, WidgetId,
    math::{Affine, Rect},
    widget::WidgetState,
};

pub struct BuildCx<'a> {
    pub(crate) tree: &'a mut Tree,
}

pub struct EventCx<'a> {
    pub(crate) fonts: &'a mut dyn Fonts,
    pub(crate) tree: &'a mut Tree,
    pub(crate) state: &'a mut WidgetState,
    pub(crate) transform: Affine,
}

pub struct LayoutCx<'a> {
    pub(crate) fonts: &'a mut dyn Fonts,
    pub(crate) tree: &'a mut Tree,
    pub(crate) state: &'a mut WidgetState,
}

pub struct DrawCx<'a> {
    pub(crate) tree: &'a mut Tree,
    pub(crate) state: &'a mut WidgetState,
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

impl EventCx<'_> {
    pub fn request_layout(&mut self) {
        self.state.needs_layout = true;
    }

    pub fn request_draw(&mut self) {
        self.state.needs_draw = true;
    }
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

        let state = self.tree.widget_state_mut(child.index);
        state.transform.offset = point - Point::all(0.0);
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
    EventCx<'_>,
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

        pub fn is_hovered(&self) -> bool {
            self.state.is_hovered
        }
    }
}
