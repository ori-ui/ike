use std::marker::PhantomData;

use crate::{MutCx, Widget, WidgetId};

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
}
