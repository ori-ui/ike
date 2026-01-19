use std::{
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::layout;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    pub const ZERO: Self = Self::all(0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn all(offset: f32) -> Self {
        Self {
            x: offset,
            y: offset,
        }
    }

    pub const fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub const fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    pub const fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    /// Compute `self` aligned to the pixel grid for a certain scale factor.
    pub const fn pixel_round(self, scale: f32) -> Self {
        Self {
            x: layout::pixel_round(self.x, scale),
            y: layout::pixel_round(self.y, scale),
        }
    }

    /// Compute `self` aligned to the pixel grid for a certain scale factor.
    pub const fn pixel_ceil(self, scale: f32) -> Self {
        Self {
            x: layout::pixel_ceil(self.x, scale),
            y: layout::pixel_ceil(self.y, scale),
        }
    }
}

impl fmt::Debug for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}>", self.x, self.y)
    }
}

impl From<[f32; 2]> for Offset {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<(f32, f32)> for Offset {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl Add<f32> for Offset {
    type Output = Offset;

    fn add(self, rhs: f32) -> Self::Output {
        Offset {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl Add for Offset {
    type Output = Offset;

    fn add(self, rhs: Self) -> Self::Output {
        Offset {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<f32> for Offset {
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}

impl AddAssign for Offset {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub<f32> for Offset {
    type Output = Offset;

    fn sub(self, rhs: f32) -> Self::Output {
        Offset {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl Sub for Offset {
    type Output = Offset;

    fn sub(self, rhs: Self) -> Self::Output {
        Offset {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign<f32> for Offset {
    fn sub_assign(&mut self, rhs: f32) {
        *self = *self - rhs;
    }
}

impl SubAssign for Offset {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<f32> for Offset {
    type Output = Offset;

    fn mul(self, rhs: f32) -> Self::Output {
        Offset {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<f32> for Offset {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Div<f32> for Offset {
    type Output = Offset;

    fn div(self, rhs: f32) -> Self::Output {
        Offset {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign<f32> for Offset {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl Neg for Offset {
    type Output = Offset;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}
