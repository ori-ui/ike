use skia_safe::{Color, FontStyle};

use crate::{Size, text::Paragraph};

pub trait Fonts {
    fn measure(&mut self, paragraph: &Paragraph) -> Size;
}

pub trait Canvas {
    fn text(&mut self, text: &str);
}

pub struct SkiaFonts {}

impl Canvas for &skia_safe::Canvas {
    fn text(&mut self, text: &str) {
        let fonts = skia_safe::FontMgr::new();
        let font = fonts
            .match_family("Ubuntu")
            .match_style(FontStyle::normal())
            .unwrap();

        let font = skia_safe::Font::new(font, Some(20.0));
        let blob = skia_safe::TextBlob::new(text, &font);

        let mut paint = skia_safe::Paint::default();
        paint.set_color(Color::BLACK);

        self.draw_text_blob(blob.unwrap(), skia_safe::Point::new(20.0, 200.0), &paint);
    }
}
