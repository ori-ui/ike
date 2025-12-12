use std::ops::Deref;

use crate::{
    Affine, AnyWidgetId, CursorIcon, Size, Tree, Widget, WidgetId, WidgetMut, Window, WindowId,
    root::RootState,
    widget::{AnyWidget, WidgetState},
};

pub struct WidgetRef<'a, T = dyn Widget>
where
    T: ?Sized,
{
    pub(super) id:     WidgetId<T>,
    pub(crate) root:   &'a RootState,
    pub(super) tree:   &'a Tree,
    pub(super) widget: &'a T,
    pub(super) state:  &'a WidgetState,
}

impl<'a, T> WidgetRef<'a, T>
where
    T: ?Sized,
{
    pub(crate) fn state(&self) -> &WidgetState {
        self.state
    }
}

impl<T> Deref for WidgetRef<'_, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.widget
    }
}

macro_rules! impl_widget_ref {
    ($name:ident) => {
        impl<'a, T> $name<'a, T>
        where
            T: ?Sized + AnyWidget,
        {
            pub fn id(&self) -> WidgetId<T> {
                self.id
            }

            pub fn get<U>(&self, id: WidgetId<U>) -> Option<WidgetRef<'_, U>>
            where
                U: ?Sized + AnyWidget,
            {
                self.tree.get(self.root, id)
            }

            pub fn children(&self) -> &[WidgetId] {
                &self.state().children
            }

            pub fn is_child(&self, child: impl AnyWidgetId) -> bool {
                self.get(child.upcast())
                    .is_some_and(|child| child.state.parent == Some(self.id.upcast()))
            }

            pub fn parent(&self) -> Option<WidgetRef<'_, dyn Widget>> {
                self.tree.get(self.root, self.state().parent?)
            }

            pub fn child(&self, index: usize) -> WidgetRef<'_, dyn Widget> {
                self.tree
                    .get(self.root, self.state().children[index])
                    .unwrap()
            }

            pub fn size(&self) -> Size {
                self.state().size
            }

            pub fn transform(&self) -> Affine {
                self.state().transform
            }

            pub fn global_transform(&self) -> Affine {
                self.state().global_transform
            }

            pub fn is_pixel_perfect(&self) -> bool {
                self.state().is_pixel_perfect
            }

            pub fn is_hovered(&self) -> bool {
                self.state().is_hovered
            }

            pub fn is_active(&self) -> bool {
                self.state().is_active
            }

            pub fn is_focused(&self) -> bool {
                self.state().is_focused
            }

            pub fn is_stashed(&self) -> bool {
                self.state().is_stashed
            }

            pub fn cursor(&self) -> CursorIcon {
                self.state().cursor
            }

            pub fn get_window(&self, id: WindowId) -> Option<&Window> {
                self.root.get_window(id)
            }
        }
    };
}

impl_widget_ref!(WidgetRef);
impl_widget_ref!(WidgetMut);
