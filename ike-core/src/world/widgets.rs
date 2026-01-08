use std::{
    any::Any,
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
};

use crate::{
    LayoutCx, MutCx, Painter, RefCx, Size, Space, Widget, WidgetId, WidgetRef, WidgetState,
    widget::WidgetHierarchy,
    world::{WidgetMut, WorldState},
};

pub struct Widgets {
    entities:  Entities,
    widgets:   Vec<UnsafeCell<Box<dyn Widget>>>,
    states:    Vec<UnsafeCell<WidgetState>>,
    hierarchy: Vec<WidgetHierarchy>,
}

impl Default for Widgets {
    fn default() -> Self {
        Self::new()
    }
}

impl Widgets {
    pub const fn new() -> Self {
        Self {
            entities:  Entities::new(),
            widgets:   Vec::new(),
            states:    Vec::new(),
            hierarchy: Vec::new(),
        }
    }

    pub fn contains(&self, widget: WidgetId) -> bool {
        self.entities.contains(widget)
    }

    pub(crate) fn insert<T>(&mut self, widget: T) -> WidgetId<T>
    where
        T: Widget,
    {
        let id = self.entities.alloc();
        let state = WidgetState::new::<T>(id);

        if id.index as usize >= self.widgets.len() {
            debug_assert_eq!(id.index as usize, self.widgets.len());

            self.widgets.push(UnsafeCell::new(Box::new(widget)));
            self.states.push(UnsafeCell::new(state));
            self.hierarchy.push(WidgetHierarchy::new::<T>());
        } else {
            let index = id.index as usize;
            self.widgets[index] = UnsafeCell::new(Box::new(widget));
            self.states[index] = UnsafeCell::new(state);
            self.hierarchy[index] = WidgetHierarchy::new::<T>();
        }

        WidgetId {
            index:      id.index,
            generation: id.generation,
            marker:     PhantomData,
        }
    }

    pub(crate) fn remove(&mut self, id: WidgetId) -> bool {
        if !self.entities.free(id) {
            // the entity could not be freed, because it never existed
            return false;
        }

        // insert a tombstone
        self.widgets[id.index as usize] = UnsafeCell::new(Box::new(Tombstone));

        true
    }

    pub fn get<'a, T>(&'a self, world: &'a WorldState, id: WidgetId<T>) -> Option<WidgetRef<'a, T>>
    where
        T: AnyWidget + ?Sized,
    {
        if !self.borrow_ref(id) {
            return None;
        }

        let widget = unsafe { self.get_widget_unchecked(id.index as usize) };
        let state = unsafe { self.get_state_unchecked(id.index as usize) };
        let hierarchy = unsafe { self.get_hierarchy_unchecked(id.index as usize) };

        let Some(widget) = T::downcast_ref(widget) else {
            unsafe { self.release_ref(id.index as usize) };
            return None;
        };

        let cx = RefCx {
            widgets: self,
            world,
            state,
            hierarchy,
        };

        Some(WidgetRef { widget, cx })
    }

