use crate::{Point, Size};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub const fn min_size(min: Point, size: Size) -> Self {
        Self {
            min,
            max: Point {
                x: min.x + size.width,
                y: min.y + size.height,
            },
        }
    }

    pub fn expand(self, margin: f32) -> Self {
        Self {
            min: self.min - margin,
            max: self.max + margin,
        }
    }

    pub const fn top(self) -> f32 {
        self.min.y
    }

    pub const fn right(self) -> f32 {
        self.max.x
    }

    pub const fn bottom(self) -> f32 {
        self.max.y
    }

    pub const fn left(self) -> f32 {
        self.min.x
    }

    pub const fn center(self) -> Point {
        Point {
            x: (self.min.x + self.max.x) / 2.0,
            y: (self.min.y + self.max.y) / 2.0,
        }
    }

    pub const fn top_left(self) -> Point {
        Point {
            x: self.left(),
            y: self.top(),
        }
    }

    pub const fn top_right(self) -> Point {
        Point {
            x: self.right(),
            y: self.top(),
        }
    }

    pub const fn bottom_left(self) -> Point {
        Point {
            x: self.left(),
            y: self.bottom(),
        }
    }

    pub const fn bottom_right(self) -> Point {
        Point {
            x: self.right(),
            y: self.bottom(),
        }
    }

    pub const fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    pub const fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    pub const fn size(self) -> Size {
        Size {
            width:  self.width(),
            height: self.height(),
        }
    }

    pub const fn contains(self, point: Point) -> bool {
        point.x >= self.min.x
            && point.y >= self.min.y
            && point.x <= self.max.x
            && point.y <= self.max.y
    }
}
