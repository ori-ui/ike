use crate::{
    AnyWidgetId, Color, Widget, WidgetId, WindowId, WindowSizing, World,
    arena::{WidgetMut, WidgetRef},
    widget::AnyWidget,
};

pub trait BuildCx {
    fn world(&self) -> &World;
    fn world_mut(&mut self) -> &mut World;

    fn get_widget<T>(&self, id: WidgetId<T>) -> Option<WidgetRef<'_, T>>
    where
        Self: Sized,
        T: ?Sized + AnyWidget,
    {
        let world = self.world();
        world.arena.get(&world.state, id)
    }

    fn get_widget_mut<T>(&mut self, id: WidgetId<T>) -> Option<WidgetMut<'_, T>>
    where
        Self: Sized,
        T: ?Sized + AnyWidget,
    {
        let world = self.world_mut();
        world.arena.get_mut(&mut world.state, id)
    }

    fn insert_widget<T>(&mut self, widget: T) -> WidgetMut<'_, T>
    where
        Self: Sized,
        T: Widget,
    {
        let world = self.world_mut();
        world.arena.insert(&mut world.state, widget)
    }

    fn remove_widget(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        let world = self.world_mut();
        world.arena.remove(&mut world.state, id.upcast());
    }

    fn is_parent(&self, parent: impl AnyWidgetId, child: impl AnyWidgetId) -> bool
    where
        Self: Sized,
    {
        let parent = parent.upcast();
        let child = child.upcast();

        self.world()
            .arena
            .get_state(child.index)
            .is_some_and(|child| child.parent == Some(parent))
    }

    fn set_window_layer(&mut self, window: WindowId, contents: impl AnyWidgetId)
    where
        Self: Sized,
    {
        self.world_mut().set_window_layer(window, contents.upcast());
    }

    fn set_window_title(&mut self, window: WindowId, title: String) {
        self.world_mut().set_window_title(window, title);
    }

    fn set_window_sizing(&mut self, window: WindowId, sizing: WindowSizing) {
        self.world_mut().set_window_sizing(window, sizing);
    }

    fn set_window_visible(&mut self, window: WindowId, visible: bool) {
        self.world_mut().set_window_visible(window, visible);
    }

    fn set_window_decorated(&mut self, window: WindowId, decorated: bool) {
        self.world_mut().set_window_decorated(window, decorated);
    }

    fn set_window_color(&mut self, window: WindowId, color: Color) {
        self.world_mut().set_window_color(window, color);
    }
}
