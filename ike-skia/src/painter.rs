use std::{collections::HashMap, hash::BuildHasherDefault, mem};

use ike_core::{
    Curve, Fill, FontStretch, FontStyle, GlyphCluster, Paint, Painter, Paragraph, Point, Rect,
    Shader, Size, Svg, TextDirection, TextLayoutLine, TextStyle, TextWrap, WeakCurve,
    WeakParagraph, WeakRecording, WeakSvg,
};

type SeaHasher = BuildHasherDefault<seahash::SeaHasher>;
type CachedParagraph = (f32, skia_safe::textlayout::Paragraph);

pub struct SkiaPainter {
    pub(crate) provider:   skia_safe::textlayout::TypefaceFontProvider,
    pub(crate) manager:    skia_safe::FontMgr,
    pub(crate) fonts:      skia_safe::textlayout::FontCollection,
    pub(crate) svgs:       HashMap<WeakSvg, Option<skia_safe::svg::Dom>, SeaHasher>,
    pub(crate) paragraphs: HashMap<WeakParagraph, CachedParagraph, SeaHasher>,
    pub(crate) recordings: HashMap<WeakRecording, (skia_safe::Image, Size), SeaHasher>,
    pub(crate) paths:      HashMap<WeakCurve, skia_safe::Path, SeaHasher>,
    pub(crate) paints:     HashMap<Paint, skia_safe::Paint, SeaHasher>,
}

impl Default for SkiaPainter {
    fn default() -> Self {
        Self::new()
    }
}

impl SkiaPainter {
    pub fn new() -> Self {
        let provider = skia_safe::textlayout::TypefaceFontProvider::new();
        let manager = skia_safe::FontMgr::new();
        let mut fonts = skia_safe::textlayout::FontCollection::new();
        fonts.set_dynamic_font_manager(skia_safe::FontMgr::clone(&provider));
        fonts.set_default_font_manager(manager.clone(), None);

        Self {
            provider,
            manager,
            fonts,
            svgs: HashMap::default(),
            paragraphs: HashMap::default(),
            recordings: HashMap::default(),
            paths: HashMap::default(),
            paints: HashMap::default(),
        }
    }

    pub fn cleanup(&mut self) {
        self.svgs.retain(|k, _| k.strong_count() > 0);
        self.paragraphs.retain(|k, _| k.strong_count() > 0);
        self.recordings.retain(|k, _| k.strong_count() > 0);
        self.paths.retain(|k, _| k.strong_count() > 0);
        self.paints.clear();
    }

    pub fn load_font(&mut self, bytes: &[u8], alias: Option<&str>) {
        if let Some(typeface) = self.manager.new_from_data(bytes, None) {
            self.provider.register_typeface(typeface, alias);
        } else {
            tracing::warn!("loading font failed");
        }
    }

    pub(crate) fn create_svg(&mut self, svg: &Svg) -> Option<skia_safe::svg::Dom> {
        let weak = Svg::downgrade(svg);

        self.svgs
            .entry(weak)
            .or_insert_with(|| {
                let dom = skia_safe::svg::Dom::from_bytes(
                    svg.bytes(),
                    self.manager.clone(), // reuse font manager
                )
                .ok()?;

                let mut svg = dom.root();

                if svg.intrinsic_size().is_zero() {
                    svg.set_height(skia_safe::svg::Length::new(
                        1.0,
                        skia_safe::svg::LengthUnit::PX,
                    ));
                    svg.set_width(skia_safe::svg::Length::new(
                        1.0,
                        skia_safe::svg::LengthUnit::PX,
                    ));
                }

                Some(dom)
            })
            .clone()
    }

    fn create_font_style(style: &TextStyle) -> skia_safe::FontStyle {
        let weight = skia_safe::font_style::Weight::from(style.font_weight.0 as i32);

        let width = match style.font_stretch {
            FontStretch::UltraCondensed => skia_safe::font_style::Width::ULTRA_CONDENSED,
            FontStretch::ExtraCondensed => skia_safe::font_style::Width::EXTRA_CONDENSED,
            FontStretch::Condensed => skia_safe::font_style::Width::CONDENSED,
            FontStretch::SemiCondensed => skia_safe::font_style::Width::SEMI_CONDENSED,
            FontStretch::Normal => skia_safe::font_style::Width::NORMAL,
            FontStretch::SemiExpanded => skia_safe::font_style::Width::SEMI_EXPANDED,
            FontStretch::Expanded => skia_safe::font_style::Width::EXPANDED,
            FontStretch::ExtraExpanded => skia_safe::font_style::Width::EXTRA_EXPANDED,
            FontStretch::UltraExpanded => skia_safe::font_style::Width::ULTRA_EXPANDED,
        };

        let slant = match style.font_style {
            FontStyle::Normal => skia_safe::font_style::Slant::Upright,
            FontStyle::Italic => skia_safe::font_style::Slant::Italic,
            FontStyle::Oblique => skia_safe::font_style::Slant::Oblique,
        };

        skia_safe::FontStyle::new(weight, width, slant)
    }

