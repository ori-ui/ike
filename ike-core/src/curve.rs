use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};

use crate::Point;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fill {
    Winding,
    EvenOdd,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    data: Arc<CurveData>,
}

impl Default for Curve {
    fn default() -> Self {
        Self::new()
    }
}

impl Curve {
    pub fn new() -> Self {
        Self {
            data: Arc::new(CurveData::new()),
        }
    }

    pub fn downgrade(this: &Self) -> WeakCurve {
        WeakCurve {
            data: Arc::downgrade(&this.data),
        }
    }
}

impl Deref for Curve {
    type Target = CurveData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Curve {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.data)
    }
}

#[derive(Clone, Debug)]
pub struct WeakCurve {
    data: Weak<CurveData>,
}

impl WeakCurve {
    pub fn upgrade(&mut self) -> Option<Curve> {
        Some(Curve {
            data: self.data.upgrade()?,
        })
    }

    pub fn strong_count(&self) -> usize {
        self.data.strong_count()
    }
}

impl PartialEq for WeakCurve {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.data, &other.data)
    }
}

impl Eq for WeakCurve {}

impl Hash for WeakCurve {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.as_ptr().hash(state);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurveData {
    pub fill: Fill,

    points: Vec<Point>,
    verbs:  Vec<CurveVerb>,
}

impl Default for CurveData {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveData {
    pub const fn new() -> Self {
        Self {
            fill: Fill::EvenOdd,

            points: Vec::new(),
            verbs:  Vec::new(),
        }
    }

    pub const fn len(&self) -> usize {
        self.verbs.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }

    pub fn is_closed(&self) -> bool {
        self.verbs.last() == Some(&CurveVerb::Close)
    }

    pub fn move_to(&mut self, p: Point) {
        self.points.push(p);
        self.verbs.push(CurveVerb::Move);
    }

    pub fn line_to(&mut self, p: Point) {
        self.points.push(p);
        self.verbs.push(CurveVerb::Line);
    }

    pub fn quad_to(&mut self, a: Point, p: Point) {
        self.points.push(a);
        self.points.push(p);
        self.verbs.push(CurveVerb::Quad);
    }

    pub fn cubic_to(&mut self, a: Point, b: Point, p: Point) {
        self.points.push(a);
        self.points.push(b);
        self.points.push(p);
        self.verbs.push(CurveVerb::Cubic);
    }

    pub fn close(&mut self) {
        if !self.is_closed() && !self.is_empty() {
            self.verbs.push(CurveVerb::Close);
        }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = CurveSegment> {
        const EXPECT: &str = "invariants of `Curve` should be upheld";

        let mut points = self.points.iter();

        self.verbs.iter().map(move |verb| match verb {
            CurveVerb::Move => {
                let p = *points.next().expect(EXPECT);
                CurveSegment::Move(p)
            }

            CurveVerb::Line => {
                let p = *points.next().expect(EXPECT);
                CurveSegment::Line(p)
            }

            CurveVerb::Quad => {
                let a = *points.next().expect(EXPECT);
                let p = *points.next().expect(EXPECT);
                CurveSegment::Quad(a, p)
            }

            CurveVerb::Cubic => {
                let a = *points.next().expect(EXPECT);
                let b = *points.next().expect(EXPECT);
                let p = *points.next().expect(EXPECT);
                CurveSegment::Cubic(a, b, p)
            }

            CurveVerb::Close => CurveSegment::Close,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CurveVerb {
    Move,
    Line,
    Quad,
    Cubic,
    Close,
}

impl CurveVerb {
    pub const fn point_count(self) -> usize {
        match self {
            CurveVerb::Move => 1,
            CurveVerb::Line => 1,
            CurveVerb::Quad => 2,
            CurveVerb::Cubic => 3,
            CurveVerb::Close => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CurveSegment {
    Move(Point),
    Line(Point),
    Quad(Point, Point),
    Cubic(Point, Point, Point),
    Close,
}
