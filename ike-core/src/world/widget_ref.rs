use std::marker::PhantomData;

use crate::{RefCx, Widget, WidgetId};

#[non_exhaustive]
pub struct WidgetRef<'a, T = dyn Widget>
where
    T: Widget + ?Sized,
{
    pub widget: &'a T,

    pub cx: RefCx<'a>,
}

impl<'a, T> Drop for WidgetRef<'a, T>
where
    T: Widget + ?Sized,
{
    fn drop(&mut self) {
        let index = self.cx.state.id.index as usize;
        unsafe { self.cx.widgets.release_ref(index) };
    }
}

impl<T> WidgetRef<'_, T>
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
}