    pub(crate) fn create_paragraph(
        &mut self,
        paragraph: &Paragraph,
        max_width: f32,
    ) -> &mut skia_safe::textlayout::Paragraph {
        let weak = Paragraph::downgrade(paragraph);

        if !self.paragraphs.contains_key(&weak) {
            let mut style = skia_safe::textlayout::ParagraphStyle::new();

            let align = match paragraph.align {
                ike_core::TextAlign::Start => skia_safe::textlayout::TextAlign::Start,
                ike_core::TextAlign::Center => skia_safe::textlayout::TextAlign::Center,
                ike_core::TextAlign::End => skia_safe::textlayout::TextAlign::End,
            };

            style.set_height(paragraph.line_height);
            style.set_text_align(align);

            if let TextWrap::None = paragraph.wrap {
                style.set_max_lines(1);
            }

            let mut builder = skia_safe::textlayout::ParagraphBuilder::new(&style, &self.fonts);

            for (text, style) in paragraph.sections() {
                let mut skia_style = skia_safe::textlayout::TextStyle::new();

                skia_style.set_subpixel(true);
                skia_style.set_font_size(style.font_size);
                skia_style.set_font_families(&[&style.font_family]);
                skia_style.set_font_style(Self::create_font_style(style));

                let paint = self.create_paint(&style.paint);
                skia_style.set_foreground_paint(paint);

                builder.push_style(&skia_style);
                builder.add_text(text);
                builder.pop();
            }

            let mut paragraph = builder.build();
            paragraph.layout(max_width);

            self.paragraphs.insert(weak.clone(), (max_width, paragraph));
        }

        let (current_max_width, paragraph) = self
            .paragraphs
            .get_mut(&weak)
            .expect("inserted if not contained");

        if *current_max_width != max_width {
            paragraph.layout(max_width);
            *current_max_width = max_width;
        }

        paragraph
    }

    pub(crate) fn create_path(&mut self, curve: &Curve) -> &skia_safe::Path {
        let weak = Curve::downgrade(curve);

        self.paths.entry(weak).or_insert_with(|| {
            let mut path = skia_safe::PathBuilder::new();

            let fill_type = match curve.fill {
                Fill::Winding => skia_safe::PathFillType::Winding,
                Fill::EvenOdd => skia_safe::PathFillType::EvenOdd,
            };

            path.set_fill_type(fill_type);

            for segment in curve.iter() {
                match segment {
                    ike_core::CurveSegment::Move(p) => {
                        path.move_to(skia_safe::Point::new(p.x, p.y));
                    }

                    ike_core::CurveSegment::Line(p) => {
                        path.line_to(skia_safe::Point::new(p.x, p.y));
                    }

                    ike_core::CurveSegment::Quad(a, p) => {
                        path.quad_to(
                            skia_safe::Point::new(a.x, a.y),
                            skia_safe::Point::new(p.x, p.y),
                        );
                    }

                    ike_core::CurveSegment::Cubic(a, b, p) => {
                        path.cubic_to(
                            skia_safe::Point::new(a.x, a.y),
                            skia_safe::Point::new(b.x, b.y),
                            skia_safe::Point::new(p.x, p.y),
                        );
                    }

                    ike_core::CurveSegment::Close => {
                        path.close();
                    }
                }
            }

            path.into()
        })
    }

