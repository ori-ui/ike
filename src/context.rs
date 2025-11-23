use crate::{
    Affine, App, Fonts, Offset, Point, Rect, Size, Space, Tree, WidgetId, Window, WindowId,
    widget::WidgetState,
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
    pub(crate) app:    &'a mut App,
    pub(crate) state:  &'a mut WidgetState,
    pub(crate) id:     WidgetId,
    pub(crate) focus:  &'a mut FocusUpdate,
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
    pub(crate) window: &'a Window,
    #[allow(dead_code)]
    pub(crate) tree:   &'a mut Tree,
    pub(crate) state:  &'a mut WidgetState,
}

impl EventCx<'_> {
    pub fn window(&self) -> &Window {
        self.app.get_window(self.window).unwrap()
    }

    pub fn window_mut(&mut self) -> &mut Window {
        self.app.get_window_mut(self.window).unwrap()
    }

    pub fn is_window_focused(&self) -> bool {
        self.window().is_focused()
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

        let state = self.tree.get_state_unchecked_mut(child.index);
        state.transform.offset = offset;
    }

    pub fn child_size(&self, i: usize) -> Size {
        let child = &self.state.children[i];

        let state = self.tree.get_state_unchecked(child.index);
        state.size
    }
}

impl DrawCx<'_> {
    pub fn is_window_focused(&self) -> bool {
        self.window.is_focused
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

        pub fn has_hovered(&self) -> bool {
            self.state.has_hovered
        }

        pub fn has_active(&self) -> bool {
            self.state.has_active
        }

        pub fn has_focused(&self) -> bool {
            self.state.has_focused
        }
    }
}
