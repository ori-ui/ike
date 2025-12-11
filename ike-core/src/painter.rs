use crate::{Paragraph, Size, Svg, TextLayoutLine};

pub trait Painter {
    fn measure_svg(&mut self, svg: &Svg) -> Size;

    fn measure_text(&mut self, paragraph: &Paragraph, max_width: f32) -> Size;

    fn layout_text(&mut self, paragraph: &Paragraph, max_width: f32) -> Vec<TextLayoutLine>;
}
