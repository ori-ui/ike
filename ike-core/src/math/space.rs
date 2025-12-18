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

    pub const fn constrain(self, mut size: Size) -> Size {
        if self.min.width.is_finite() {
            size.width = size.width.max(self.min.width);
        }

        if self.min.height.is_finite() {
            size.height = size.height.max(self.min.height);
        }

        Size {
            width:  size.width.min(self.max.width),
            height: size.height.min(self.max.height),
        }
    }
}
