use ike_core::{
    Affine, BorderWidth, Canvas, Clip, CornerRadius, Curve, Offset, Paint, Painter, Paragraph,
    Recording, RecordingData, Rect, Size, Svg,
};

use crate::{painter::SkiaPainter, vulkan::Surface};

pub struct SkiaCanvas<'a> {
    pub(crate) surface: &'a mut Surface,
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
            affine.matrix.matrix[2],
            affine.offset.x,
            affine.matrix.matrix[1],
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

    fn record(&mut self, size: Size, f: &mut dyn FnMut(&mut dyn Canvas)) -> Option<Recording> {
        let matrix = self.canvas.local_to_device_as_3x3();
        let scale_x = matrix.scale_x();
        let scale_y = matrix.scale_y();

        let width = (size.width * scale_x).ceil() as u32;
        let height = (size.height * scale_y).ceil() as u32;
        let memory = width as u64 * height as u64 * 4;

        let mut surface = self
            .surface
            .create_render_target(width, height, false)
            .ok()?;

        let mut canvas = SkiaCanvas {
            surface: self.surface,
            painter: self.painter,
            canvas:  surface.canvas(),
        };

        canvas.canvas.clear(skia_safe::Color::TRANSPARENT);
        canvas.canvas.scale((scale_x, scale_y));

        f(&mut canvas);

        let image = surface.image_snapshot();

        let recording = Recording::new(RecordingData {
            size,
            width,
            height,
            memory,
        });
        let weak = Recording::downgrade(&recording);

        self.painter.recordings.insert(weak, (image, size));

        Some(recording)
    }

    fn clip(&mut self, clip: &Clip, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.canvas.save();

        match clip {
            Clip::Rect(rect, radius) => {
                if *radius != CornerRadius::all(0.0) {
                    let rect = skia_safe::RRect::new_nine_patch(
                        skia_safe::Rect::new(
                            rect.min.x, rect.min.y, rect.max.x, rect.max.y,
                        ),
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
                } else {
                    let rect = skia_safe::Rect::new(
                        rect.min.x, rect.min.y, rect.max.x, rect.max.y,
                    );

                    self.canvas.clip_rect(
                        rect,
                        skia_safe::ClipOp::Intersect,
                        true, // enable anti aliasing
                    );
                }
            }
        }

        f(self);

        self.canvas.restore();
    }

    fn fill(&mut self, paint: &Paint) {
        let paint = self.painter.create_paint(paint);
        self.canvas.draw_paint(paint);
    }

    fn draw_curve(&mut self, curve: &Curve, paint: &Paint) {
        let path = self.painter.create_path(curve).clone();
        let paint = self.painter.create_paint(paint);

        self.canvas.draw_path(&path, paint);
    }

    fn draw_rect(&mut self, rect: Rect, radius: CornerRadius, paint: &Paint) {
        let rect = skia_safe::RRect::new_rect_radii(
            skia_safe::Rect::new(
                rect.min.x, rect.min.y, rect.max.x, rect.max.y,
            ),
            &[
                skia_safe::Point::new(radius.top_left, radius.top_left),
                skia_safe::Point::new(radius.top_right, radius.top_right),
                skia_safe::Point::new(radius.bottom_right, radius.bottom_right),
                skia_safe::Point::new(radius.bottom_left, radius.bottom_left),
            ],
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
            skia_safe::Rect::new(
                rect.min.x, rect.min.y, rect.max.x, rect.max.y,
            ),
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

    fn draw_recording(&mut self, recoding: &Recording) {
        let weak = Recording::downgrade(recoding);

        if let Some((image, size)) = self.painter.recordings.get(&weak) {
            self.canvas.draw_image_rect_with_sampling_options(
                image,
                None,
                skia_safe::Rect::new(0.0, 0.0, size.width, size.height),
                skia_safe::SamplingOptions::new(
                    skia_safe::FilterMode::Linear,
                    skia_safe::MipmapMode::None,
                ),
                &skia_safe::Paint::default(),
            );
        } else {
            tracing::error!("invalid recording drawn");
        }
    }
}
