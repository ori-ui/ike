use crate::{Palette, View};

pub fn palette<T, V>(f: impl FnOnce(&mut T, &Palette) -> V) -> impl View<T>
where
    V: View<T>,
{
    ori::views::using_or_default(f)
}
