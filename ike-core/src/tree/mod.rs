mod widget_mut;
mod widget_ref;

use std::marker::PhantomData;

pub use widget_mut::WidgetMut;
pub use widget_ref::WidgetRef;

use crate::{
    LayoutCx, Painter, Size, Space, Update, Widget, WidgetId,
    widget::{AnyWidget, ChildUpdate, WidgetState},
};

pub struct Tree {
    entries: Vec<Entry>,
    free:    Vec<u32>,
}

struct Entry {
    generation: u32,
    borrowed:   bool,
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
    pub(crate) fn insert<T>(&mut self, widget: T) -> WidgetMut<'_, T>
    where
        T: Widget,
    {
        let id = if let Some(index) = self.free.pop() {
            let entry = &mut self.entries[index as usize];

            let old_widget = entry.widget;

            entry.generation += 1;
            entry.widget = Box::into_raw(Box::new(widget));
            unsafe { *entry.state = WidgetState::new::<T>() };
            let _ = unsafe { Box::from_raw(old_widget) };

            WidgetId {
                index,
                generation: entry.generation,
                marker: PhantomData,
            }
        } else {
            let index = self.entries.len() as u32;
            self.entries.push(Entry {
                generation: 0,
                borrowed:   false,
                widget:     Box::into_raw(Box::new(widget)),
                state:      Box::into_raw(Box::new(WidgetState::new::<T>())),
            });

            WidgetId {
                index,
                generation: 0,
                marker: PhantomData,
            }
        };

        let entry = &mut self.entries[id.index as usize];
        entry.borrowed = true;

        WidgetMut {
            id,
            widget: entry.widget.cast(),
            state: entry.state,
            tree: self,
        }
    }

    /// Remove a widget and all it's descendants.
    pub(crate) fn remove<T>(&mut self, id: WidgetId<T>)
    where
        T: ?Sized + AnyWidget,
    {
        let Some(mut widget) = self.get_mut(id) else {
            return;
        };

        if let Some(mut parent) = widget.parent_mut()
            && let Some(index) = parent.children().iter().position(|x| *x == id)
        {
            parent.state_mut().children.remove(index);
            parent.update_without_propagate(Update::Children(ChildUpdate::Removed(index)));
            parent.request_layout();
        }

        for &child in &unsafe { &*widget.state }.children {
            widget.tree.remove(child);
        }

        drop(widget);

        let entry = &mut self.entries[id.index as usize];

        let old_widget = entry.widget;

        entry.borrowed = false;
        entry.widget = Box::into_raw(Box::new(Tombstone));
        unsafe { *entry.state = WidgetState::new::<Tombstone>() };
        let _ = unsafe { Box::from_raw(old_widget) };

        self.free.push(id.index);
    }

    /// Get a reference to a widget.
    pub(crate) fn get<T>(&self, id: WidgetId<T>) -> Option<WidgetRef<'_, T>>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = self.entries.get(id.index as usize)?;

        if id.generation != entry.generation || entry.borrowed || !T::is(unsafe { &*entry.widget })
        {
            return None;
        }

        Some(WidgetRef {
            id,
            tree: self,
            widget: unsafe { &*T::downcast_ptr(entry.widget) },
            state: unsafe { &*entry.state },
        })
    }

    /// Get a mutable reference to a widget.
    pub(crate) fn get_mut<T>(&mut self, id: WidgetId<T>) -> Option<WidgetMut<'_, T>>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = self.entries.get_mut(id.index as usize)?;

        if id.generation != entry.generation || entry.borrowed || !T::is(unsafe { &*entry.widget })
        {
            return None;
        }

        Some(WidgetMut {
            id,
            widget: T::downcast_ptr(entry.widget),
            state: entry.state,
            tree: self,
        })
    }

    pub(crate) fn get_state_unchecked(&self, index: u32) -> &WidgetState {
        let entry = &self.entries[index as usize];
        assert!(!entry.borrowed);

        unsafe { &*entry.state }
    }

    pub(crate) fn get_state_unchecked_mut(&mut self, index: u32) -> &mut WidgetState {
        let entry = &mut self.entries[index as usize];
        assert!(!entry.borrowed);

        unsafe { &mut *entry.state }
    }

    pub(crate) fn debug_validate_id<T>(&self, id: WidgetId<T>)
    where
        T: ?Sized,
    {
        let entry = &self.entries[id.index as usize];
        debug_assert_eq!(id.generation, entry.generation);
    }

    pub(crate) fn needs_animate<T>(&self, id: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.debug_validate_id(id);
        self.get_state_unchecked(id.index).needs_animate
    }

    pub(crate) fn needs_draw<T>(&self, id: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.debug_validate_id(id);
        self.get_state_unchecked(id.index).needs_draw
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
