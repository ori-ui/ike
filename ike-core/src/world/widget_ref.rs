use std::{cell::Ref, marker::PhantomData};

use crate::{AnyWidget, RefCx, Widget, WidgetId};

pub struct WidgetRef<'a, T = dyn Widget>
where
    T: Widget + ?Sized,
{
    pub widget: Ref<'a, T>,
    pub cx:     RefCx<'a>,
}

impl<'a> WidgetRef<'a, dyn Widget> {
    pub fn downcast<T>(self) -> Option<WidgetRef<'a, T>>
    where
        T: Widget,
    {
        Some(WidgetRef {
            widget: Ref::filter_map(self.widget, T::downcast_ref).ok()?,

            cx: RefCx {
                widgets:   self.cx.widgets,
                world:     self.cx.world,
                state:     self.cx.state,
                hierarchy: self.cx.hierarchy,
            },
        })
    }
}

impl<'a, T> WidgetRef<'a, T>
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

    pub fn upcast(self) -> WidgetRef<'a>
    where
        T: Sized,
    {
        WidgetRef {
            widget: Ref::map(self.widget, T::upcast_ref),

            cx: RefCx {
                widgets:   self.cx.widgets,
                world:     self.cx.world,
                state:     self.cx.state,
                hierarchy: self.cx.hierarchy,
            },
        }
    }
}
