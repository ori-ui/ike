use std::{
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
};

use crate::{
    Canvas, DrawCx, Fonts, LayoutCx, Size, Space, Widget,
    widget::{AnyWidget, WidgetId, WidgetState},
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

    pub fn add_child<T, U>(&mut self, parent: &WidgetId<T>, child: WidgetId<U>)
    where
        T: ?Sized,
        U: ?Sized,
    {
        let child_state = self.widget_state_mut(child.index);
        child_state.parent = Some(parent.clone_internal().upcast());

        let parent_state = self.widget_state_mut(parent.index);
        parent_state.children.push(child.upcast());
    }

    pub fn swap_children<T>(&mut self, parent: &WidgetId<T>, index_a: usize, index_b: usize)
    where
        T: ?Sized,
    {
        let parent_state = self.widget_state_mut(parent.index);
        parent_state.children.swap(index_a, index_b);
    }

    pub fn replace_child<T, U>(&mut self, parent: &WidgetId<T>, index: usize, child: WidgetId<U>)
    where
        T: ?Sized,
        U: ?Sized,
    {
        let child_state = self.widget_state_mut(child.index);
        child_state.parent = Some(parent.clone_internal().upcast());

        let parent_state = self.widget_state_mut(parent.index);
        let prev_child = mem::replace(&mut parent_state.children[index], child.upcast());

        let entry = &mut self.entries[prev_child.index as usize];

        entry.state.take();
        entry.widget.take();
    }

    pub(crate) fn with<T: ?Sized, U>(
        &mut self,
        id: &WidgetId<T>,
        f: impl FnOnce(&mut Self, &mut dyn Widget, &mut WidgetState) -> U,
    ) -> U {
        let entry = &mut self.entries[id.index as usize];

        let mut widget = entry.widget.take();
        let mut state = entry.state.take();

        let output = {
            let widget = widget.as_mut().unwrap();
            let state = state.as_mut().unwrap();

            f(self, widget.as_mut(), state)
        };

        let entry = &mut self.entries[id.index as usize];

        entry.widget = widget;
        entry.state = state;

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

    pub fn needs_layout<T>(&self, id: &WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.widget_state(id.index).needs_layout
    }

    pub fn needs_draw<T>(&self, id: &WidgetId<T>) -> bool
    where
        T: ?Sized,
    {
        self.widget_state(id.index).needs_draw
    }

    pub fn layout(&mut self, fonts: &mut dyn Fonts, root: &WidgetId, size: Size) {
        self.with(root, |tree, widget, state| {
            state.needs_layout = false;

            let mut cx = LayoutCx { fonts, tree, state };
            let space = Space::new(Size::new(0.0, 0.0), size);

            let size = widget.layout(&mut cx, space);
            state.size = size;
        });
    }

    pub fn draw(&mut self, root: &WidgetId, canvas: &mut dyn Canvas) {
        self.with(root, |tree, widget, state| {
            canvas.transform(state.transform, &mut |canvas| {
                state.needs_draw = false;

                let mut cx = DrawCx { tree, state };

                widget.draw(&mut cx, canvas);

                for child in &state.children {
                    tree.draw(child, canvas);
                }
            });
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
