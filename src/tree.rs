use std::{
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
};

use crate::{
    AnyWidgetId, Widget,
    context::UpdateCx,
    widget::{AnyWidget, ChildUpdate, Update, WidgetId, WidgetState},
};

pub struct Tree {
    entries: Vec<Entry>,
    free:    Vec<u32>,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Tree {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free:    Vec::new(),
        }
    }

    /// Add a widget to the tree.
    ///
    /// Will use a previously freed entry.
    pub(crate) fn insert<T>(&mut self, widget: T) -> WidgetId<T>
    where
        T: Widget,
    {
        if let Some(index) = self.free.pop() {
            let entry = &mut self.entries[index as usize];

            entry.generation += 1;
            entry.state = Some(WidgetState::new::<T>());
            entry.widget = Some(Box::new(widget));

            return WidgetId {
                index,
                generation: entry.generation,
                marker: PhantomData,
            };
        }

        let index = self.entries.len() as u32;
        self.entries.push(Entry {
            generation: 0,
            widget:     Some(Box::new(widget)),
            state:      Some(WidgetState::new::<T>()),
        });

        WidgetId {
            index,
            generation: 0,
            marker: PhantomData,
        }
    }

    /// Remove a widget and all it's descendants.
    pub(crate) fn remove<T>(&mut self, id: WidgetId<T>)
    where
        T: ?Sized + Widget,
    {
        let entry = &mut self.entries[id.index as usize];

        if entry.generation != id.generation {
            return;
        }

        let _widget = entry.widget.take();

        if let Some(state) = entry.state.take() {
            if let Some(parent) = state.parent {
                let parent_state = self.get_state_unchecked_mut(parent.index);

                if let Some(index) = parent_state.children.iter().position(|x| *x == id) {
                    parent_state.children.remove(index);

                    self.with_entry(parent, |tree, widget, state| {
                        let mut cx = UpdateCx { tree, state };

                        let update = Update::Children(ChildUpdate::Removed(index));
                        widget.update(&mut cx, update);
                    });

                    self.request_layout(parent);
                }
            }

            for child in state.children {
                self.remove(child);
            }
        }

        self.free.push(id.index);
    }

    /// Get a reference to a widget.
    pub(crate) fn get<T>(&self, id: WidgetId<T>) -> Option<&T>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = &self.entries[id.index as usize];

        if id.generation != entry.generation {
            return None;
        }

        Some(T::downcast_ref(entry.widget.as_deref()?))
    }

    /// Get a mutable reference to a widget.
    pub(crate) fn get_mut<T>(&mut self, id: WidgetId<T>) -> Option<&mut T>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = &mut self.entries[id.index as usize];

        if id.generation != entry.generation {
            return None;
        }

        Some(T::downcast_mut(entry.widget.as_deref_mut()?))
    }

    /// Add a child to widget.
    pub(crate) fn add_child(&mut self, parent: impl AnyWidgetId, child: impl AnyWidgetId) {
        let parent = parent.upcast();
        let child = child.upcast();

        self.debug_validate_id(parent);
        self.debug_validate_id(child);

        let child_state = self.get_state_unchecked_mut(child.index);
        child_state.parent = Some(parent.upcast());

        let parent_state = self.get_state_unchecked_mut(parent.index);

        let index = parent_state.children.len();
        parent_state.children.push(child.upcast());

        self.with_entry(parent, |tree, widget, state| {
            let mut cx = UpdateCx { tree, state };

            let update = Update::Children(ChildUpdate::Added(index));
            widget.update(&mut cx, update);
        });

        self.request_layout(parent);
    }

    /// Swap two children of a widget.
    pub(crate) fn swap_children(
        &mut self,
        parent: impl AnyWidgetId,
        index_a: usize,
        index_b: usize,
    ) {
        let parent = parent.upcast();

        self.debug_validate_id(parent);

        let parent_state = self.get_state_unchecked_mut(parent.index);
        parent_state.children.swap(index_a, index_b);

        self.with_entry(parent, |tree, widget, state| {
            let mut cx = UpdateCx { tree, state };

            let update = Update::Children(ChildUpdate::Swapped(index_a, index_b));
            widget.update(&mut cx, update);
        });

        self.request_layout(parent);
    }

    /// Replace the child of a widget with another.
    pub(crate) fn replace_child(
        &mut self,
        parent: impl AnyWidgetId,
        index: usize,
        child: impl AnyWidgetId,
    ) {
        let parent = parent.upcast();
        let child = child.upcast();

        self.debug_validate_id(parent);
        self.debug_validate_id(child);

        let child_state = self.get_state_unchecked_mut(child.index);
        assert!(child_state.parent.is_none());
        child_state.parent = Some(parent.upcast());

        let parent_state = self.get_state_unchecked_mut(parent.index);
        let prev_child = mem::replace(&mut parent_state.children[index], child.upcast());

        self.remove(prev_child);

        self.with_entry(parent, |tree, widget, state| {
            let mut cx = UpdateCx { tree, state };

            let update = Update::Children(ChildUpdate::Replaced(index));
            widget.update(&mut cx, update);
        });

        self.request_layout(child);
    }

    pub(crate) fn take_entry<T>(
        &mut self,
        id: WidgetId<T>,
    ) -> Option<(Box<dyn Widget>, WidgetState)>
    where
        T: ?Sized,
    {
        self.debug_validate_id(id);

        let entry = &mut self.entries[id.index as usize];
        Some((entry.widget.take()?, entry.state.take()?))
    }

    pub(crate) fn insert_entry<T>(
        &mut self,
        id: WidgetId<T>,
        widget: Box<dyn Widget>,
        state: WidgetState,
    ) where
        T: ?Sized,
    {
        self.debug_validate_id(id);

        let entry = &mut self.entries[id.index as usize];
        entry.widget = Some(widget);
        entry.state = Some(state);
    }

    pub(crate) fn with_entry<T, U>(
        &mut self,
        id: WidgetId<T>,
        f: impl FnOnce(&mut Self, &mut dyn Widget, &mut WidgetState) -> U,
    ) -> U
    where
        T: ?Sized,
    {
        let (mut widget, mut state) = self.take_entry(id).unwrap();
        let output = { f(self, widget.as_mut(), &mut state) };
        self.insert_entry(id, widget, state);
        output
    }

    pub(crate) fn get_state_unchecked(&self, index: u32) -> &WidgetState {
        let entry = &self.entries[index as usize];
        entry.state.as_ref().unwrap()
    }

    pub(crate) fn get_state_unchecked_mut(&mut self, index: u32) -> &mut WidgetState {
        let entry = &mut self.entries[index as usize];
        entry.state.as_mut().unwrap()
    }

    pub(crate) fn debug_validate_id<T>(&self, id: WidgetId<T>)
    where
        T: ?Sized,
    {
        let entry = &self.entries[id.index as usize];
        debug_assert_eq!(id.generation, entry.generation);
    }

    pub(crate) fn update_state(&mut self, id: impl AnyWidgetId) {
        self.with_entry(id.upcast(), |tree, _widget, state| {
            state.reset();

            let children = mem::take(&mut state.children);
            for &child in &children {
                let child_state = tree.get_state_unchecked(child.index);
                state.merge(child_state);
            }

            state.children = children;
        });
    }

    pub(crate) fn state_changed(&mut self, id: impl AnyWidgetId) {
        let mut current = Some(id.upcast());

        while let Some(id) = current {
            self.update_state(id);

            let state = self.get_state_unchecked(id.index);
            current = state.parent;
        }
    }

    pub(crate) fn set_hovered(&mut self, id: impl AnyWidgetId, is_hovered: bool) {
        self.with_entry(id.upcast(), |tree, widget, state| {
            state.is_hovered = is_hovered;
            state.has_hovered = is_hovered;

            let mut cx = UpdateCx { tree, state };

            widget.update(&mut cx, Update::Hovered(is_hovered));
        });

        self.state_changed(id);
    }

    pub(crate) fn set_active(&mut self, id: impl AnyWidgetId, is_active: bool) {
        self.with_entry(id.upcast(), |tree, widget, state| {
            state.is_active = is_active;
            state.has_active = is_active;

            let mut cx = UpdateCx { tree, state };

            widget.update(&mut cx, Update::Active(is_active));
        });

        self.state_changed(id);
    }

    pub(crate) fn set_focused(&mut self, id: impl AnyWidgetId, is_focused: bool) {
        self.with_entry(id.upcast(), |tree, widget, state| {
            state.is_focused = is_focused;
            state.has_focused = is_focused;

            let mut cx = UpdateCx { tree, state };

            widget.update(&mut cx, Update::Focused(is_focused));
        });

        self.state_changed(id);
    }

    pub(crate) fn request_animate(&mut self, id: impl AnyWidgetId) {
        let id = id.upcast();
        self.debug_validate_id(id);

        let state = self.get_state_unchecked_mut(id.index);
        state.needs_animate = true;

        self.state_changed(id);
    }

    pub(crate) fn request_layout(&mut self, id: impl AnyWidgetId) {
        let id = id.upcast();
        self.debug_validate_id(id);

        let state = self.get_state_unchecked_mut(id.index);
        state.needs_layout = true;
        state.needs_draw = true;

        self.state_changed(id);
    }

    pub(crate) fn request_draw(&mut self, id: impl AnyWidgetId) {
        let id = id.upcast();
        self.debug_validate_id(id);

        let state = self.get_state_unchecked_mut(id.index);
        state.needs_draw = true;

        self.state_changed(id);
    }

    pub(crate) fn needs_animate<T>(&self, id: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.debug_validate_id(id);
        self.get_state_unchecked(id.index).needs_animate
    }

    pub(crate) fn needs_layout<T>(&self, id: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.debug_validate_id(id);
        self.get_state_unchecked(id.index).needs_layout
    }

    pub(crate) fn needs_draw<T>(&self, id: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.debug_validate_id(id);
        self.get_state_unchecked(id.index).needs_draw
    }
}

struct Entry {
    generation: u32,
    widget:     Option<Box<dyn Widget>>,
    state:      Option<WidgetState>,
}

impl<T> Index<WidgetId<T>> for Tree
where
    T: ?Sized + AnyWidget,
{
    type Output = T;

    #[track_caller]
    fn index(&self, id: WidgetId<T>) -> &Self::Output {
        self.get(id).unwrap()
    }
}

impl<T> IndexMut<WidgetId<T>> for Tree
where
    T: ?Sized + AnyWidget,
{
    #[track_caller]
    fn index_mut(&mut self, id: WidgetId<T>) -> &mut Self::Output {
        self.get_mut(id).unwrap()
    }
}
