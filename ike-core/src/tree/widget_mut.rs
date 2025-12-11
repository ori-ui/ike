use std::{
    mem,
    ops::{Deref, DerefMut},
    time::Duration,
};

use crate::{
    Affine, AnyWidget, AnyWidgetId, Canvas, ChildUpdate, CursorIcon, DrawCx, LayoutCx, Painter,
    Size, Space, Tree, Update, UpdateCx, Widget, WidgetId, WidgetRef, WidgetState, Window,
};

pub struct WidgetMut<'a, T = dyn Widget>
where
    T: ?Sized + AnyWidget,
{
    pub(crate) id:     WidgetId<T>,
    pub(crate) tree:   &'a mut Tree,
    pub(crate) widget: *mut T,
    pub(crate) state:  *mut WidgetState,
}

impl<'a, T> WidgetMut<'a, T>
where
    T: ?Sized + AnyWidget,
{
    pub fn request_layout(&mut self) {
        self.state_mut().needs_layout = true;
        self.state_mut().needs_draw = true;

        self.propagate_state();
    }

    pub fn request_draw(&mut self) {
        self.state_mut().needs_draw = true;

        self.propagate_state();
    }

    pub fn request_animate(&mut self) {
        self.state_mut().needs_animate = true;
    }

    pub fn parent_mut(&mut self) -> Option<WidgetMut<'_, dyn Widget>> {
        self.tree.get_mut(self.state().parent?)
    }

    pub fn child_mut(&mut self, index: usize) -> WidgetMut<'_, dyn Widget> {
        self.tree.get_mut(self.state().children[index]).unwrap()
    }

    pub fn set_cursor(&mut self, cursor: CursorIcon) {
        self.state_mut().cursor = cursor;
    }

    pub fn for_each_child(&mut self, mut f: impl FnMut(&mut WidgetMut)) {
        for &child in &unsafe { &*self.state }.children {
            let mut child = self.tree.get_mut(child).unwrap();
            f(&mut child);
        }
    }

    pub fn as_ref(&self) -> WidgetRef<'_, T> {
        WidgetRef {
            id:     self.id,
            tree:   self.tree,
            widget: unsafe { &*self.widget },
            state:  unsafe { &*self.state },
        }
    }

    pub fn get_mut<U>(&mut self, id: WidgetId<U>) -> Option<WidgetMut<'_, U>>
    where
        U: ?Sized + AnyWidget,
    {
        self.tree.get_mut(id)
    }

    pub fn set_pixel_perfect(&mut self, pixel_perfect: bool) {
        if self.is_pixel_perfect() != pixel_perfect {
            self.state_mut().is_pixel_perfect = pixel_perfect;
            self.request_layout();
        }
    }

    pub fn set_stashed(&mut self, is_stashed: bool) {
        if self.is_stashed() == is_stashed {
            return;
        }

        if !is_stashed && self.parent().is_some_and(|p| p.state().is_stashed) {
            return;
        }

        self.state_mut().is_stashing = is_stashed;
        self.set_stashed_recursive(is_stashed);
        self.propagate_state();
    }

    fn set_stashed_recursive(&mut self, is_stashed: bool) {
        self.for_each_child(|child| child.set_stashed_recursive(is_stashed));

        self.state_mut().is_stashed = is_stashed;
        self.state_mut().needs_layout = true;

        if !is_stashed {
            self.state_mut().is_focused = false;
            self.state_mut().is_hovered = false;
            self.state_mut().is_active = false;

            self.update_without_propagate(Update::Focused(false));
            self.update_without_propagate(Update::Hovered(false));
            self.update_without_propagate(Update::Active(false));
        }

        self.update_without_propagate(Update::Stashed(is_stashed));
        self.update_state();
    }

    /// Add a child to widget.
    pub fn add_child(&mut self, child: impl AnyWidgetId) {
        let child = child.upcast();
        self.tree.debug_validate_id(child);

        let child_state = self.tree.get_state_unchecked_mut(child.index);
        child_state.parent = Some(self.id.upcast());

        let index = self.state().children.len();
        self.state_mut().children.push(child.upcast());
        self.update_without_propagate(Update::Children(ChildUpdate::Added(
            index,
        )));
        self.request_layout();
    }

    pub fn remove_child(&mut self, index: usize) {
        if let Some(&child) = self.children().get(index) {
            self.with_tree(|tree| tree.remove(child));
        }
    }

    /// Swap two children of a widget.
    pub fn swap_children(&mut self, index_a: usize, index_b: usize) {
        self.state_mut().children.swap(index_a, index_b);
        self.update_without_propagate(Update::Children(ChildUpdate::Swapped(
            index_a, index_b,
        )));
        self.request_layout();
    }

    /// Replace the child of a widget with another returning the previous child.
    pub fn replace_child(&mut self, index: usize, child: impl AnyWidgetId) -> WidgetId {
        let child = child.upcast();
        self.tree.debug_validate_id(child);

        let child_state = self.tree.get_state_unchecked_mut(child.index);
        assert!(child_state.parent.is_none());
        child_state.parent = Some(self.id.upcast());

        let prev_child = mem::replace(
            &mut self.state_mut().children[index],
            child,
        );
        self.update_without_propagate(Update::Children(ChildUpdate::Replaced(
            index,
        )));
        self.request_layout();

        prev_child
    }
}

