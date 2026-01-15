use crate::math::Size;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Space {
    pub min: Size,
    pub max: Size,
}

impl Space {
    const EPSILON: f32 = 0.0001;

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
        size.width >= self.min.width - Self::EPSILON
            && size.width <= self.max.width + Self::EPSILON
            && size.height >= self.min.height - Self::EPSILON
            && size.height <= self.max.height + Self::EPSILON
    }

    pub const fn constrain(self, size: Size) -> Size {
        Size {
            width:  size.width.max(self.min.width).min(self.max.width),
            height: size.height.max(self.min.height).min(self.max.height),
        }
    }
}
