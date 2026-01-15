use std::ops::Add;

use crate::{LayoutCx, Offset, Size, Space};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerRadius {
    pub top_left:     f32,
    pub top_right:    f32,
    pub bottom_left:  f32,
    pub bottom_right: f32,
}

impl CornerRadius {
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left:     radius,
            top_right:    radius,
            bottom_left:  radius,
            bottom_right: radius,
        }
    }
}

impl From<[f32; 4]> for CornerRadius {
    fn from([top_left, top_right, bottom_right, bottom_left]: [f32; 4]) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }
}

impl From<f32> for CornerRadius {
    fn from(radius: f32) -> Self {
        Self::all(radius)
    }
}

impl Add<f32> for CornerRadius {
    type Output = CornerRadius;

    fn add(self, rhs: f32) -> Self::Output {
        CornerRadius {
            top_left:     self.top_left + rhs,
            top_right:    self.top_right + rhs,
            bottom_left:  self.bottom_left + rhs,
            bottom_right: self.bottom_right + rhs,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderWidth {
    pub left:   f32,
    pub right:  f32,
    pub top:    f32,
    pub bottom: f32,
}

impl BorderWidth {
    pub const fn all(width: f32) -> Self {
        Self {
            right:  width,
            top:    width,
            left:   width,
            bottom: width,
        }
    }

    pub const fn size(self) -> Size {
        Size::new(
            self.left + self.right,
            self.top + self.bottom,
        )
    }

    pub fn layout_down(self, cx: &LayoutCx<'_>, space: Space) -> Space {
        let size = Size::new(self.left, self.top).pixel_align(cx.scale())
            + Size::new(self.right, self.bottom).pixel_align(cx.scale());

        space.shrink(size)
    }

    pub fn layout_up(self, cx: &LayoutCx<'_>, size: Size) -> Size {
        size + Size::new(self.left, self.top).pixel_align(cx.scale())
            + Size::new(self.right, self.bottom).pixel_align(cx.scale())
    }

    pub const fn offset(self) -> Offset {
        Offset::new(self.left, self.top)
    }

    /// Compute the offset aligned to the device pixel grid.
    pub fn aligned_offset(self, cx: &LayoutCx<'_>) -> Offset {
        self.offset().pixel_align(cx.scale())
    }
}

impl From<[f32; 4]> for BorderWidth {
    fn from([top, right, bottom, left]: [f32; 4]) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }
}

impl From<[f32; 2]> for BorderWidth {
    fn from([horizontal, vertical]: [f32; 2]) -> Self {
        Self {
            left:   horizontal,
            right:  horizontal,
            top:    vertical,
            bottom: vertical,
        }
    }
}

impl From<f32> for BorderWidth {
    fn from(width: f32) -> Self {
        Self::all(width)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Padding {
    pub right:  f32,
    pub top:    f32,
    pub left:   f32,
    pub bottom: f32,
}

impl Padding {
    pub const fn all(padding: f32) -> Self {
        Self {
            right:  padding,
            top:    padding,
            left:   padding,
            bottom: padding,
        }
    }

    pub const fn size(self) -> Size {
        Size::new(
            self.left + self.right,
            self.top + self.bottom,
        )
    }

    pub fn layout_down(self, cx: &LayoutCx<'_>, space: Space) -> Space {
        let size = Size::new(self.left, self.top).pixel_align(cx.scale())
            + Size::new(self.right, self.bottom).pixel_align(cx.scale());

        space.shrink(size)
    }

    pub fn layout_up(self, cx: &LayoutCx<'_>, size: Size) -> Size {
        size + Size::new(self.left, self.top).pixel_align(cx.scale())
            + Size::new(self.right, self.bottom).pixel_align(cx.scale())
    }

    pub const fn offset(self) -> Offset {
        Offset::new(self.left, self.top)
    }

    /// Compute the offset aligned to the device pixel grid.
    pub fn aligned_offset(self, cx: &LayoutCx<'_>) -> Offset {
        self.offset().pixel_align(cx.scale())
    }
}

impl From<[f32; 4]> for Padding {
    fn from([top, right, bottom, left]: [f32; 4]) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }
}

impl From<[f32; 2]> for Padding {
    fn from([horizontal, vertical]: [f32; 2]) -> Self {
        Self {
            left:   horizontal,
            right:  horizontal,
            top:    vertical,
            bottom: vertical,
        }
    }
}

impl From<f32> for Padding {
    fn from(width: f32) -> Self {
        Self::all(width)
    }
}