    pub(crate) fn create_paint(&mut self, paint: &Paint) -> &skia_safe::Paint {
        self.paints.entry(paint.clone()).or_insert_with(|| {
            let mut skia_paint = skia_safe::Paint::default();
            skia_paint.set_anti_alias(true);

            match paint.shader {
                Shader::Solid(color) => {
                    skia_paint.set_color4f(
                        skia_safe::Color4f::new(color.r, color.g, color.b, color.a),
                        None,
                    );
                }
            }

            let blend = match paint.blend {
                ike_core::Blend::Clear => skia_safe::BlendMode::Clear,
                ike_core::Blend::Src => skia_safe::BlendMode::Src,
                ike_core::Blend::Dst => skia_safe::BlendMode::Dst,
                ike_core::Blend::SrcOver => skia_safe::BlendMode::SrcOver,
                ike_core::Blend::DstOver => skia_safe::BlendMode::DstOver,
                ike_core::Blend::SrcIn => skia_safe::BlendMode::SrcIn,
                ike_core::Blend::DstIn => skia_safe::BlendMode::DstIn,
                ike_core::Blend::SrcATop => skia_safe::BlendMode::SrcATop,
                ike_core::Blend::DstATop => skia_safe::BlendMode::DstATop,
            };

            skia_paint.set_blend_mode(blend);

            skia_paint
        })
    }
}

impl Painter for SkiaPainter {
    fn measure_svg(&mut self, svg: &Svg) -> Size {
        if let Some(skia_dom) = self.create_svg(svg) {
            let size = skia_dom.root().intrinsic_size();
            Size::new(size.width, size.height)
        } else {
            Size::ZERO
        }
    }

    fn measure_text(&mut self, paragraph: &Paragraph, max_width: f32) -> Size {
        let mut min_height = 0.0;

        if let Some((_, style)) = paragraph.sections().next() {
            let typefaces = self.fonts.find_typefaces(
                &[&style.font_family],
                Self::create_font_style(style),
            );

            if let Some(typeface) = typefaces.first() {
                let font = skia_safe::Font::new(typeface, style.font_size);
                let (_, metrics) = font.metrics();

                min_height = metrics.descent - metrics.ascent + metrics.leading;
            }
        }

        let paragraph = self.create_paragraph(paragraph, max_width);

        Size {
            width:  paragraph.max_intrinsic_width(),
            height: paragraph.height().max(min_height),
        }
    }

    fn layout_text(
        &mut self,
        paragraph: &Paragraph,
        max_width: f32,
    ) -> Vec<ike_core::TextLayoutLine> {
        let skia = self.create_paragraph(paragraph, max_width);

        let mut lines = Vec::new();
        let mut glyphs = Vec::new();
        let mut prev_start = 0;
        let mut prev_end = 0;

        fn create_line(
            metrics: &skia_safe::textlayout::LineMetrics,
            glyphs: &mut Vec<GlyphCluster>,
            start: usize,
            end: usize,
        ) -> TextLayoutLine {
            let start_index = glyphs.first().map_or(start, |glyph| glyph.start_index);
            let end_index = glyphs.last().map_or(end, |glyph| glyph.end_index);

            let left = glyphs.first().map_or(0.0, |glyph| glyph.bounds.left());
            let right = glyphs.last().map_or(0.0, |glyph| glyph.bounds.right());

            TextLayoutLine {
                ascent: metrics.ascent as f32,
                descent: metrics.descent as f32,
                left,
                width: right - left,
                height: metrics.height as f32,
                baseline: metrics.baseline as f32,
                start_index,
                end_index,
                glyphs: mem::take(glyphs),
            }
        }

        let metrics = skia.get_line_metrics();

        for (i, c) in paragraph.text.char_indices() {
            let Some(glyph) = skia.get_glyph_cluster_at(i) else {
                continue;
            };

            let bounds = Rect::min_size(
                Point::new(glyph.bounds.x(), glyph.bounds.y()),
                Size::new(
                    glyph.bounds.width(),
                    glyph.bounds.height(),
                ),
            );

            let direction = match glyph.position {
                skia_safe::textlayout::TextDirection::LTR => TextDirection::Ltr,
                skia_safe::textlayout::TextDirection::RTL => TextDirection::Rtl,
            };

            let glyph = GlyphCluster {
                bounds,
                start_index: glyph.text_range.start,
                end_index: glyph.text_range.end,
                direction,
            };

            if c == '\n' || skia.get_line_number_at(i) != Some(lines.len()) {
                lines.push(create_line(
                    &metrics[lines.len()],
                    &mut glyphs,
                    prev_start,
                    prev_end,
                ));
            }

            prev_start = glyph.start_index;
            prev_end = glyph.end_index;

            if c != '\n' {
                glyphs.push(glyph);
            }
        }

        if lines.len() < metrics.len() {
            lines.push(create_line(
                &metrics[lines.len()],
                &mut glyphs,
                prev_start,
                prev_end,
            ));
        }

        assert_eq!(lines.len(), metrics.len());

        lines
    }
}
