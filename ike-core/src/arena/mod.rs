mod widget_mut;
mod widget_ref;

use std::{
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
};

pub use widget_mut::WidgetMut;
pub use widget_ref::WidgetRef;

use crate::{
    AnyWidgetId, LayoutCx, Painter, Size, Space, Update, Widget, WidgetId,
    context::{MutCx, RefCx},
    root::RootState,
    widget::{AnyWidget, ChildUpdate, WidgetState},
};

pub struct Arena {
    // entries in the arena
    entries: Vec<Entry>,

    // list of freed entries
    free: Vec<u32>,
}

struct Entry {
    // current generation of this entry, a `WidgetId` with a `generation` different from this will
    // be invalid for this entry.
    generation: u32,

    // whether this entry is currently borrowed, used to ensure mutability rules are not broken.
    borrowed: Cell<bool>,
    state:    UnsafeCell<WidgetState>,
    widget:   Box<UnsafeCell<dyn Widget>>,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free:    Vec::new(),
        }
    }

    /// Add a widget to the arena.
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
                marker: PhantomData::<T>,
            };

            entry.widget = Box::new(UnsafeCell::new(widget));
            *entry.state.get_mut() = WidgetState::new::<T>(id.upcast());

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
                state:      UnsafeCell::new(state),
                widget:     Box::new(UnsafeCell::new(widget)),
            });

            id
        };

        let entry = &mut self.entries[id.index as usize];
        entry.borrowed.set(true);

        WidgetMut {
            widget: unsafe { &mut *entry.widget.get().cast() },

            cx: MutCx {
                root,
                state: unsafe { &mut *entry.state.get() },
                arena: self,
            },
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

        if let Some(mut parent) = widget.cx.get_parent_mut()
            && let Some(index) = parent.cx.state.children.iter().position(|x| *x == id)
        {
            let _ = parent.cx.state.children.remove(index);
            parent.update_without_propagate(Update::Children(ChildUpdate::Removed(
                index,
            )));
            parent.cx.request_layout();
        }

        for &child in &widget.cx.state.children {
            widget.cx.arena.remove(widget.cx.root, child);
        }

        drop(widget);

        let entry = &mut self.entries[id.index as usize];
        entry.widget = Box::new(UnsafeCell::new(Tombstone));

        self.free.push(id.index);
    }

    pub(crate) fn free_recursive(&mut self, id: WidgetId) -> bool {
        let Some(entry) = self.entries.get_mut(id.index as usize) else {
            return false;
        };

        if entry.borrowed.get() {
            return false;
        }

        entry.borrowed.set(true);
        let state = unsafe { &*entry.state.get() };
        let mut freed = true;

        for &child in &state.children {
            freed |= self.free_recursive(child);
        }

        if let Some(entry) = self.entries.get_mut(id.index as usize) {
            entry.borrowed.set(false);
            entry.widget = Box::new(UnsafeCell::new(Tombstone));
            self.free.push(id.index);

            freed
        } else {
            false
        }
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
            || !T::is(unsafe { &*entry.widget.get() })
        {
            return None;
        }

        entry.borrowed.set(true);

        Some(WidgetRef {
            widget: unsafe { &*T::downcast_ptr(entry.widget.get()) },

            cx: RefCx {
                root,
                arena: self,
                state: unsafe { &*entry.state.get() },
            },
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
            || !T::is(unsafe { &*entry.widget.get() })
        {
            return None;
        }

        entry.borrowed.set(true);

        Some(WidgetMut {
            widget: unsafe { &mut *T::downcast_ptr(entry.widget.get()) },

            cx: MutCx {
                root,
                state: unsafe { &mut *entry.state.get() },
                arena: self,
            },
        })
    }

    pub(crate) fn get_state(&self, index: u32) -> Option<&WidgetState> {
        let entry = match self.entries.get(index as usize) {
            Some(entry) => entry,
            None => {
                if cfg!(debug_assertions) {
                    panic!("tried to get state of non existent widget");
                } else {
                    return None;
                }
            }
        };

        match entry.borrowed.get() {
            false => unsafe { entry.state.get().as_ref() },
            true => {
                if cfg!(debug_assertions) {
                    panic!("tried to get state of borrowed widget");
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn get_state_mut(&mut self, index: u32) -> Option<&mut WidgetState> {
        let entry = match self.entries.get(index as usize) {
            Some(entry) => entry,
            None => {
                if cfg!(debug_assertions) {
                    panic!("tried to get state of non existent widget");
                } else {
                    return None;
                }
            }
        };

        match entry.borrowed.get() {
            false => unsafe { entry.state.get().as_mut() },
            true => {
                if cfg!(debug_assertions) {
                    panic!("tried to get state of borrowed widget");
                } else {
                    None
                }
            }
        }
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
    use crate::{Arena, LayoutCx, Painter, Root, Size, Space, Widget};

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
        let mut arena = Arena::new();

        let id = arena.insert(&mut root.state, TestWidget(0.0)).id();
        assert!(arena.get(&root.state, id).is_some());
    }

    #[test]
    fn get_mutability() {
        let mut root = Root::new(|_| {});
        let mut arena = Arena::new();

        let a = arena.insert(&mut root.state, TestWidget(0.0)).id();
        let b = arena.insert(&mut root.state, TestWidget(1.0)).id();

        let mut widget_mut = arena.get_mut(&mut root.state, a).unwrap();

        // getting a widget while it's borrowed mutable is not possible
        assert!(widget_mut.cx.get(a).is_none());
        assert!(widget_mut.cx.get_mut(a).is_none());

        // but getting others is
        assert!(widget_mut.cx.get(b).is_some());
        assert!(widget_mut.cx.get_mut(b).is_some());

        widget_mut.widget.0 = 10.0;
        drop(widget_mut);

        // getting a widget after the reference has been dropped is possible
        assert!(arena.get(&root.state, a).is_some());
    }
}
