use crate::{
    Affine, AnyWidgetId, App, Fonts, Offset, Point, Size, Space, Tree, Widget, WidgetId,
    math::Rect,
    widget::{AnyWidget, WidgetState},
};

pub struct EventCx<'a> {
    #[allow(dead_code)]
    pub(crate) app:   &'a mut App,
    pub(crate) state: &'a mut WidgetState,
    pub(crate) id:    WidgetId,
    pub(crate) focus: &'a mut Option<WidgetId>,
}

pub struct UpdateCx<'a> {
    #[allow(dead_code)]
    pub(crate) tree:  &'a mut Tree,
    pub(crate) state: &'a mut WidgetState,
}

pub struct LayoutCx<'a> {
    pub(crate) fonts: &'a mut dyn Fonts,
    pub(crate) tree:  &'a mut Tree,
    pub(crate) state: &'a mut WidgetState,
}

pub struct DrawCx<'a> {
    #[allow(dead_code)]
    pub(crate) tree:  &'a mut Tree,
    pub(crate) state: &'a mut WidgetState,
}

pub trait BuildCx {
    fn app(&self) -> &App;
    fn app_mut(&mut self) -> &mut App;

    fn get<T>(&self, id: WidgetId<T>) -> &T
    where
        T: ?Sized + AnyWidget,
    {
        &self.app().tree[id]
    }

    fn get_mut<T>(&mut self, id: WidgetId<T>) -> &mut T
    where
        T: ?Sized + AnyWidget,
    {
        &mut self.app_mut().tree[id]
    }

    fn insert<T>(&mut self, widget: T) -> WidgetId<T>
    where
        Self: Sized,
        T: Widget,
    {
        self.app_mut().tree.insert(widget)
    }

    fn remove(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        self.app_mut().tree.remove(id.upcast());
    }

    fn add_child(&mut self, parent: impl AnyWidgetId, child: impl AnyWidgetId)
    where
        Self: Sized,
    {
        let child = child.upcast();
        self.app_mut().tree.add_child(parent, child);
        self.app_mut().tree.request_layout(child);
    }

    fn remove_child(&mut self, parent: impl AnyWidgetId, index: usize)
    where
        Self: Sized,
    {
        let parent = parent.upcast();
        let state = self.app().tree.widget_state(parent.index);
        let child = state.children[index];
        self.app_mut().tree.remove(child);
        self.app_mut().tree.request_layout(parent);
    }

    fn replace_child(&mut self, parent: impl AnyWidgetId, index: usize, child: impl AnyWidgetId)
    where
        Self: Sized,
    {
        let child = child.upcast();
        self.app_mut().tree.replace_child(parent, index, child);
        self.app_mut().tree.request_layout(child);
    }

    fn swap_children(&mut self, parent: impl AnyWidgetId, index_a: usize, index_b: usize)
    where
        Self: Sized,
    {
        let parent = parent.upcast();

        self.app_mut().tree.swap_children(parent, index_a, index_b);
        self.app_mut().tree.request_layout(parent);
    }

    fn children(&self, widget: impl AnyWidgetId) -> &[WidgetId] {
        &self.app().tree.widget_state(widget.upcast().index).children
    }

    fn is_parent(&self, parent: impl AnyWidgetId, child: impl AnyWidgetId) -> bool
    where
        Self: Sized,
    {
        let parent = parent.upcast();
        let child = child.upcast();

        self.app().tree.widget_state(child.index).parent == Some(parent)
    }

    fn request_animate(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        self.app_mut().tree.request_animate(id);
    }

    fn request_layout(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        self.app_mut().tree.request_layout(id);
    }

    fn request_draw(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        self.app_mut().tree.request_draw(id);
    }
}

impl EventCx<'_> {
    pub fn request_focus(&mut self) {
        *self.focus = Some(self.id);
    }

    pub fn request_unfocus(&mut self) {
        self.state.is_focused = false;
    }
}

impl LayoutCx<'_> {
    pub fn fonts(&mut self) -> &mut dyn Fonts {
        self.fonts
    }

    pub fn layout_child(&mut self, i: usize, space: Space) -> Size {
        let child = self.state.children[i];

        self.tree.with_entry(child, |tree, widget, state| {
            state.needs_layout = false;

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

    pub fn place_child(&mut self, i: usize, offset: Offset) {
        let child = &self.state.children[i];

        let state = self.tree.widget_state_mut(child.index);
        state.transform.offset = offset;
    }

    pub fn child_size(&self, i: usize) -> Size {
        let child = &self.state.children[i];

        let state = self.tree.widget_state(child.index);
        state.size
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
    UpdateCx<'_> {
        pub fn request_animate(&mut self) {
            self.state.needs_animate = true;
        }

        pub fn request_layout(&mut self) {
            self.state.needs_layout = true;
            self.state.needs_draw = true;
        }

        pub fn request_draw(&mut self) {
            self.state.needs_draw = true;
        }
    }
}

impl_contexts! {
    EventCx<'_>,
    UpdateCx<'_>,
    LayoutCx<'_>,
    DrawCx<'_> {
        pub fn children(&self) -> &[WidgetId] {
            &self.state.children
        }

        pub fn transform(&self) -> Affine {
            self.state.transform
        }
    }
}

impl_contexts! {
    EventCx<'_>,
    UpdateCx<'_>,
    DrawCx<'_> {
        pub fn global_transform(&self) -> Affine {
            self.state.global_transform
        }

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

        pub fn is_active(&self) -> bool {
            self.state.is_active
        }

        pub fn is_focused(&self) -> bool {
            self.state.is_focused
        }
    }
}
