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

    pub fn shrink(self, size: Size) -> Self {
        Self {
            min: Size::max(self.min - size, Size::new(0.0, 0.0)),
            max: Size::max(self.max - size, Size::new(0.0, 0.0)),
        }
    }
}
