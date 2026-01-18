use crate::{
    AnyWidget, AnyWidgetId, Color, GetError, Update, Widget, WidgetId, WidgetMut, WidgetRef,
    WindowId, WindowSizing, World, passes,
};

pub trait Builder {
    fn world(&self) -> &World;
    fn world_mut(&mut self) -> &mut World;

    fn get_widget<T>(&self, id: WidgetId<T>) -> Result<WidgetRef<'_, T>, GetError>
    where
        Self: Sized,
        T: ?Sized + AnyWidget,
    {
        let world = self.world();
        world.widgets.get(&world.state, id)
    }

    fn get_widget_mut<T>(&mut self, id: WidgetId<T>) -> Result<WidgetMut<'_, T>, GetError>
    where
        Self: Sized,
        T: ?Sized + AnyWidget,
    {
        let world = self.world_mut();
        world.widgets.get_mut(&mut world.state, id)
    }

    fn build_widget<T>(&mut self, widget: T) -> WidgetBuilder<'_, T>
    where
        Self: Sized,
        T: Widget,
    {
        let world = self.world_mut();
        let id = world.widgets.insert(widget);

        if let Ok(mut widget) = world.widget_mut(id.upcast()) {
            passes::update::widget(&mut widget, Update::Added);
            passes::hierarchy::propagate_down(widget.cx.widgets, id.upcast());
        }

        WidgetBuilder { world, id }
    }

    fn remove_widget(&mut self, widget: impl AnyWidgetId)
    where
        Self: Sized,
    {
        let world = self.world_mut();
        let widget = widget.upcast();

        passes::hierarchy::remove(world, widget);
    }

    fn add_child(&mut self, parent: impl AnyWidgetId, child: impl AnyWidgetId)
    where
        Self: Sized,
    {
        let index = self.children(parent).len();

        passes::hierarchy::insert_child(
            self.world_mut(),
            parent.upcast(),
            index,
            child.upcast(),
        );
    }

    fn insert_child(&mut self, parent: impl AnyWidgetId, index: usize, child: impl AnyWidgetId)
    where
        Self: Sized,
    {
        passes::hierarchy::insert_child(
            self.world_mut(),
            parent.upcast(),
            index,
            child.upcast(),
        );
    }

    fn set_child(&mut self, parent: impl AnyWidgetId, index: usize, child: impl AnyWidgetId)
    where
        Self: Sized,
    {
        passes::hierarchy::set_child(
            self.world_mut(),
            parent.upcast(),
            index,
            child.upcast(),
        );
    }

    fn swap_children(&mut self, parent: impl AnyWidgetId, index_a: usize, index_b: usize)
    where
        Self: Sized,
    {
        passes::hierarchy::swap_children(
            self.world_mut(),
            parent.upcast(),
            index_a,
            index_b,
        );
    }

    fn replace_widget(&mut self, widget: impl AnyWidgetId, other: impl AnyWidgetId)
    where
        Self: Sized,
    {
        passes::hierarchy::replace_widget(
            self.world_mut(),
            widget.upcast(),
            other.upcast(),
        );
    }

    fn remove_child(&mut self, parent: impl AnyWidgetId, index: usize) -> Option<WidgetId>
    where
        Self: Sized,
    {
        let world = self.world_mut();
        let parent = parent.upcast();

        passes::hierarchy::remove_child(world, parent, index)
    }

    fn set_stashed(&mut self, widget: impl AnyWidgetId, is_stashed: bool)
    where
        Self: Sized,
    {
        let world = self.world_mut();
        let widget = widget.upcast();

        if let Ok(mut widget) = world.widget_mut(widget) {
            passes::hierarchy::set_stashed(&mut widget, is_stashed);
        }
    }

    fn children(&self, parent: impl AnyWidgetId) -> &[WidgetId]
    where
        Self: Sized,
    {
        let parent = parent.upcast();

        self.world()
            .widgets
            .get_hierarchy(parent)
            .map_or(&[], |hierarchy| &hierarchy.children)
    }

    fn is_child(&self, parent: impl AnyWidgetId, child: impl AnyWidgetId) -> bool
    where
        Self: Sized,
    {
        let parent = parent.upcast();
        let child = child.upcast();

        self.get_widget(child)
            .is_ok_and(|child| child.cx.parent() == Some(parent))
    }

    fn set_window_base_layer(&mut self, window: WindowId, contents: impl AnyWidgetId)
    where
        Self: Sized,
    {
        (self.world_mut()).set_window_widget(window, contents.upcast());
    }

    fn set_window_title(&mut self, window: WindowId, title: String) {
        let state = &mut self.world_mut().state;
        state.set_window_title(window, title);
    }

    fn set_window_sizing(&mut self, window: WindowId, sizing: WindowSizing) {
        let state = &mut self.world_mut().state;
        state.set_window_sizing(window, sizing);
    }

    fn set_window_visible(&mut self, window: WindowId, visible: bool) {
        let state = &mut self.world_mut().state;
        state.set_window_visible(window, visible);
    }

    fn set_window_decorated(&mut self, window: WindowId, decorated: bool) {
        let state = &mut self.world_mut().state;
        state.set_window_decorated(window, decorated);
    }

    fn set_window_color(&mut self, window: WindowId, color: Color) {
        let state = &mut self.world_mut().state;
        state.set_window_color(window, color);
    }
}

impl<T> Builder for &mut T
where
    T: Builder,
{
    fn world(&self) -> &World {
        T::world(self)
    }

    fn world_mut(&mut self) -> &mut World {
        T::world_mut(self)
    }
}

pub struct WidgetBuilder<'a, T> {
    world: &'a mut World,
    id:    WidgetId<T>,
}

impl<'a, T> WidgetBuilder<'a, T> {
    pub fn with_child(self, child: impl AnyWidgetId) -> Self {
        self.world.add_child(self.id, child);
        self
    }

    pub fn finish(self) -> WidgetMut<'a, T>
    where
        T: Widget,
    {
        self.world
            .get_widget_mut(self.id)
            .expect("widget cannot not have been removed")
    }
}
