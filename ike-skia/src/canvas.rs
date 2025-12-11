use ike_core::{
    Affine, BorderWidth, Canvas, Clip, CornerRadius, Offset, Paint, Painter, Paragraph, Rect, Svg,
};

use crate::painter::SkiaPainter;

pub struct SkiaCanvas<'a> {
    pub(crate) painter: &'a mut SkiaPainter,
    pub(crate) canvas:  &'a skia_safe::Canvas,
}

impl Canvas for SkiaCanvas<'_> {
    fn painter(&mut self) -> &mut dyn Painter {
        self.painter
    }

    fn transform(&mut self, affine: Affine, f: &mut dyn FnMut(&mut dyn Canvas)) {
        let matrix = skia_safe::Matrix::new_all(
            affine.matrix.matrix[0],
            affine.matrix.matrix[1],
            affine.offset.x,
            affine.matrix.matrix[2],
            affine.matrix.matrix[3],
            affine.offset.y,
            0.0,
            0.0,
            1.0,
        );

        self.canvas.save();
        self.canvas.concat(&matrix);

        f(self);

        self.canvas.restore();
    }

    fn layer(&mut self, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.canvas.save_layer_alpha_f(None, 1.0);
        f(self);
        self.canvas.restore();
    }

    fn clip(&mut self, clip: &Clip, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.canvas.save();

        match clip {
            Clip::Rect(rect, radius) => {
                let rect = skia_safe::RRect::new_nine_patch(
                    skia_safe::Rect::new(rect.min.x, rect.min.y, rect.max.x, rect.max.y),
                    radius.top_left,
                    radius.top_right,
                    radius.bottom_right,
                    radius.bottom_left,
                );

                self.canvas.clip_rrect(
                    rect,
                    skia_safe::ClipOp::Intersect,
                    true, // enable anti aliasing
                );
            }
        }

        f(self);

        self.canvas.restore();
    }

    fn fill(&mut self, paint: &Paint) {
        let paint = self.painter.create_paint(paint);
        self.canvas.draw_paint(paint);
    }

    fn draw_rect(&mut self, rect: Rect, radius: CornerRadius, paint: &Paint) {
        let rect = skia_safe::RRect::new_nine_patch(
            skia_safe::Rect::new(rect.min.x, rect.min.y, rect.max.x, rect.max.y),
            radius.top_left,
            radius.top_right,
            radius.bottom_right,
            radius.bottom_left,
        );

        let paint = self.painter.create_paint(paint);
        self.canvas.draw_rrect(rect, paint);
    }

    fn draw_border(&mut self, rect: Rect, width: BorderWidth, radius: CornerRadius, paint: &Paint) {
        if width == BorderWidth::all(0.0) {
            return;
        }

        let inner = skia_safe::RRect::new_nine_patch(
            skia_safe::Rect::new(
                rect.min.x + width.left,
                rect.min.y + width.top,
                rect.max.x - width.right,
                rect.max.y - width.bottom,
            ),
            radius.top_left - (width.top + width.left) / 2.0,
            radius.top_right - (width.top + width.right) / 2.0,
            radius.bottom_right - (width.bottom + width.right) / 2.0,
            radius.bottom_left - (width.bottom + width.left) / 2.0,
        );

        let outer = skia_safe::RRect::new_nine_patch(
            skia_safe::Rect::new(rect.min.x, rect.min.y, rect.max.x, rect.max.y),
            radius.top_left,
            radius.top_right,
            radius.bottom_right,
            radius.bottom_left,
        );

        let paint = self.painter.create_paint(paint);
        self.canvas.draw_drrect(outer, inner, paint);
    }

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset) {
        let paragraph = self.painter.create_paragraph(paragraph, max_width + 1.0);
        paragraph.paint(self.canvas, (offset.x, offset.y));
    }

    fn draw_svg(&mut self, svg: &Svg) {
        if let Some(skia_dom) = self.painter.create_svg(svg) {
            skia_dom.render(self.canvas);
        }
    }
}
