use crate::{
    Affine, AnyWidget, Clip, CursorIcon, Offset, Painter, Point, Rect, Size, Space, Tree, WidgetId,
    WidgetRef, Window, WindowId, root::RootState, widget::WidgetState,
};

pub(crate) enum FocusUpdate {
    None,
    Unfocus,
    Target(WidgetId),
    Next,
    Previous,
}

pub struct EventCx<'a> {
    pub(crate) window: WindowId,
    pub(crate) root:   &'a mut RootState,
    pub(crate) tree:   &'a mut Tree,
    pub(crate) state:  &'a mut WidgetState,
    pub(crate) id:     WidgetId,
    pub(crate) focus:  &'a mut FocusUpdate,
}

pub struct UpdateCx<'a> {
    pub(crate) root:  &'a mut RootState,
    pub(crate) tree:  &'a mut Tree,
    pub(crate) state: &'a mut WidgetState,
}

pub struct LayoutCx<'a> {
    pub(crate) window: WindowId,
    pub(crate) root:   &'a mut RootState,
    pub(crate) tree:   &'a mut Tree,
    pub(crate) state:  &'a mut WidgetState,
}

pub struct DrawCx<'a> {
    pub(crate) window: WindowId,
    pub(crate) root:   &'a mut RootState,
    pub(crate) tree:   &'a mut Tree,
    pub(crate) state:  &'a mut WidgetState,
}

impl EventCx<'_> {
    pub fn window(&self) -> &Window {
        self.root.get_window(self.window).unwrap()
    }

    pub fn request_focus(&mut self) {
        *self.focus = FocusUpdate::Target(self.id);
    }

    pub fn request_unfocus(&mut self) {
        *self.focus = FocusUpdate::Unfocus;
    }

    pub fn request_focus_next(&mut self) {
        *self.focus = FocusUpdate::Next;
    }

    pub fn request_focus_previous(&mut self) {
        *self.focus = FocusUpdate::Previous;
    }
}

impl UpdateCx<'_> {
    pub fn window(&self) -> &Window {
        self.root.get_window(self.state.window.unwrap()).unwrap()
    }
}

impl LayoutCx<'_> {
    pub fn get<T>(&self, id: WidgetId<T>) -> WidgetRef<'_, T>
    where
        T: ?Sized + AnyWidget,
    {
        self.tree.get(self.root, id).unwrap()
    }

    pub fn set_child_stashed(&mut self, index: usize, is_stashed: bool) {
        let child = self.state.children[index];
        let mut child = self.tree.get_mut(self.root, child).unwrap();
        child.set_stashed(is_stashed);
    }

    pub fn layout_child(&mut self, index: usize, painter: &mut dyn Painter, space: Space) -> Size {
        let child = self.state.children[index];
        let mut child = self.tree.get_mut(self.root, child).unwrap();
        child.layout_recursive(self.window, painter, space)
    }

    pub fn place_child(&mut self, index: usize, offset: Offset) {
        let child = &self.state.children[index];

        let state = self.tree.get_state_unchecked_mut(child.index);
        state.transform.offset = offset;
    }

    pub fn child_size(&self, index: usize) -> Size {
        let child = &self.state.children[index];

        let state = self.tree.get_state_unchecked(child.index);
        state.size
    }

    pub fn set_clip(&mut self, rect: impl Into<Option<Clip>>) {
        self.state.clip = rect.into();
    }

    pub fn window(&self) -> &Window {
        self.root.get_window(self.window).unwrap()
    }
}

impl DrawCx<'_> {
    pub fn window(&self) -> &Window {
        self.root.get_window(self.window).unwrap()
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

            if let Some(window) = self.state.window {
                self.root.request_animate(window);
            }
        }

        pub fn request_layout(&mut self) {
            self.state.needs_layout = true;
            self.state.needs_draw = true;

            if let Some(window) = self.state.window {
                self.root.request_redraw(window);
            }
        }

        pub fn request_draw(&mut self) {
            self.state.needs_draw = true;

            if let Some(window) = self.state.window {
                self.root.request_redraw(window);
            }
        }

        pub fn set_pixel_perfect(&mut self, pixel_perfect: bool) {
            if self.is_pixel_perfect() != pixel_perfect {
                self.state.is_pixel_perfect = pixel_perfect;
                self.request_layout();
            }
        }

        pub fn set_cursor(&mut self, cursor: CursorIcon) {
            self.state.cursor = cursor;
        }
    }
}

impl_contexts! {
    EventCx<'_>,
    UpdateCx<'_>,
    LayoutCx<'_>,
    DrawCx<'_> {
        pub fn children(&self) -> impl ExactSizeIterator<Item = WidgetRef<'_>> {
            self.state.children.iter().map(|child| self.tree.get(self.root, *child).unwrap())
        }

        pub fn child(&self, index: usize) -> Option<WidgetRef<'_>> {
            self.tree.get(self.root, self.state.children[index])
        }

        pub fn is_window_focused(&self) -> bool {
            self.window().is_focused()
        }

        pub fn is_pixel_perfect(&self) -> bool {
            self.state.is_pixel_perfect
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

        pub fn has_hovered(&self) -> bool {
            self.state.has_hovered
        }

        pub fn has_active(&self) -> bool {
            self.state.has_active
        }

        pub fn has_focused(&self) -> bool {
            self.state.has_focused
        }

        pub fn clip(&self) -> Option<&Clip> {
            self.state.clip.as_ref()
        }

        pub fn id(&self) -> WidgetId {
            self.state.widget_id
        }
    }
}

impl_contexts! {
    EventCx<'_>,
    UpdateCx<'_>,
    DrawCx<'_> {
        pub fn transform(&self) -> Affine {
            self.state.transform
        }

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
    }
}
