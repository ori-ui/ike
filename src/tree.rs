use std::{
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
};

use crate::{
    AnyWidgetId, Fonts, LayoutCx, Size, Space, Widget,
    context::UpdateCx,
    widget::{AnyWidget, Update, WidgetId, WidgetState},
};

pub struct Tree {
    entries: Vec<Entry>,
    free: Vec<u32>,
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
            free: Vec::new(),
        }
    }

    pub fn insert<T>(&mut self, widget: T) -> WidgetId<T>
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
            widget: Some(Box::new(widget)),
            state: Some(WidgetState::new::<T>()),
        });

        WidgetId {
            index,
            generation: 0,
            marker: PhantomData,
        }
    }

    pub fn remove<T>(&mut self, id: WidgetId<T>)
    where
        T: ?Sized + Widget,
    {
        let entry = &mut self.entries[id.index as usize];

        let _widget = entry.widget.take();

        if let Some(state) = entry.state.take() {
            if let Some(parent) = state.parent {
                let parent_state = self.widget_state_mut(parent.index);

                if let Some(i) = parent_state.children.iter().position(|x| *x == id) {
                    parent_state.children.remove(i);

                    self.with_entry(parent, |tree, widget, state| {
                        let mut cx = UpdateCx { tree, state };

                        widget.update(&mut cx, Update::ChildRemoved(i));
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

    pub fn get<T>(&self, id: WidgetId<T>) -> Option<&T>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = &self.entries[id.index as usize];

        if id.generation != entry.generation {
            return None;
        }

        Some(T::downcast_ref(entry.widget.as_deref()?))
    }

    pub fn get_mut<T>(&mut self, id: WidgetId<T>) -> Option<&mut T>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = &mut self.entries[id.index as usize];

        if id.generation != entry.generation {
            return None;
        }

        Some(T::downcast_mut(entry.widget.as_deref_mut()?))
    }

    pub fn add_child(&mut self, parent: impl AnyWidgetId, child: impl AnyWidgetId) {
        let parent = parent.upcast();
        let child = child.upcast();

        self.debug_validate_id(parent);
        self.debug_validate_id(child);

        let child_state = self.widget_state_mut(child.index);
        child_state.parent = Some(parent.upcast());

        let parent_state = self.widget_state_mut(parent.index);

        let index = parent_state.children.len();
        parent_state.children.push(child.upcast());

        self.with_entry(parent, |tree, widget, state| {
            let mut cx = UpdateCx { tree, state };

            widget.update(&mut cx, Update::ChildAdded(index));
        });

        self.request_layout(parent);
    }

    pub fn swap_children(&mut self, parent: impl AnyWidgetId, index_a: usize, index_b: usize) {
        let parent = parent.upcast();

        self.debug_validate_id(parent);

        let parent_state = self.widget_state_mut(parent.index);
        parent_state.children.swap(index_a, index_b);

        self.with_entry(parent, |tree, widget, state| {
            let mut cx = UpdateCx { tree, state };

            widget.update(&mut cx, Update::ChildrenSwapped(index_a, index_b));
        });

        self.request_layout(parent);
    }

    pub fn replace_child(
        &mut self,
        parent: impl AnyWidgetId,
        index: usize,
        child: impl AnyWidgetId,
    ) {
        let parent = parent.upcast();
        let child = child.upcast();

        self.debug_validate_id(parent);
        self.debug_validate_id(child);

        let child_state = self.widget_state_mut(child.index);
        child_state.parent = Some(parent.upcast());

        let parent_state = self.widget_state_mut(parent.index);
        let prev_child = mem::replace(&mut parent_state.children[index], child.upcast());

        self.remove(prev_child);

        self.with_entry(parent, |tree, widget, state| {
            let mut cx = UpdateCx { tree, state };

            widget.update(&mut cx, Update::ChildReplaced(index));
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

    pub(crate) fn widget_state(&self, index: u32) -> &WidgetState {
        let entry = &self.entries[index as usize];
        entry.state.as_ref().unwrap()
    }

    pub(crate) fn widget_state_mut(&mut self, index: u32) -> &mut WidgetState {
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

    pub(crate) fn propagate_state(&mut self, id: impl AnyWidgetId) {
        let parent = self.with_entry(id.upcast(), |tree, _, state| {
            if let Some(parent) = state.parent {
                let parent_state = tree.widget_state_mut(parent.index);
                parent_state.merge(state);
            }

            state.parent
        });

        if let Some(parent) = parent {
            self.propagate_state(parent);
        }
    }

    pub(crate) fn request_layout(&mut self, id: impl AnyWidgetId) {
        let id = id.upcast();
        self.debug_validate_id(id);

        let state = self.widget_state_mut(id.index);
        state.needs_layout = true;
        state.needs_draw = true;

        self.propagate_state(id);
    }

    pub(crate) fn request_draw(&mut self, id: impl AnyWidgetId) {
        let id = id.upcast();
        self.debug_validate_id(id);

        let state = self.widget_state_mut(id.index);
        state.needs_draw = true;

        self.propagate_state(id);
    }

    pub fn needs_layout<T>(&self, id: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.debug_validate_id(id);
        self.widget_state(id.index).needs_layout
    }

    pub fn needs_draw<T>(&self, id: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.debug_validate_id(id);
        self.widget_state(id.index).needs_draw
    }

    pub fn layout(&mut self, fonts: &mut dyn Fonts, root: WidgetId, size: Size) {
        self.with_entry(root, |tree, widget, state| {
            state.needs_layout = false;

            let mut cx = LayoutCx { fonts, tree, state };
            let space = Space::new(Size::new(0.0, 0.0), size);

            let size = widget.layout(&mut cx, space);
            state.size = size;
        });
    }
}

struct Entry {
    generation: u32,
    widget: Option<Box<dyn Widget>>,
    state: Option<WidgetState>,
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
