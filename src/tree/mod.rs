mod widget_mut;
mod widget_ref;

use std::marker::PhantomData;

pub use widget_mut::WidgetMut;
pub use widget_ref::WidgetRef;

use crate::{
    Update, Widget, WidgetId,
    widget::{AnyWidget, ChildUpdate, WidgetState},
};

pub struct Tree {
    entries: Vec<Entry>,
    free:    Vec<u32>,
}

struct Entry {
    generation: u32,
    widget:     Option<Box<dyn Widget>>,
    // WidgetState is very large, around 150 bytes, and moving it is slow. since it is moved on
    // every get_mut call, having it boxed is 2-3 times faster than not.
    state:      Option<Box<WidgetState>>,
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

            entry.generation += 1;

            WidgetId {
                index,
                generation: entry.generation,
                marker: PhantomData,
            }
        } else {
            let index = self.entries.len() as u32;
            self.entries.push(Entry {
                generation: 0,
                widget:     None,
                state:      None,
            });

            WidgetId {
                index,
                generation: 0,
                marker: PhantomData,
            }
        };

        WidgetMut {
            id,
            tree: self,
            widget: Some(Box::new(widget)),
            state: Some(Box::new(WidgetState::new::<T>())),
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
            parent.update(Update::Children(ChildUpdate::Removed(index)));
            parent.request_layout();
        }

        for &child in &widget.state.as_ref().unwrap().children {
            widget.tree.remove(child);
        }

        drop(widget);

        let entry = &mut self.entries[id.index as usize];

        let _ = entry.widget.take();
        let _ = entry.state.take();

        self.free.push(id.index);
    }

    /// Get a reference to a widget.
    pub(crate) fn get<T>(&self, id: WidgetId<T>) -> Option<WidgetRef<'_, T>>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = self.entries.get(id.index as usize)?;

        if id.generation != entry.generation {
            return None;
        }

        Some(WidgetRef {
            id,
            tree: self,
            widget: T::downcast_ref(entry.widget.as_deref()?),
            state: entry.state.as_ref()?,
        })
    }

    /// Get a mutable reference to a widget.
    pub(crate) fn get_mut<T>(&mut self, id: WidgetId<T>) -> Option<WidgetMut<'_, T>>
    where
        T: ?Sized + AnyWidget,
    {
        let entry = self.entries.get_mut(id.index as usize)?;

        if id.generation != entry.generation {
            return None;
        }

        let widget = entry.widget.take()?;
        let state = entry.state.take()?;

        Some(WidgetMut {
            id,
            tree: self,
            widget: Some(AnyWidget::downcast_boxed(widget).unwrap()),
            state: Some(state),
        })
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
