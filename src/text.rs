use crate::{Color, Rect, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextWrap {
    None,
    Word,
}

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub line_height: f32,
    pub align:       TextAlign,
    pub wrap:        TextWrap,
    pub text:        String,
    pub sections:    Vec<(usize, TextStyle)>,
}

impl Paragraph {
    pub const fn new(line_height: f32, align: TextAlign, wrap: TextWrap) -> Self {
        Self {
            line_height,
            align,
            wrap,
            text: String::new(),
            sections: Vec::new(),
        }
    }

    pub fn push(&mut self, text: impl AsRef<str>, style: TextStyle) {
        self.sections.push((self.text.len(), style));
        self.text.push_str(text.as_ref());
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.sections.clear();
    }

    pub fn sections(&self) -> impl Iterator<Item = (&str, &TextStyle)> {
        let last = self
            .sections
            .last()
            .map(|(start, style)| (&self.text[*start..], style));

        self.sections
            .windows(2)
            .map(|sections| {
                let (start, ref style) = sections[0];
                let (end, _) = sections[1];
                (&self.text[start..end], style)
            })
            .chain(last)
    }
}

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub font_size:    f32,
    pub font_family:  String,
    pub font_weight:  FontWeight,
    pub font_stretch: FontStretch,
    pub font_style:   FontStyle,
    pub color:        Color,
}

/// A font weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontWeight(pub u16);

impl FontWeight {
    /// Thin font weight (100), the thinnest possible.
    pub const THIN: Self = Self(100);
    /// Extra-light font weight (200).
    pub const EXTRA_LIGHT: Self = Self(200);
    /// Light font weight (300).
    pub const LIGHT: Self = Self(300);
    /// Normal font weight (400), the default.
    pub const NORMAL: Self = Self(400);
    /// Medium font weight (500).
    pub const MEDIUM: Self = Self(500);
    /// Semi-bold font weight (600).
    pub const SEMI_BOLD: Self = Self(600);
    /// Bold font weight (700).
    pub const BOLD: Self = Self(700);
    /// Extra-bold font weight (800).
    pub const EXTRA_BOLD: Self = Self(800);
    /// Black font weight (900), the boldest possible.
    pub const BLACK: Self = Self(900);
}

/// A font stretch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontStretch {
    /// Ultra-condensed font stretch.
    UltraCondensed,

    /// Extra-condensed font stretch.
    ExtraCondensed,

    /// Condensed font stretch.
    Condensed,

    /// Semi-condensed font stretch.
    SemiCondensed,

    /// Normal font stretch, the default.
    #[default]
    Normal,

    /// Semi-expanded font stretch.
    SemiExpanded,

    /// Expanded font stretch.
    Expanded,

    /// Extra-expanded font stretch.
    ExtraExpanded,

    /// Ultra-expanded font stretch.
    UltraExpanded,
}

/// A font style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontStyle {
    /// Normal font style, the default.
    #[default]
    Normal,

    /// Italic font style.
    Italic,

    /// Oblique font style.
    Oblique,
}

pub trait Fonts {
    fn measure(&mut self, paragraph: &Paragraph, max_width: f32) -> Size;
    fn layout(&mut self, paragraph: &Paragraph, max_width: f32) -> Vec<TextLayoutLine>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayoutLine {
    pub ascent:      f32,
    pub descent:     f32,
    pub left:        f32,
    pub width:       f32,
    pub height:      f32,
    pub baseline:    f32,
    pub start_index: usize,
    pub end_index:   usize,
    pub glyphs:      Vec<GlyphCluster>,
}

impl TextLayoutLine {
    pub fn left(&self) -> f32 {
        self.left
    }

    pub fn right(&self) -> f32 {
        self.left + self.width
    }

    pub fn top(&self) -> f32 {
        self.baseline - self.ascent
    }

    pub fn bottom(&self) -> f32 {
        self.baseline + self.descent
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyphCluster {
    pub bounds:      Rect,
    pub start_index: usize,
    pub end_index:   usize,
    pub direction:   TextDirection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextDirection {
    Ltr,
    Rtl,
}
