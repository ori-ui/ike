use crate::{
    AnyWidgetId, AppState, Widget, WidgetId,
    tree::{WidgetMut, WidgetRef},
    widget::AnyWidget,
};

pub trait BuildCx {
    fn app(&self) -> &AppState;
    fn app_mut(&mut self) -> &mut AppState;

    fn get<T>(&self, id: WidgetId<T>) -> WidgetRef<'_, T>
    where
        T: ?Sized + AnyWidget,
    {
        self.app().tree.get(id).unwrap()
    }

    fn get_mut<T>(&mut self, id: WidgetId<T>) -> WidgetMut<'_, T>
    where
        T: ?Sized + AnyWidget,
    {
        self.app_mut().tree.get_mut(id).unwrap()
    }

    fn insert<T>(&mut self, widget: T) -> WidgetMut<'_, T>
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

    fn is_parent(&self, parent: impl AnyWidgetId, child: impl AnyWidgetId) -> bool
    where
        Self: Sized,
    {
        let parent = parent.upcast();
        let child = child.upcast();

        self.app().tree.get_state_unchecked(child.index).parent == Some(parent)
    }
}
