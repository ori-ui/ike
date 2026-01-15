use std::{cell::RefMut, marker::PhantomData};

use crate::{AnyWidget, MutCx, Update, Widget, WidgetId, passes};

pub struct WidgetMut<'a, T = dyn Widget>
where
    T: Widget + ?Sized,
{
    pub widget: RefMut<'a, T>,
    pub cx:     MutCx<'a>,
}

impl<'a> WidgetMut<'a, dyn Widget> {
    pub fn downcast<T>(self) -> Option<WidgetMut<'a, T>>
    where
        T: Widget,
    {
        Some(WidgetMut {
            widget: RefMut::filter_map(self.widget, T::downcast_mut).ok()?,

            cx: MutCx {
                widgets:   self.cx.widgets,
                world:     self.cx.world,
                state:     self.cx.state,
                hierarchy: self.cx.hierarchy,
            },
        })
    }
}

impl<'a, T> WidgetMut<'a, T>
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

    pub fn upcast(self) -> WidgetMut<'a>
    where
        T: Sized,
    {
        WidgetMut {
            widget: RefMut::map(self.widget, T::upcast_mut),

            cx: MutCx {
                widgets:   self.cx.widgets,
                world:     self.cx.world,
                state:     self.cx.state,
                hierarchy: self.cx.hierarchy,
            },
        }
    }

    pub fn set_stashed(&mut self, is_stashed: bool) {
        passes::hierarchy::set_stashed(self, is_stashed);
    }

    pub fn set_disabled(&mut self, is_disabled: bool) {
        passes::hierarchy::set_disabled(self, is_disabled);
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

        passes::hierarchy::propagate_down(self.cx.widgets, self.cx.id());
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

        passes::hierarchy::propagate_down(self.cx.widgets, self.cx.id());
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

        passes::hierarchy::propagate_down(self.cx.widgets, self.cx.id());
    }
}
