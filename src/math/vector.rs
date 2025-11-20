use std::{
    fmt,
    ops::{Add, Neg, Sub},
};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width:  f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const fn all(size: f32) -> Self {
        Self {
            width:  size,
            height: size,
        }
    }

    pub fn min(self, other: Self) -> Self {
        Self {
            width:  self.width.min(other.width),
            height: self.height.min(other.height),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            width:  self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn all(point: f32) -> Self {
        Self { x: point, y: point }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn all(offset: f32) -> Self {
        Self {
            x: offset,
            y: offset,
        }
    }
}

impl fmt::Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl fmt::Debug for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}>", self.x, self.y)
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

impl Sub for Size {
    type Output = Size;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            width:  self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl Sub for Point {
    type Output = Offset;

    fn sub(self, rhs: Self) -> Self::Output {
        Offset {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add<f32> for Point {
    type Output = Point;

    fn add(self, rhs: f32) -> Self::Output {
        Point {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl Add<Offset> for Point {
    type Output = Point;

    fn add(self, rhs: Offset) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<f32> for Point {
    type Output = Point;

    fn sub(self, rhs: f32) -> Self::Output {
        Point {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl Sub<Offset> for Point {
    type Output = Point;

    fn sub(self, rhs: Offset) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
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

impl Neg for Offset {
    type Output = Offset;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}
