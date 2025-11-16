use crate::{
    Color, CornerRadius, Offset, Size,
    math::{Affine, Rect},
    text::Paragraph,
};

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Shader {
    Solid(Color),
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Paint {
    pub shader: Shader,
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self {
            shader: Shader::Solid(color),
        }
    }
}

pub trait Fonts {
    fn measure(&mut self, paragraph: &Paragraph, max_width: f32) -> Size;
}

pub trait Canvas {
    fn transform(&mut self, affine: Affine, f: &mut dyn FnMut(&mut dyn Canvas));

    fn draw_rect(&mut self, rect: Rect, corners: CornerRadius, paint: &Paint);

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset);
}
