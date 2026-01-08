use std::marker::PhantomData;

use crate::{AnyWidget, MutCx, Update, Widget, WidgetId, passes};

#[non_exhaustive]
pub struct WidgetMut<'a, T = dyn Widget>
where
    T: Widget + ?Sized,
{
    pub widget: &'a mut T,

    pub cx: MutCx<'a>,
}

impl<'a, T> Drop for WidgetMut<'a, T>
where
    T: Widget + ?Sized,
{
    fn drop(&mut self) {
        let index = self.cx.state.id.index as usize;
        unsafe { self.cx.widgets.release_mut(index) };
    }
}

impl<T> WidgetMut<'_, T>
where
    T: Widget + ?Sized,
{
    pub fn id(&self) -> WidgetId<T> {
        let id = self.cx.id();

        WidgetId {
            index:      id.index,
            generation: id.generation,
            marker:     PhantomData,
        }
    }

    pub fn upcast(&mut self) -> WidgetMut<'_>
    where
        T: AnyWidget,
    {
        WidgetMut {
            widget: AnyWidget::upcast_mut(self.widget),

            cx: MutCx {
                widgets:   self.cx.widgets,
                world:     self.cx.world,
                state:     self.cx.state,
                hierarchy: self.cx.hierarchy,
            },
        }
    }

    pub fn set_disabled(&mut self, is_disabled: bool) {
        passes::propagate::set_disabled(self, is_disabled);
    }

    pub(crate) fn set_hovered(&mut self, is_hovered: bool) {
        if self.cx.hierarchy.is_hovered() == is_hovered {
            return;
        }

        self.cx.hierarchy.set_hovered(is_hovered);
        self.widget.update(
            &mut self.cx.as_update_cx(),
            Update::Hovered(is_hovered),
        );

        passes::propagate::propagate_down(self.cx.widgets, self.cx.id());
    }

    pub(crate) fn set_focused(&mut self, is_focused: bool) {
        if self.cx.hierarchy.is_focused() == is_focused {
            return;
        }

        self.cx.hierarchy.set_focused(is_focused);
        self.widget.update(
            &mut self.cx.as_update_cx(),
            Update::Focused(is_focused),
        );

        passes::propagate::propagate_down(self.cx.widgets, self.cx.id());
    }

    pub(crate) fn set_active(&mut self, is_active: bool) {
        if self.cx.hierarchy.is_active() == is_active {
            return;
        }

        self.cx.hierarchy.set_active(is_active);
        self.widget.update(
            &mut self.cx.as_update_cx(),
            Update::Active(is_active),
        );

        passes::propagate::propagate_down(self.cx.widgets, self.cx.id());
    }
}