    pub fn get_mut<'a, T>(
        &'a self,
        world: &'a mut WorldState,
        id: WidgetId<T>,
    ) -> Option<WidgetMut<'a, T>>
    where
        T: AnyWidget + ?Sized,
    {
        if !self.borrow_mut(id) {
            return None;
        }

        let widget = unsafe { self.get_widget_unchecked_mut(id.index as usize) };
        let state = unsafe { self.get_state_unchecked_mut(id.index as usize) };
        let hierarchy = unsafe { self.get_hierarchy_unchecked(id.index as usize) };

        let Some(widget) = T::downcast_mut(widget) else {
            unsafe { self.release_mut(id.index as usize) };
            return None;
        };

        let cx = MutCx {
            widgets: self,
            world,
            state,
            hierarchy,
        };

        Some(WidgetMut { widget, cx })
    }

    pub fn borrow_ref<T>(&self, widget: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.entities.borrow_ref(widget)
    }

    pub fn borrow_mut<T>(&self, widget: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.entities.borrow_mut(widget)
    }

    /// # Safety
    /// - Widget at `index` must first have been borrowed with [`Self::borrow_mut`], and no
    ///   references may be held to the widget or state in question.
    /// - `index` must be a valid index of a widget.
    pub unsafe fn release_ref(&self, index: usize) {
        self.entities.release_ref(index);
    }

    /// # Safety
    /// - Widget at `index` must first have been borrowed with [`Self::borrow_mut`], and no mutable
    ///   reference may be held to the widget or state in question.
    /// - `index` must be a valid index of a widget.
    pub unsafe fn release_mut(&self, index: usize) {
        self.entities.release_mut(index);
    }

    /// # Safety
    /// - Widget at `index` must be borrowed by [`Self::borrow_ref`].
    /// - `index` must be a valid index of a widget.
    pub unsafe fn get_widget_unchecked(&self, index: usize) -> &dyn Widget {
        unsafe { &**self.widgets.get_unchecked(index).get() }
    }

    /// # Safety
    /// - Widget at `index` must be borrowed by [`Self::borrow_mut`].
    /// - `index` must be a valid index of a widget.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_widget_unchecked_mut(&self, index: usize) -> &mut dyn Widget {
        unsafe { &mut **self.widgets.get_unchecked(index).get() }
    }

    /// # Safety
    /// - Widget at `index` must be borrowed by [`Self::borrow_ref`].
    /// - `index` must be a valid index of a widget.
    pub unsafe fn get_state_unchecked(&self, index: usize) -> &WidgetState {
        unsafe { &*self.states.get_unchecked(index).get() }
    }

    /// # Safety
    /// - Widget at `index` must be borrowed by [`Self::borrow_mut`].
    /// - `index` must be a valid index of a widget.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_state_unchecked_mut(&self, index: usize) -> &mut WidgetState {
        unsafe { &mut *self.states.get_unchecked(index).get() }
    }

    pub(crate) fn get_hierarchy(&self, id: WidgetId) -> Option<&WidgetHierarchy> {
        debug_assert!(
            self.entities.contains(id),
            "widget {id:?} not contained",
        );
        self.hierarchy.get(id.index as usize)
    }

    pub(crate) fn get_hierarchy_mut(&mut self, id: WidgetId) -> Option<&mut WidgetHierarchy> {
        debug_assert!(
            self.entities.contains(id),
            "widget {id:?} not contained",
        );
        self.hierarchy.get_mut(id.index as usize)
    }

    /// # Safety
    /// - `index` must be a valid index of a widget.
    pub unsafe fn get_hierarchy_unchecked(&self, index: usize) -> &WidgetHierarchy {
        unsafe { self.hierarchy.get_unchecked(index) }
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

struct Entities {
    entities: Vec<Entity>,
    freed:    Vec<u32>,
}

impl Entities {
    const fn new() -> Self {
        Self {
            entities: Vec::new(),
            freed:    Vec::new(),
        }
    }

    fn alloc(&mut self) -> WidgetId {
        if let Some(index) = self.freed.pop() {
            let entity = &mut self.entities[index as usize];
            entity.set_empty(false);
            entity.generation += 1;

            return WidgetId {
                index,
                generation: entity.generation,
                marker: PhantomData,
            };
        }

        let index = self
            .entities
            .len()
            .try_into()
            .expect("no more than `u32::MAX` widgets should exist at once");

        self.entities.push(Entity::new());

        WidgetId {
            index,
            generation: 0,
            marker: PhantomData,
        }
    }

    fn free(&mut self, widget: WidgetId) -> bool {
        let entity = &self.entities[widget.index as usize];

        if entity.generation != widget.generation {
            return false;
        }

        entity.set_empty(true);
        self.freed.push(widget.index);

        true
    }

    fn contains(&self, widget: WidgetId) -> bool {
        self.entities
            .get(widget.index as usize)
            .is_some_and(|e| e.generation == widget.generation && !e.is_empty())
    }

    fn borrow_ref<T>(&self, widget: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.entities
            .get(widget.index as usize)
            .is_some_and(|e| e.generation == widget.generation && e.borrow_ref())
    }

    fn borrow_mut<T>(&self, widget: WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.entities
            .get(widget.index as usize)
            .is_some_and(|e| e.generation == widget.generation && e.borrow_mut())
    }

    fn release_ref(&self, index: usize) {
        self.entities[index].release_ref()
    }

    fn release_mut(&self, index: usize) {
        self.entities[index].release_mut()
    }
}

struct Entity {
    generation: u32,
    borrow:     Cell<u32>,
}

impl Entity {
    const EMPTY: u32 = u32::MAX;
    const MUT: u32 = u32::MAX - 1;

    const fn new() -> Self {
        Self {
            generation: 0,
            borrow:     Cell::new(0),
        }
    }

    fn is_empty(&self) -> bool {
        self.borrow.get() == Self::EMPTY
    }

    fn set_empty(&self, empty: bool) {
        match empty {
            true => {
                debug_assert_eq!(self.borrow.get(), 0);
                self.borrow.set(Self::EMPTY);
            }

            false => {
                debug_assert_eq!(self.borrow.get(), Self::EMPTY);
                self.borrow.set(0);
            }
        }
    }

    fn borrow_ref(&self) -> bool {
        let count = self.borrow.get();

        if count < Self::MUT {
            self.borrow.update(|x| x + 1);
            true
        } else {
            false
        }
    }

    fn borrow_mut(&self) -> bool {
        let count = self.borrow.get();

        if count == 0 {
            self.borrow.set(Self::MUT);
            true
        } else {
            false
        }
    }

    fn release_ref(&self) {
        debug_assert!(self.borrow.get() > 0);
        debug_assert_ne!(self.borrow.get(), Self::EMPTY);
        debug_assert_ne!(self.borrow.get(), Self::MUT);

        self.borrow.update(|x| x - 1);
    }

    fn release_mut(&self) {
        debug_assert_ne!(self.borrow.get(), Self::EMPTY);
        debug_assert_eq!(self.borrow.get(), Self::MUT);

        self.borrow.set(0);
    }
}

pub trait AnyWidget: Widget {
    fn upcast_mut(&mut self) -> &mut dyn Widget;

    fn downcast_ref(widget: &dyn Widget) -> Option<&Self>;

    fn downcast_mut(widget: &mut dyn Widget) -> Option<&mut Self>;
}

impl AnyWidget for dyn Widget {
    fn upcast_mut(&mut self) -> &mut dyn Widget {
        self
    }

    fn downcast_ref(widget: &dyn Widget) -> Option<&Self> {
        Some(widget)
    }

    fn downcast_mut(widget: &mut dyn Widget) -> Option<&mut Self> {
        Some(widget)
    }
}

impl<T> AnyWidget for T
where
    T: Widget,
{
    fn upcast_mut(&mut self) -> &mut dyn Widget {
        self
    }

    fn downcast_ref(widget: &dyn Widget) -> Option<&Self> {
        (widget as &dyn Any).downcast_ref()
    }

    fn downcast_mut(widget: &mut dyn Widget) -> Option<&mut Self> {
        (widget as &mut dyn Any).downcast_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_borrow_release() {
        let entity = Entity::new();

        assert!(entity.borrow_ref());
        assert!(entity.borrow_ref());
        assert!(!entity.borrow_mut());

        entity.release_ref();
        entity.release_ref();

        assert!(entity.borrow_mut());
        assert!(!entity.borrow_ref());

        entity.release_mut();

        assert!(entity.borrow_ref());
    }
}
