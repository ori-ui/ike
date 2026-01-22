use crate::{Affine, PixelRect, Point, Size};

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

    pub fn shrink(self, margin: f32) -> Self {
        self.expand(-margin)
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

    pub const fn intersection(mut self, other: Self) -> Self {
        self.min.x = self.min.x.max(other.min.x);
        self.min.y = self.min.y.max(other.min.y);

        self.max.x = self.max.x.min(other.max.x);
        self.max.y = self.max.y.min(other.max.y);

        self
    }

    pub fn to_pixels(self, scale: f32) -> PixelRect {
        const EPSILON: f32 = 0.001;

        PixelRect {
            left:   (self.left() * scale + EPSILON).floor().max(0.0) as u32,
            top:    (self.top() * scale + EPSILON).floor().max(0.0) as u32,
            right:  (self.right() * scale - EPSILON).ceil() as u32,
            bottom: (self.bottom() * scale - EPSILON).ceil() as u32,
        }
    }

    pub fn transform_bounds(mut self, transform: Affine) -> Self {
        let tl = transform * self.top_left();
        let tr = transform * self.top_right();
        let bl = transform * self.bottom_left();
        let br = transform * self.bottom_right();

        self.min.x = tl.x.min(tr.x).min(bl.x).min(br.x);
        self.min.y = tl.y.min(tr.y).min(bl.y).min(br.y);
        self.max.x = tl.x.max(tr.x).max(bl.x).max(br.x);
        self.max.y = tl.y.max(tr.y).max(bl.y).max(br.y);

        self
    }
}
