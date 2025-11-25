use crate::{Affine, BorderWidth, Color, CornerRadius, Offset, Painter, Paragraph, Rect, Svg};

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Shader {
    Solid(Color),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlendMode {
    Clear,
    Src,
    Dst,
    SrcOver,
    DstOver,
    SrcATop,
    DstATop,
}

#[derive(Clone, PartialEq, PartialOrd)]
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

pub trait Canvas {
    fn painter(&mut self) -> &mut dyn Painter;

    fn transform(&mut self, affine: Affine, f: &mut dyn FnMut(&mut dyn Canvas));

    fn layer(&mut self, f: &mut dyn FnMut(&mut dyn Canvas));

    fn fill(&mut self, paint: &Paint);

    fn draw_rect(&mut self, rect: Rect, corners: CornerRadius, paint: &Paint);

    fn draw_border(&mut self, rect: Rect, width: BorderWidth, radius: CornerRadius, paint: &Paint);

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset);

    fn draw_svg(&mut self, svg: &Svg);
}