impl<'a, T> WidgetMut<'a, T>
where
    T: ?Sized + AnyWidget,
{
    pub(crate) fn state(&self) -> &WidgetState {
        unsafe { &*self.state }
    }

    pub(crate) fn state_mut(&mut self) -> &mut WidgetState {
        unsafe { &mut *self.state }
    }

    pub(crate) fn enter_span(&self) -> tracing::span::EnteredSpan {
        self.state().tracing_span.clone().entered()
    }

    pub(crate) fn split(&mut self) -> (&mut Tree, &mut T, &mut WidgetState) {
        let widget = unsafe { &mut *self.widget };
        let state = unsafe { &mut *self.state };
        (self.tree, widget, state)
    }

    pub(crate) fn split_update_cx(&mut self) -> (&mut T, UpdateCx<'_>) {
        let (tree, widget, state) = self.split();
        let cx = UpdateCx { tree, state };
        (widget, cx)
    }

    pub(crate) fn split_layout_cx<'b>(
        &'b mut self,
        window: &'b Window,
    ) -> (&'b mut T, LayoutCx<'b>) {
        let (tree, widget, state) = self.split();
        let cx = LayoutCx {
            window,
            tree,
            state,
        };
        (widget, cx)
    }

    pub(crate) fn split_draw_cx<'b>(&'b mut self, window: &'b Window) -> (&'b mut T, DrawCx<'b>) {
        let (tree, widget, state) = self.split();
        let cx = DrawCx {
            window,
            tree,
            state,
        };
        (widget, cx)
    }

    pub(crate) fn update_without_propagate(&mut self, update: Update) {
        let (widget, mut cx) = self.split_update_cx();
        widget.update(&mut cx, update);
    }

    pub(crate) fn update_recursive(&mut self, update: Update) {
        let _span = self.enter_span();

        self.for_each_child(|child| {
            child.update_recursive(update.clone());
        });

        self.update_state();

        if let Update::WindowScaleChanged(..) = update
            && self.is_pixel_perfect()
        {
            self.state_mut().needs_layout = true;
        }

        self.update_without_propagate(update);
    }

    pub(crate) fn animate_recursive(&mut self, dt: Duration) {
        if self.is_stashed() || !self.state().needs_animate {
            return;
        }

        let _span = self.enter_span();

        self.for_each_child(|child| child.animate_recursive(dt));

        self.state_mut().needs_animate = false;
        self.update_state();

        let (widget, mut cx) = self.split_update_cx();
        widget.animate(&mut cx, dt);
    }

    pub(crate) fn layout_recursive(
        &mut self,
        window: &Window,
        painter: &mut dyn Painter,
        space: Space,
    ) -> Size {
        if self.state().is_stashing {
            self.state_mut().transform = Affine::IDENTITY;
            self.state_mut().size = space.min;
            return space.min;
        }

        if self.state().previous_space == Some(space) && !self.state().needs_layout {
            return self.state().size;
        }

        let _span = self.enter_span();
        self.state_mut().needs_layout = false;

        let mut size = {
            let (widget, mut cx) = self.split_layout_cx(window);
            widget.layout(&mut cx, painter, space)
        };

        if self.is_pixel_perfect() {
            size = size.ceil_to_scale(window.scale())
        }

        if !size.is_finite() {
            tracing::warn!(size = ?size, "size is not finite");
        }

        self.state_mut().size = size;
        self.state_mut().previous_space = Some(space);

        size
    }

    pub(crate) fn compose_recursive(&mut self, window: &Window, transform: Affine) {
        if self.is_stashed() {
            return;
        }

        if self.is_pixel_perfect() {
            let transform = &mut self.state_mut().transform;
            transform.offset = transform.offset.round_to_scale(window.scale());
        }

        let _span = self.enter_span();

        let transform = transform * self.transform();
        self.state_mut().global_transform = transform;

        self.for_each_child(|child| child.compose_recursive(window, transform));

        self.update_state();

        let (widget, mut cx) = self.split_update_cx();
        widget.compose(&mut cx);
    }

    pub(crate) fn draw_recursive(&mut self, window: &Window, canvas: &mut dyn Canvas) {
        if self.is_stashed() {
            return;
        }

        let _span = self.enter_span();

        canvas.transform(self.transform(), &mut |canvas| {
            if let Some(ref clip) = self.state().clip {
                canvas.clip(&clip.clone(), &mut |canvas| {
                    let (widget, mut cx) = self.split_draw_cx(window);
                    widget.draw(&mut cx, canvas);

                    self.for_each_child(|child| {
                        child.draw_recursive(window, canvas);
                    });
                });
            } else {
                let (widget, mut cx) = self.split_draw_cx(window);
                widget.draw(&mut cx, canvas);

                self.for_each_child(|child| {
                    child.draw_recursive(window, canvas);
                });
            }
        });
    }

    pub(crate) fn draw_over_recursive(&mut self, window: &Window, canvas: &mut dyn Canvas) {
        self.state_mut().needs_draw = false;

        if self.is_stashed() {
            return;
        }

        let _span = self.enter_span();

        canvas.transform(self.transform(), &mut |canvas| {
            if let Some(ref clip) = self.state().clip {
                canvas.clip(&clip.clone(), &mut |canvas| {
                    let (widget, mut cx) = self.split_draw_cx(window);
                    widget.draw_over(&mut cx, canvas);

                    self.for_each_child(|child| {
                        child.draw_over_recursive(window, canvas);
                    });
                });
            } else {
                let (widget, mut cx) = self.split_draw_cx(window);
                widget.draw_over(&mut cx, canvas);

                self.for_each_child(|child| {
                    child.draw_over_recursive(window, canvas);
                });
            }
        });
    }

    pub(crate) fn update_state(&mut self) {
        self.state_mut().reset();

        let children = mem::take(&mut self.state_mut().children);
        for &child in &children {
            let (tree, _, state) = self.split();
            let child_state = tree.get_state_unchecked(child.index);
            state.merge(child_state);
        }

        self.state_mut().children = children;
    }

    pub(crate) fn propagate_state(&mut self) {
        let mut current = Some(self.id.upcast());
        self.with_tree(|tree| {
            while let Some(id) = current {
                let mut widget = tree.get_mut(id).unwrap();
                widget.update_state();
                current = widget.parent_mut().map(|w| w.id);
            }
        });
    }

    pub(crate) fn with_tree<U>(&mut self, f: impl FnOnce(&mut Tree) -> U) -> U {
        let entry = &mut self.tree.entries[self.id.index as usize];
        entry.borrowed = false;

        let output = f(self.tree);

        let entry = &mut self.tree.entries[self.id.index as usize];
        entry.borrowed = true;

        output
    }

    pub(crate) fn set_hovered(&mut self, is_hovered: bool) {
        self.state_mut().is_hovered = is_hovered;
        self.update_without_propagate(Update::Hovered(is_hovered));
        self.propagate_state();
    }

    pub(crate) fn set_active(&mut self, is_active: bool) {
        self.state_mut().is_active = is_active;
        self.update_without_propagate(Update::Active(is_active));
        self.propagate_state();
    }

    pub(crate) fn set_focused(&mut self, is_focused: bool) {
        self.state_mut().is_focused = is_focused;
        self.update_without_propagate(Update::Focused(is_focused));
        self.propagate_state();
    }
}

impl<T> Deref for WidgetMut<'_, T>
where
    T: ?Sized + AnyWidget,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.widget }
    }
}

impl<T> DerefMut for WidgetMut<'_, T>
where
    T: ?Sized + AnyWidget,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.widget }
    }
}

impl<T> Drop for WidgetMut<'_, T>
where
    T: ?Sized + AnyWidget,
{
    fn drop(&mut self) {
        let entry = &mut self.tree.entries[self.id.index as usize];
        entry.borrowed = false;
    }
}
