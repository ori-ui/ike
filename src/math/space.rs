use crate::math::Size;

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

    pub const fn contains(self, size: Size) -> bool {
        size.width >= self.min.width
            && size.width <= self.max.width
            && size.height >= self.min.height
            && size.height <= self.max.height
    }

    pub const fn constrain(self, size: Size) -> Size {
        Size {
            width:  size.width.clamp(self.min.width, self.max.width),
            height: size.height.clamp(self.min.height, self.max.height),
        }
    }
}
