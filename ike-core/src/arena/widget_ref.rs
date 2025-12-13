use crate::{Widget, context::RefCx};

pub struct WidgetRef<'a, T = dyn Widget>
where
    T: ?Sized,
{
    pub widget: &'a T,

    pub cx: RefCx<'a>,
}

impl<T> Drop for WidgetRef<'_, T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        let entry = &self.cx.arena.entries[self.cx.state.id.index as usize];
        entry.borrowed.set(false);
    }
}
