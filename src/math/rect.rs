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
}
