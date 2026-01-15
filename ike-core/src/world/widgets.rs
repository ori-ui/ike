use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    fmt,
    marker::PhantomData,
};

use crate::{
    AnyWidgetId, LayoutCx, MutCx, RefCx, Size, Space, Widget, WidgetId, WidgetRef, WidgetState,
    widget::WidgetHierarchy,
    world::{WidgetMut, WorldState},
};

#[derive(Debug)]
pub enum GetError {
    InvalidId,
    InvalidType,
    InvalidChild,
    InvalidParent,
    Borrowed,
}

impl fmt::Display for GetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId => write!(f, "widget id is invalid"),
            Self::InvalidType => write!(f, "widget has the wrong type"),
            Self::InvalidChild => write!(f, "child is not valid for the parent"),
            Self::InvalidParent => write!(f, "widget is an orphan"),
            Self::Borrowed => write!(f, "widget already borrowed"),
        }
    }
}

pub(crate) struct Widgets {
    entities:  Entities,
    widgets:   Vec<RefCell<Box<dyn Widget>>>,
    states:    Vec<RefCell<WidgetState>>,
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

    pub fn insert<T>(&mut self, widget: T) -> WidgetId<T>
    where
        T: Widget,
    {
        let id = self.entities.alloc();
        let state = WidgetState::new::<T>(id);

        if id.index as usize >= self.widgets.len() {
            debug_assert_eq!(id.index as usize, self.widgets.len());

            self.widgets.push(RefCell::new(Box::new(widget)));
            self.states.push(RefCell::new(state));
            self.hierarchy.push(WidgetHierarchy::new::<T>());
        } else {
            let index = id.index as usize;
            self.widgets[index] = RefCell::new(Box::new(widget));
            self.states[index] = RefCell::new(state);
            self.hierarchy[index] = WidgetHierarchy::new::<T>();
        }

        WidgetId {
            index:      id.index,
            generation: id.generation,
            marker:     PhantomData,
        }
    }

    pub fn remove(&mut self, id: WidgetId) -> bool {
        if !self.entities.free(id) {
            // the entity could not be freed, because it never existed
            return false;
        }

        // insert a tombstone
        self.widgets[id.index as usize] = RefCell::new(Box::new(Tombstone));

        true
    }

    pub fn get<'a, T>(
        &'a self,
        world: &'a WorldState,
        id: WidgetId<T>,
    ) -> Result<WidgetRef<'a, T>, GetError>
    where
        T: AnyWidget + ?Sized,
    {
        if !self.entities.contains(id.upcast()) {
            return Err(GetError::InvalidId);
        }

        let widget = self.widgets[id.index as usize]
            .try_borrow()
            .map_err(|_| GetError::Borrowed)?;

        let widget = Ref::filter_map(widget, |widget| {
            T::downcast_ref(widget.as_ref())
        })
        .map_err(|_| GetError::InvalidType)?;

        let state = self.states[id.index as usize]
            .try_borrow()
            .map_err(|_| GetError::Borrowed)?;

        let hierarchy = &self.hierarchy[id.index as usize];

        let cx = RefCx {
            widgets: self,
            world,
            state,
            hierarchy,
        };

        Ok(WidgetRef { widget, cx })
    }

    pub fn get_mut<'a, T>(
        &'a self,
        world: &'a mut WorldState,
        id: WidgetId<T>,
    ) -> Result<WidgetMut<'a, T>, GetError>
    where
        T: AnyWidget + ?Sized,
    {
        if !self.entities.contains(id.upcast()) {
            return Err(GetError::InvalidId);
        }

        let widget = self.widgets[id.index as usize]
            .try_borrow_mut()
            .map_err(|_| GetError::Borrowed)?;

        let widget = RefMut::filter_map(widget, |widget| {
            T::downcast_mut(widget.as_mut())
        })
        .map_err(|_| GetError::InvalidType)?;

        let state = self.states[id.index as usize]
            .try_borrow_mut()
            .map_err(|_| GetError::Borrowed)?;

        let hierarchy = &self.hierarchy[id.index as usize];

        let cx = MutCx {
            widgets: self,
            world,
            state,
            hierarchy,
        };

        Ok(WidgetMut { widget, cx })
    }

    pub fn get_hierarchy(&self, id: WidgetId) -> Option<&WidgetHierarchy> {
        debug_assert!(
            self.entities.contains(id),
            "widget {id:?} not contained",
        );
        self.hierarchy.get(id.index as usize)
    }

    pub fn get_hierarchy_mut(&mut self, id: WidgetId) -> Option<&mut WidgetHierarchy> {
        debug_assert!(
            self.entities.contains(id),
            "widget {id:?} not contained",
        );
        self.hierarchy.get_mut(id.index as usize)
    }
}

struct Tombstone;

impl Widget for Tombstone {
    fn layout(&mut self, _cx: &mut LayoutCx<'_>, _space: Space) -> Size {
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
            entity.is_empty = false;
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
        let entity = &mut self.entities[widget.index as usize];

        if entity.generation != widget.generation {
            return false;
        }

        entity.is_empty = true;
        self.freed.push(widget.index);

        true
    }

    fn contains(&self, widget: WidgetId) -> bool {
        self.entities
            .get(widget.index as usize)
            .is_some_and(|e| e.generation == widget.generation && !e.is_empty)
    }
}

struct Entity {
    generation: u32,
    is_empty:   bool,
}

impl Entity {
    fn new() -> Self {
        Self {
            generation: 0,
            is_empty:   false,
        }
    }
}

pub trait AnyWidget: Widget {
    fn upcast_ref(&self) -> &dyn Widget;

    fn upcast_mut(&mut self) -> &mut dyn Widget;

    fn downcast_ref(widget: &dyn Widget) -> Option<&Self>;

    fn downcast_mut(widget: &mut dyn Widget) -> Option<&mut Self>;
}

impl AnyWidget for dyn Widget {
    fn upcast_ref(&self) -> &dyn Widget {
        self
    }

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
    fn upcast_ref(&self) -> &dyn Widget {
        self
    }

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
