use crate::{
    Affine, BorderWidth, Color, CornerRadius, Offset, Painter, Paragraph, Recording, Rect, Size,
    Svg,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub enum Shader {
    Solid(Color),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlendMode {
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct Paint {
    pub shader: Shader,
    pub blend:  BlendMode,
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self {
            shader: Shader::Solid(color),
            blend:  BlendMode::SrcOver,
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

pub trait Canvas {
    fn painter(&mut self) -> &mut dyn Painter;

    fn transform(&mut self, affine: Affine, f: &mut dyn FnMut(&mut dyn Canvas));

    fn layer(&mut self, f: &mut dyn FnMut(&mut dyn Canvas));

    #[must_use]
    fn record(&mut self, size: Size, f: &mut dyn FnMut(&mut dyn Canvas)) -> Option<Recording>;

    fn clip(&mut self, clip: &Clip, f: &mut dyn FnMut(&mut dyn Canvas));

    fn fill(&mut self, paint: &Paint);

    fn draw_rect(&mut self, rect: Rect, corners: CornerRadius, paint: &Paint);

    fn draw_border(&mut self, rect: Rect, width: BorderWidth, radius: CornerRadius, paint: &Paint);

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset);

    fn draw_svg(&mut self, svg: &Svg);

    fn draw_recording(&mut self, recording: &Recording);
}
