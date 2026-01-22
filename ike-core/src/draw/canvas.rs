use std::hash::Hash;

use crate::{
    Affine, BorderWidth, CornerRadius, Curve, Offset, Paint, Painter, Paragraph, Recording, Rect,
    Svg,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Clip {
    Rect(Rect, CornerRadius),
}

impl From<Rect> for Clip {
    fn from(rect: Rect) -> Self {
        Self::Rect(rect, CornerRadius::all(0.0))
    }
}

impl From<Rect> for Option<Clip> {
    fn from(rect: Rect) -> Self {
        Some(Clip::from(rect))
    }
}

impl Clip {
    pub const fn bounds(&self) -> Rect {
        match self {
            Clip::Rect(rect, _) => *rect,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PixelRect {
    pub left:   u32,
    pub top:    u32,
    pub right:  u32,
    pub bottom: u32,
}

pub trait Canvas {
    fn painter(&mut self) -> &mut dyn Painter;

    fn transform(&mut self, affine: Affine, f: &mut dyn FnMut(&mut dyn Canvas));

    fn layer(&mut self, f: &mut dyn FnMut(&mut dyn Canvas));

    #[must_use]
    fn record(
        &mut self,
        width: u32,
        height: u32,
        f: &mut dyn FnMut(&mut dyn Canvas),
    ) -> Option<Recording>;

    fn clip(&mut self, clip: &Clip, f: &mut dyn FnMut(&mut dyn Canvas));

    fn fill(&mut self, paint: &Paint);

    fn draw_curve(&mut self, curve: &Curve, paint: &Paint);

    fn draw_rect(&mut self, rect: Rect, corners: CornerRadius, paint: &Paint);

    fn draw_border(&mut self, rect: Rect, width: BorderWidth, radius: CornerRadius, paint: &Paint);

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset);

    fn draw_svg(&mut self, svg: &Svg);

    fn draw_recording(&mut self, rect: Rect, recording: &Recording);
}
