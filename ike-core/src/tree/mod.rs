mod widget_mut;
mod widget_ref;

use std::{cell::Cell, marker::PhantomData};

pub use widget_mut::WidgetMut;
pub use widget_ref::WidgetRef;

use crate::{
    AnyWidgetId, LayoutCx, Painter, Size, Space, Update, Widget, WidgetId,
    root::RootState,
    widget::{AnyWidget, ChildUpdate, WidgetState},
};

pub struct Tree {
    entries: Vec<Entry>,
    free:    Vec<u32>,
}

struct Entry {
    generation: u32,
    borrowed:   Cell<bool>,
    widget:     *mut dyn Widget,
    state:      *mut WidgetState,
}

impl Drop for Entry {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.widget);
            let _ = Box::from_raw(self.state);
        }
    }
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
    pub(crate) fn insert<'a, T>(
        &'a mut self,
        root: &'a mut RootState,
        widget: T,
    ) -> WidgetMut<'a, T>
    where
        T: Widget,
    {
        let id = if let Some(index) = self.free.pop() {
            let entry = &mut self.entries[index as usize];
            entry.generation += 1;

            let id = WidgetId {
                index,
                generation: entry.generation,
                marker: PhantomData,
            };

            let old_widget = entry.widget;

            entry.widget = Box::into_raw(Box::new(widget));
            unsafe { *entry.state = WidgetState::new::<T>(id.upcast()) };
            let _ = unsafe { Box::from_raw(old_widget) };

            id
        } else {
            let index = self.entries.len() as u32;

            let id = WidgetId {
                index,
                generation: 0,
                marker: PhantomData,
            };

            let state = WidgetState::new::<T>(id.upcast());
            self.entries.push(Entry {
                generation: 0,
                borrowed:   Cell::new(false),
                widget:     Box::into_raw(Box::new(widget)),
                state:      Box::into_raw(Box::new(state)),
            });

            id
        };

        let entry = &mut self.entries[id.index as usize];
        entry.borrowed.set(true);

        WidgetMut {
            id,
            root,
            widget: entry.widget.cast(),
            state: entry.state,
            tree: self,
        }
    }

    /// Remove a widget and all it's descendants.
    pub(crate) fn remove<T>(&mut self, root: &mut RootState, id: WidgetId<T>)
    where
        T: ?Sized + AnyWidget,
    {
        let Some(mut widget) = self.get_mut(root, id) else {
            return;
        };

        if let Some(mut parent) = widget.parent_mut()
            && let Some(index) = parent.children().iter().position(|x| *x == id)
        {
            parent.state_mut().children.remove(index);
            parent.update_without_propagate(Update::Children(ChildUpdate::Removed(
                index,
            )));
            parent.request_layout();
        }

        for &child in &unsafe { &*widget.state }.children {
            widget.tree.remove(widget.root, child);
        }

        drop(widget);

        let entry = &mut self.entries[id.index as usize];

        let old_widget = entry.widget;

        entry.widget = Box::into_raw(Box::new(Tombstone));
        let _ = unsafe { Box::from_raw(old_widget) };

        self.free.push(id.index);
    }

    /// Get a reference to a widget.
    pub(crate) fn get<'a, T>(
        &'a self,
        root: &'a RootState,
        id: WidgetId<T>,
    ) -> Option<WidgetRef<'a, T>>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = self.entries.get(id.index as usize)?;

        if id.generation != entry.generation
            || entry.borrowed.get()
            || !T::is(unsafe { &*entry.widget })
        {
            return None;
        }

        entry.borrowed.set(true);

        Some(WidgetRef {
            id,
            root,
            tree: self,
            widget: unsafe { &*T::downcast_ptr(entry.widget) },
            state: unsafe { &*entry.state },
        })
    }

    /// Get a mutable reference to a widget.
    pub(crate) fn get_mut<'a, T>(
        &'a mut self,
        root: &'a mut RootState,
        id: WidgetId<T>,
    ) -> Option<WidgetMut<'a, T>>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = self.entries.get_mut(id.index as usize)?;

        if id.generation != entry.generation
            || entry.borrowed.get()
            || !T::is(unsafe { &*entry.widget })
        {
            return None;
        }

        entry.borrowed.set(true);

        Some(WidgetMut {
            id,
            root,
            widget: T::downcast_ptr(entry.widget),
            state: entry.state,
            tree: self,
        })
    }

    pub(crate) fn get_state_unchecked(&self, index: u32) -> &WidgetState {
        let entry = &self.entries[index as usize];
        assert!(!entry.borrowed.get());

        unsafe { &*entry.state }
    }

    pub(crate) fn get_state_unchecked_mut(&mut self, index: u32) -> &mut WidgetState {
        let entry = &mut self.entries[index as usize];
        assert!(!entry.borrowed.get());

        unsafe { &mut *entry.state }
    }

    pub(crate) fn debug_validate_id<T>(&self, id: WidgetId<T>)
    where
        T: ?Sized,
    {
        let entry = &self.entries[id.index as usize];
        debug_assert_eq!(id.generation, entry.generation);
    }
}

struct Tombstone;

impl Widget for Tombstone {
    fn layout(
        &mut self,
        _cx: &mut LayoutCx<'_>,
        _painter: &mut dyn Painter,
        _space: Space,
    ) -> Size {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{LayoutCx, Painter, Root, Size, Space, Tree, Widget};

    struct TestWidget(f32);

    impl Widget for TestWidget {
        fn layout(
            &mut self,
            _cx: &mut LayoutCx<'_>,
            _painter: &mut dyn Painter,
            space: Space,
        ) -> Size {
            space.min
        }
    }

    #[test]
    fn insert_and_get() {
        let mut root = Root::new(|_| {});
        let mut tree = Tree::new();

        let id = tree.insert(&mut root.state, TestWidget(0.0)).id();
        assert!(tree.get(&root.state, id).is_some());
    }

    #[test]
    fn get_mutability() {
        let mut root = Root::new(|_| {});
        let mut tree = Tree::new();

        let a = tree.insert(&mut root.state, TestWidget(0.0)).id();
        let b = tree.insert(&mut root.state, TestWidget(1.0)).id();

        let mut widget = tree.get_mut(&mut root.state, a).unwrap();

        // getting a widget while it's borrowed mutable is not possible
        assert!(widget.get(a).is_none());
        assert!(widget.get_mut(a).is_none());

        // but getting others is
        assert!(widget.get(b).is_some());
        assert!(widget.get_mut(b).is_some());

        widget.0 = 10.0;
        drop(widget);

        // getting a widget after the reference has been dropped is possible
        assert!(tree.get(&root.state, a).is_some());
    }
}
