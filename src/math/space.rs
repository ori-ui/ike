use crate::math::vector::Size;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Space {
    pub min: Size,
    pub max: Size,
}

impl Space {
    pub const fn new(min: Size, max: Size) -> Self {
        Self { min, max }
    }
}
