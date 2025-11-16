use std::ops::Mul;

use crate::{Offset, Point};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Matrix {
    pub matrix: [f32; 4],
}

impl Matrix {
    pub const IDENTITY: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0],
    };
}

impl Mul for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Self) -> Self::Output {
        let [a1, b1, c1, d1] = self.matrix;
        let [a2, b2, c2, d2] = rhs.matrix;

        Matrix {
            matrix: [
                a1 * a2 + b1 * c2,
                a1 * b2 + b1 * d2,
                c1 * a2 + d1 * c2,
                c1 * b2 + d1 * d2,
            ],
        }
    }
}

impl Mul<Point> for Matrix {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        let [a, b, c, d] = self.matrix;
        let Point { x, y } = rhs;

        Point {
            x: a * x + b * y,
            y: c * x + d * y,
        }
    }
}

impl Mul<Offset> for Matrix {
    type Output = Offset;

    fn mul(self, rhs: Offset) -> Self::Output {
        let [a, b, c, d] = self.matrix;
        let Offset { x, y } = rhs;

        Offset {
            x: a * x + b * y,
            y: c * x + d * y,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Affine {
    pub matrix: Matrix,
    pub offset: Offset,
}

impl Affine {
    pub const IDENTITY: Self = Self {
        matrix: Matrix::IDENTITY,
        offset: Offset::new(0.0, 0.0),
    };
}

impl Mul<Offset> for Affine {
    type Output = Offset;

    fn mul(self, rhs: Offset) -> Self::Output {
        self.matrix * rhs + self.offset
    }
}

impl Mul for Affine {
    type Output = Affine;

    fn mul(self, rhs: Self) -> Self::Output {
        Affine {
            matrix: self.matrix * rhs.matrix,
            offset: self * rhs.offset,
        }
    }
}
