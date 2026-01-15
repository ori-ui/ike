use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width:  f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self::all(0.0);
    pub const INFINITY: Self = Self::all(f32::INFINITY);

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const fn all(size: f32) -> Self {
        Self {
            width:  size,
            height: size,
        }
    }

    pub const fn min(self, other: Self) -> Self {
        Self {
            width:  self.width.min(other.width),
            height: self.height.min(other.height),
        }
    }

    pub const fn max(self, other: Self) -> Self {
        Self {
            width:  self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }

    pub const fn ceil(self) -> Self {
        Self {
            width:  self.width.round(),
            height: self.height.round(),
        }
    }

    pub const fn pixel_ceil(self, scale: f32) -> Self {
        Self {
            width:  (self.width * scale).ceil() / scale,
            height: (self.height * scale).ceil() / scale,
        }
    }

    pub const fn area(self) -> f32 {
        self.width * self.height
    }

    pub const fn has_zero_area(self) -> bool {
        self.area() == 0.0
    }

    /// Check if both `width` and `height` are finite.
    pub const fn is_finite(self) -> bool {
        self.width.is_finite() && self.height.is_finite()
    }

    /// Check if both `width` and `height` are infinite.
    pub const fn is_infinite(self) -> bool {
        self.width.is_infinite() && self.height.is_infinite()
    }
}

impl fmt::Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl From<[f32; 2]> for Size {
    fn from([width, height]: [f32; 2]) -> Self {
        Self::new(width, height)
    }
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Self::new(width, height)
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width:  self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl Add<f32> for Size {
    type Output = Size;

    fn add(self, rhs: f32) -> Self::Output {
        Self {
            width:  self.width + rhs,
            height: self.height + rhs,
        }
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign<f32> for Size {
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}

impl Sub for Size {
    type Output = Size;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            width:  self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl SubAssign for Size {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
