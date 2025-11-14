use std::{
    any::Any,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    BuildCx, Canvas, DrawCx, LayoutCx, Size, Space,
    widget::{Widget, WidgetState},
};

pub struct WidgetId<T: ?Sized = dyn Widget> {
    index: u32,
    generation: u32,
    marker: PhantomData<T>,
}

impl<T: ?Sized> WidgetId<T> {
    pub fn cast<U: ?Sized>(self) -> WidgetId<U> {
        WidgetId {
            index: self.index,
            generation: self.generation,
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Clone for WidgetId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for WidgetId<T> {}

impl<T: ?Sized> PartialEq for WidgetId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T: ?Sized> Eq for WidgetId<T> {}

impl<T: ?Sized> Hash for WidgetId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T: ?Sized> fmt::Debug for WidgetId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}e{}", self.index, self.generation)
    }
}

pub struct Tree {
    entries: Vec<Entry>,
    free: Vec<WidgetId>,
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
        let index = self.entries.len() as u32;
        self.entries.push(Entry {
            generation: 0,
            widget: Some(Box::new(widget)),
            state: Some(WidgetState::new()),
        });

        WidgetId {
            index,
            generation: 0,
            marker: PhantomData,
        }
    }

    pub fn remove<T>(&mut self, id: WidgetId<T>) -> Option<T>
    where
        T: Widget,
    {
        let entry = &mut self.entries[id.index as usize];

        entry.state.take();
        let widget = entry.widget.take()?;
        Some(*(widget as Box<dyn Any>).downcast().ok()?)
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

    pub(crate) fn with<T: ?Sized, U>(
        &mut self,
        id: &WidgetId<T>,
        f: impl FnOnce(&mut Self, &mut dyn Widget, &mut WidgetState) -> U,
    ) -> U {
        let entry = &mut self.entries[id.index as usize];

        let mut widget = entry.widget.take();
        let mut state = entry.state.take();

        let output = if let (Some(widget), Some(state)) = (widget.as_mut(), state.as_mut()) {
            f(self, widget.as_mut(), state)
        } else {
            panic!();
        };

        let entry = &mut self.entries[id.index as usize];

        entry.widget = widget;
        entry.state = state;

        output
    }

    pub fn build(&mut self) -> BuildCx<'_> {
        BuildCx { tree: self }
    }

    pub fn layout(&mut self, root: &WidgetId, size: Size) {
        self.with(root, |tree, widget, state| {
            let mut cx = LayoutCx { tree, state };
            let space = Space::new(Size::new(0.0, 0.0), size);

            let size = widget.layout(&mut cx, space);
            state.size = size;
        });
    }

    pub fn draw(&mut self, root: &WidgetId, canvas: &mut dyn Canvas) {
        self.with(root, |tree, widget, state| {
            let mut cx = DrawCx { tree, state };

            widget.draw(&mut cx, canvas);
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

pub trait AnyWidget: Widget {
    fn downcast_ref(any: &dyn Widget) -> &Self;

    fn downcast_mut(any: &mut dyn Widget) -> &mut Self;
}

impl AnyWidget for dyn Widget {
    fn downcast_ref(any: &dyn Widget) -> &Self {
        any
    }

    fn downcast_mut(any: &mut dyn Widget) -> &mut Self {
        any
    }
}

impl<T> AnyWidget for T
where
    T: Widget,
{
    fn downcast_ref(any: &dyn Widget) -> &Self {
        (any as &dyn Any).downcast_ref().unwrap()
    }

    fn downcast_mut(any: &mut dyn Widget) -> &mut Self {
        (any as &mut dyn Any).downcast_mut().unwrap()
    }
}
