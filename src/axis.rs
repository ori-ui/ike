use crate::{Offset, Point, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub(crate) fn unpack_size(self, size: Size) -> (f32, f32) {
        match self {
            Axis::Horizontal => (size.width, size.height),
            Axis::Vertical => (size.height, size.width),
        }
    }

    pub(crate) fn unpack_offset(self, offet: Offset) -> (f32, f32) {
        match self {
            Axis::Horizontal => (offet.x, offet.y),
            Axis::Vertical => (offet.y, offet.x),
        }
    }

    pub(crate) fn unpack_point(self, point: Point) -> (f32, f32) {
        match self {
            Axis::Horizontal => (point.x, point.y),
            Axis::Vertical => (point.y, point.x),
        }
    }

    pub(crate) fn pack_size(self, major: f32, minor: f32) -> Size {
        match self {
            Axis::Horizontal => Size::new(major, minor),
            Axis::Vertical => Size::new(minor, major),
        }
    }

    pub(crate) fn pack_offset(self, major: f32, minor: f32) -> Offset {
        match self {
            Axis::Horizontal => Offset::new(major, minor),
            Axis::Vertical => Offset::new(minor, major),
        }
    }

    pub(crate) fn pack_point(self, major: f32, minor: f32) -> Point {
        match self {
            Axis::Horizontal => Point::new(major, minor),
            Axis::Vertical => Point::new(minor, major),
        }
    }
}
