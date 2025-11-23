use crate::{AnyWidgetId, App, Widget, WidgetId, widget::AnyWidget};

pub trait BuildCx {
    fn app(&self) -> &App;
    fn app_mut(&mut self) -> &mut App;

    fn get<T>(&self, id: WidgetId<T>) -> &T
    where
        T: ?Sized + AnyWidget,
    {
        &self.app().tree[id]
    }

    fn get_mut<T>(&mut self, id: WidgetId<T>) -> &mut T
    where
        T: ?Sized + AnyWidget,
    {
        &mut self.app_mut().tree[id]
    }

    fn insert<T>(&mut self, widget: T) -> WidgetId<T>
    where
        Self: Sized,
        T: Widget,
    {
        self.app_mut().tree.insert(widget)
    }

    fn remove(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        self.app_mut().tree.remove(id.upcast());
    }

    fn add_child(&mut self, parent: impl AnyWidgetId, child: impl AnyWidgetId)
    where
        Self: Sized,
    {
        let child = child.upcast();
        self.app_mut().tree.add_child(parent, child);
        self.app_mut().tree.request_layout(child);
    }

    fn remove_child(&mut self, parent: impl AnyWidgetId, index: usize)
    where
        Self: Sized,
    {
        let parent = parent.upcast();
        let state = self.app().tree.get_state_unchecked(parent.index);
        let child = state.children[index];
        self.app_mut().tree.remove(child);
        self.app_mut().tree.request_layout(parent);
    }

    fn replace_child(&mut self, parent: impl AnyWidgetId, index: usize, child: impl AnyWidgetId)
    where
        Self: Sized,
    {
        let child = child.upcast();
        self.app_mut().tree.replace_child(parent, index, child);
        self.app_mut().tree.request_layout(child);
    }

    fn swap_children(&mut self, parent: impl AnyWidgetId, index_a: usize, index_b: usize)
    where
        Self: Sized,
    {
        let parent = parent.upcast();

        self.app_mut().tree.swap_children(parent, index_a, index_b);
        self.app_mut().tree.request_layout(parent);
    }

    fn children(&self, widget: impl AnyWidgetId) -> &[WidgetId] {
        &self
            .app()
            .tree
            .get_state_unchecked(widget.upcast().index)
            .children
    }

    fn is_parent(&self, parent: impl AnyWidgetId, child: impl AnyWidgetId) -> bool
    where
        Self: Sized,
    {
        let parent = parent.upcast();
        let child = child.upcast();

        self.app().tree.get_state_unchecked(child.index).parent == Some(parent)
    }

    fn request_animate(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        self.app_mut().tree.request_animate(id);
    }

    fn request_layout(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        self.app_mut().tree.request_layout(id);
    }

    fn request_draw(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        self.app_mut().tree.request_draw(id);
    }
}
