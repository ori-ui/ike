use std::{
    fmt,
    ops::{Add, AddAssign, Sub},
};

use crate::Offset;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ORIGIN: Self = Self::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn all(point: f32) -> Self {
        Self { x: point, y: point }
    }

    pub fn distance(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<[f32; 2]> for Point {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
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

impl AddAssign<f32> for Point {
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}

impl AddAssign<Offset> for Point {
    fn add_assign(&mut self, rhs: Offset) {
        *self = *self + rhs;
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
