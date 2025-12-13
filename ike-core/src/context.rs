use std::mem;

use crate::{
    Affine, AnyWidget, AnyWidgetId, Arena, Clip, CursorIcon, Offset, Painter, Point, Rect, Size,
    Space, Widget, WidgetId, WidgetMut, WidgetRef, Window, WindowId, root::RootState,
    widget::WidgetState,
};

pub(crate) enum FocusUpdate {
    None,
    Unfocus,
    Target(WidgetId),
    Next,
    Previous,
}

pub struct RefCx<'a> {
    pub(crate) root:  &'a RootState,
    pub(crate) arena: &'a Arena,
    pub(crate) state: &'a WidgetState,
}

pub struct MutCx<'a> {
    pub(crate) root:  &'a mut RootState,
    pub(crate) arena: &'a mut Arena,
    pub(crate) state: &'a mut WidgetState,
}

pub struct EventCx<'a> {
    pub(crate) root:  &'a mut RootState,
    pub(crate) arena: &'a mut Arena,
    pub(crate) state: &'a mut WidgetState,
    pub(crate) focus: &'a mut FocusUpdate,
}

pub struct UpdateCx<'a> {
    pub(crate) root:  &'a mut RootState,
    pub(crate) arena: &'a mut Arena,
    pub(crate) state: &'a mut WidgetState,
}

pub struct LayoutCx<'a> {
    pub(crate) scale: f32,
    pub(crate) root:  &'a mut RootState,
    pub(crate) arena: &'a mut Arena,
    pub(crate) state: &'a mut WidgetState,
}

pub struct DrawCx<'a> {
    pub(crate) root:  &'a mut RootState,
    pub(crate) arena: &'a mut Arena,
    pub(crate) state: &'a mut WidgetState,
}

impl MutCx<'_> {
    pub fn get_mut<U>(&mut self, id: WidgetId<U>) -> Option<WidgetMut<'_, U>>
    where
        U: ?Sized + AnyWidget,
    {
        self.arena.get_mut(self.root, id)
    }

    pub fn get_child_mut(&mut self, index: usize) -> Option<WidgetMut<'_, dyn Widget>> {
        self.arena.get_mut(
            self.root,
            *self.state.children.get(index)?,
        )
    }

    pub fn get_parent_mut(&mut self) -> Option<WidgetMut<'_>> {
        self.arena.get_mut(self.root, self.state.parent?)
    }

    pub fn for_each_child(&mut self, mut f: impl FnMut(&mut WidgetMut)) {
        for &child in &self.state.children {
            if let Some(mut child) = self.arena.get_mut(self.root, child) {
                f(&mut child);
            }
        }
    }

    pub fn request_animate(&mut self) {
        self.state.needs_animate = true;
        self.propagate_state();

        if let Some(window) = self.state.window {
            self.root.request_animate(window);
        }
    }

    pub fn request_layout(&mut self) {
        self.state.needs_layout = true;
        self.state.needs_draw = true;
        self.propagate_state();

        if let Some(window) = self.state.window {
            self.root.request_redraw(window);
        }
    }

    pub fn request_draw(&mut self) {
        self.state.needs_draw = true;
        self.propagate_state();

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

    pub(crate) fn split(
        &mut self,
    ) -> (
        &mut RootState,
        &mut Arena,
        &mut WidgetState,
    ) {
        (self.root, self.arena, self.state)
    }

    pub(crate) fn as_update_cx(&mut self) -> UpdateCx<'_> {
        let (root, arena, state) = self.split();
        UpdateCx { root, arena, state }
    }

    pub(crate) fn as_layout_cx(&mut self, scale: f32) -> LayoutCx<'_> {
        let (root, arena, state) = self.split();

        LayoutCx {
            scale,
            root,
            arena,
            state,
        }
    }

    pub(crate) fn as_draw_cx(&mut self) -> DrawCx<'_> {
        let (root, arena, state) = self.split();
        DrawCx { root, arena, state }
    }

    pub(crate) fn enter_span(&self) -> tracing::span::EnteredSpan {
        self.state.tracing_span.clone().entered()
    }

    pub(crate) fn update_state(&mut self) {
        self.state.reset();

        let children = mem::take(&mut self.state.children);
        for &child in &children {
            if let Some(child_state) = self.arena.get_state(child.index) {
                self.state.merge(child_state);
            }
        }

        self.state.children = children;
    }

    pub(crate) fn propagate_state(&mut self) {
        // update our own state, this is necessary, but I don't quite know why
        self.update_state();

        let Some(parent) = self.state.parent else {
            return;
        };

        let id = self.id();

        // this is a bit of a mess since self is borrowed, we need to handle that special case for
        // our parent, but not for our parents ancestors
        let grand_parent = if let Some(parent) = self.arena.get_mut(self.root, parent) {
            parent.cx.state.reset();

            let children = mem::take(&mut parent.cx.state.children);

            for &child in &children {
                if child == id {
                    parent.cx.state.merge(self.state);
                    continue;
                }

                if let Some(child_state) = parent.cx.arena.get_state(child.index) {
                    parent.cx.state.merge(child_state);
                }
            }

            parent.cx.state.children = children;
            parent.cx.state.parent
        } else {
            None
        };

        if let Some(grand_parent) = grand_parent {
            self.propagate_state_recursive(grand_parent);
        }
    }

    fn propagate_state_recursive(&mut self, widget: WidgetId) {
        let parent = if let Some(widget) = self.arena.get_mut(self.root, widget) {
            widget.cx.state.reset();

            let children = mem::take(&mut widget.cx.state.children);

            for &child in &children {
                if let Some(child_state) = widget.cx.arena.get_state(child.index) {
                    widget.cx.state.merge(child_state);
                }
            }

            widget.cx.state.children = children;
            widget.cx.state.parent
        } else {
            None
        };

        if let Some(parent) = parent {
            self.propagate_state_recursive(parent);
        }
    }

    pub(crate) fn set_window_recursive(&mut self, window: Option<WindowId>) {
        self.state.window = window;
        self.for_each_child(|child| child.cx.set_window_recursive(window));
    }
}

