use std::hash::{Hash, Hasher};

use crate::{
    Affine, BorderWidth, Color, CornerRadius, Curve, Offset, Painter, Paragraph, Recording, Rect,
    Svg,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub enum Shader {
    Solid(Color),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Blend {
    Clear,
    Src,
    Dst,
    SrcOver,
    DstOver,
    SrcIn,
    DstIn,
    SrcATop,
    DstATop,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Join {
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Stroke {
    pub width: f32,
    pub miter: f32,
    pub join:  Join,
    pub cap:   Cap,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            miter: 4.0,
            join:  Join::Miter,
            cap:   Cap::Butt,
        }
    }
}

impl Eq for Stroke {}

impl Hash for Stroke {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.miter.to_bits().hash(state);
        self.join.hash(state);
        self.cap.hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct Paint {
    pub shader: Shader,
    pub blend:  Blend,
    pub stroke: Stroke,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            shader: Shader::Solid(Color::BLACK),
            blend:  Blend::SrcOver,
            stroke: Stroke::default(),
        }
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self {
            shader: Shader::Solid(color),
            ..Default::default()
        }
    }
}

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
