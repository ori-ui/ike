use crate::{Offset, Size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerRadius {
    pub top_right: f32,
    pub top_left: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl CornerRadius {
    pub const fn all(radius: f32) -> Self {
        Self {
            top_right: radius,
            top_left: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderWidth {
    pub right: f32,
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
}

impl BorderWidth {
    pub const fn all(width: f32) -> Self {
        Self {
            right: width,
            top: width,
            left: width,
            bottom: width,
        }
    }

    pub const fn size(self) -> Size {
        Size::new(self.left + self.right, self.top + self.bottom)
    }

    pub const fn offset(self) -> Offset {
        Offset::new(self.left, self.top)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Padding {
    pub right: f32,
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
}

impl Padding {
    pub const fn all(padding: f32) -> Self {
        Self {
            right: padding,
            top: padding,
            left: padding,
            bottom: padding,
        }
    }

    pub const fn size(self) -> Size {
        Size::new(self.left + self.right, self.top + self.bottom)
    }

    pub const fn offset(self) -> Offset {
        Offset::new(self.left, self.top)
    }
}