impl EventCx<'_> {
    pub fn request_focus(&mut self) {
        *self.focus = FocusUpdate::Target(self.id());
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
    pub fn set_child_stashed(&mut self, index: usize, is_stashed: bool) {
        let child = self.state.children[index];
        let mut child = self.arena.get_mut(self.root, child).unwrap();
        child.set_stashed_without_propagate(is_stashed);
    }

    pub fn layout_child(&mut self, index: usize, painter: &mut dyn Painter, space: Space) -> Size {
        let child = self.state.children[index];
        let mut child = self.arena.get_mut(self.root, child).unwrap();
        child.layout_recursive(self.scale, painter, space)
    }

    pub fn place_child(&mut self, index: usize, offset: Offset) {
        let child = &self.state.children[index];

        if let Some(state) = self.arena.get_state_mut(child.index) {
            state.transform.offset = offset;
        } else {
            tracing::error!("`LayoutCx::place` called on widget that doesn't exist");
        }
    }

    pub fn child_size(&self, index: usize) -> Size {
        let child = &self.state.children[index];

        if let Some(state) = self.arena.get_state(child.index) {
            state.size
        } else {
            Size::ZERO
        }
    }

    pub fn set_clip(&mut self, rect: impl Into<Option<Clip>>) {
        self.state.clip = rect.into();
    }
}

macro_rules! impl_contexts {
    ($cx:ident <'_ $(, $t:ident)?> { $($tt:tt)* }) => {
        impl $(<$t: ?Sized + AnyWidget>)? $cx<'_ $(, $t)?> {
            $($tt)*
        }
    };
    (
        $cx:ident <'_ $(, $t:ident)?>,
        $($cxs:ident <'_ $(, $u:ident)?>),* $(,)*
        { $($tt:tt)* }
    ) => {
        impl_contexts!($cx <'_ $(, $t)?> { $($tt)* });
        impl_contexts!($($cxs <'_ $(, $u)?>),* { $($tt)* });
    }
}

impl_contexts! {
    RefCx<'_>,
    MutCx<'_>,
    EventCx<'_>,
    UpdateCx<'_>,
    LayoutCx<'_>,
    DrawCx<'_> {
        pub fn id(&self) -> WidgetId {
            self.state.id
        }
    }
}

impl_contexts! {
    RefCx<'_>,
    MutCx<'_>,
    EventCx<'_>,
    UpdateCx<'_>,
    LayoutCx<'_>,
    DrawCx<'_> {
        pub fn get<U>(&self, id: WidgetId<U>) -> Option<WidgetRef<'_, U>>
        where
            U: ?Sized + AnyWidget,
        {
            self.arena.get(self.root, id)
        }

        pub fn children(&self) -> &[WidgetId] {
            &self.state.children
        }

        pub fn iter_children(&self) -> impl ExactSizeIterator<Item = WidgetRef<'_>> {
            self.state.children.iter().map(|child| {
                self.arena.get(self.root, *child).unwrap()
            })
        }

        pub fn is_child(&self, id: impl AnyWidgetId) -> bool {
            self.get(id.upcast()).is_some_and(|w| w.cx.state.parent == Some(self.id().upcast()))
        }

        pub fn get_child(&self, index: usize) -> Option<WidgetRef<'_>> {
            self.arena.get(self.root, self.state.children[index])
        }

        pub fn parent(&self) -> Option<WidgetId> {
            self.state.parent
        }

        pub fn get_parent(&self) -> Option<WidgetRef<'_>> {
            self.arena.get(self.root, self.state.parent?)
        }

        pub fn get_window(&self) -> Option<&Window> {
            self.root.get_window(self.state.window?)
        }

        pub fn is_window_focused(&self) -> bool {
            self.get_window().is_some_and(|window| window.is_focused())
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

        pub fn is_stashed(&self) -> bool {
            self.state.is_stashed
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

        pub fn cursor(&self) -> CursorIcon{
            self.state.cursor
        }
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
    RefCx<'_>,
    MutCx<'_>,
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
