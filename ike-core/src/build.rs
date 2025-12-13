use crate::{
    AnyWidgetId, Color, Root, Widget, WidgetId, WindowId, WindowSizing,
    arena::{WidgetMut, WidgetRef},
    widget::AnyWidget,
};

pub trait BuildCx {
    fn root(&self) -> &Root;
    fn root_mut(&mut self) -> &mut Root;

    fn get<T>(&self, id: WidgetId<T>) -> Option<WidgetRef<'_, T>>
    where
        Self: Sized,
        T: ?Sized + AnyWidget,
    {
        let root = self.root();
        root.arena.get(&root.state, id)
    }

    fn get_mut<T>(&mut self, id: WidgetId<T>) -> Option<WidgetMut<'_, T>>
    where
        Self: Sized,
        T: ?Sized + AnyWidget,
    {
        let root = self.root_mut();
        root.arena.get_mut(&mut root.state, id)
    }

    fn insert<T>(&mut self, widget: T) -> WidgetMut<'_, T>
    where
        Self: Sized,
        T: Widget,
    {
        let root = self.root_mut();
        root.arena.insert(&mut root.state, widget)
    }

    fn remove(&mut self, id: impl AnyWidgetId)
    where
        Self: Sized,
    {
        let root = self.root_mut();
        root.arena.remove(&mut root.state, id.upcast());
    }

    fn is_parent(&self, parent: impl AnyWidgetId, child: impl AnyWidgetId) -> bool
    where
        Self: Sized,
    {
        let parent = parent.upcast();
        let child = child.upcast();

        self.root()
            .arena
            .get_state(child.index)
            .is_some_and(|child| child.parent == Some(parent))
    }

    #[must_use]
    fn set_window_contents(
        &mut self,
        window: WindowId,
        contents: impl AnyWidgetId,
    ) -> Option<WidgetId>
    where
        Self: Sized,
    {
        self.root_mut()
            .set_window_contents(window, contents.upcast())
    }

    fn set_window_title(&mut self, window: WindowId, title: String) {
        self.root_mut().set_window_title(window, title);
    }

    fn set_window_sizing(&mut self, window: WindowId, sizing: WindowSizing) {
        self.root_mut().set_window_sizing(window, sizing);
    }

    fn set_window_visible(&mut self, window: WindowId, visible: bool) {
        self.root_mut().set_window_visible(window, visible);
    }

    fn set_window_decorated(&mut self, window: WindowId, decorated: bool) {
        self.root_mut().set_window_decorated(window, decorated);
    }

    fn set_window_color(&mut self, window: WindowId, color: Color) {
        self.root_mut().set_window_color(window, color);
    }
}
