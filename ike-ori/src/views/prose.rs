use ike_core::{
    Builder, Color, FontStretch, FontStyle, FontWeight, Paint, Paragraph, TextAlign, TextStyle,
    TextWrap, WidgetId, widgets,
};
use ori::{Action, Provider, View, ViewId, ViewMarker};

use crate::{Context, Palette, views::TextTheme};

#[derive(Clone, Debug)]
pub struct ProseTheme {
    pub font_size:       Option<f32>,
    pub font_family:     Option<String>,
    pub font_weight:     Option<FontWeight>,
    pub font_stretch:    Option<FontStretch>,
    pub font_style:      Option<FontStyle>,
    pub line_height:     Option<f32>,
    pub align:           Option<TextAlign>,
    pub wrap:            Option<TextWrap>,
    pub color:           Option<Color>,
    pub cursor_color:    Option<Color>,
    pub selection_color: Option<Color>,
    pub blink_rate:      f32,
}

impl Default for ProseTheme {
    fn default() -> Self {
        Self {
            font_size:       None,
            font_family:     None,
            font_weight:     None,
            font_stretch:    None,
            font_style:      None,
            line_height:     None,
            align:           None,
            wrap:            None,
            color:           None,
            cursor_color:    None,
            selection_color: None,
            blink_rate:      5.0,
        }
    }
}

pub fn prose(text: impl Into<String>) -> Prose {
    Prose::new(text)
}

pub struct Prose {
    text:            String,
    font_size:       Option<f32>,
    font_family:     Option<String>,
    font_weight:     Option<FontWeight>,
    font_stretch:    Option<FontStretch>,
    font_style:      Option<FontStyle>,
    line_height:     Option<f32>,
    align:           Option<TextAlign>,
    wrap:            Option<TextWrap>,
    color:           Option<Color>,
    cursor_color:    Option<Color>,
    selection_color: Option<Color>,
    blink_rate:      Option<f32>,
}

impl Prose {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text:            text.into(),
            font_size:       None,
            font_family:     None,
            font_weight:     None,
            font_stretch:    None,
            font_style:      None,
            line_height:     None,
            align:           None,
            wrap:            None,
            color:           None,
            cursor_color:    None,
            selection_color: None,
            blink_rate:      None,
        }
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn font_family(mut self, font_family: impl ToString) -> Self {
        self.font_family = Some(font_family.to_string());
        self
    }

    pub fn font_weight(mut self, font_weight: FontWeight) -> Self {
        self.font_weight = Some(font_weight);
        self
    }

    pub fn font_stretch(mut self, font_stretch: FontStretch) -> Self {
        self.font_stretch = Some(font_stretch);
        self
    }

    pub fn font_style(mut self, font_style: FontStyle) -> Self {
        self.font_style = Some(font_style);
        self
    }

    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = Some(align);
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_color = Some(color);
        self
    }

    pub fn cursor_color(mut self, color: Color) -> Self {
        self.cursor_color = Some(color);
        self
    }

    pub fn blink_rate(mut self, rate: f32) -> Self {
        self.blink_rate = Some(rate);
        self
    }
}

impl Prose {
    fn build_paragraph(
        &self,
        text: &str,
        palette: &Palette,
        text_theme: &TextTheme,
        text_area_theme: &ProseTheme,
    ) -> Paragraph {
        let style = TextStyle {
            font_size: self
                .font_size
                .unwrap_or_else(|| text_area_theme.font_size.unwrap_or(text_theme.font_size)),

            font_weight: self.font_weight.unwrap_or_else(|| {
                text_area_theme
                    .font_weight
                    .unwrap_or(text_theme.font_weight)
            }),

            font_stretch: self.font_stretch.unwrap_or_else(|| {
                text_area_theme
                    .font_stretch
                    .unwrap_or(text_theme.font_stretch)
            }),

            font_style: self
                .font_style
                .unwrap_or_else(|| text_area_theme.font_style.unwrap_or(text_theme.font_style)),

            font_family: self.font_family.clone().unwrap_or_else(|| {
                text_area_theme
                    .font_family
                    .clone()
                    .unwrap_or_else(|| text_theme.font_family.clone().into_owned())
            }),

            paint: Paint::from(self.color.unwrap_or_else(|| {
                text_area_theme
                    .color
                    .unwrap_or_else(|| text_theme.color.unwrap_or(palette.contrast))
            })),
        };

        let mut paragraph = Paragraph::new(
            self.line_height.unwrap_or_else(|| {
                text_area_theme
                    .line_height
                    .unwrap_or(text_theme.line_height)
            }),
            self.align
                .unwrap_or_else(|| text_area_theme.align.unwrap_or(text_theme.align)),
            self.wrap
                .unwrap_or_else(|| text_area_theme.wrap.unwrap_or(text_theme.wrap)),
        );

        paragraph.push(text, style);
        paragraph
    }

    fn get_cursor_color(&self, palette: &Palette, theme: &ProseTheme) -> Color {
        self.cursor_color
            .unwrap_or_else(|| theme.cursor_color.unwrap_or(palette.contrast))
    }

    fn get_selection_color(&self, palette: &Palette, theme: &ProseTheme) -> Color {
        self.selection_color
            .unwrap_or_else(|| theme.selection_color.unwrap_or(palette.info))
    }
}

impl ViewMarker for Prose {}
impl<T> View<Context, T> for Prose {
    type Element = WidgetId<widgets::TextArea<false>>;
    type State = ViewId;

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let palette = cx.get_or_default::<Palette>();
        let text_theme = cx.get_or_default::<TextTheme>();
        let theme = cx.get_or_default::<ProseTheme>();
        let id = ViewId::next();

        let paragraph = self.build_paragraph(
            &self.text,
            &palette,
            &text_theme,
            &theme,
        );

        let mut widget = widgets::TextArea::<false>::new(cx, paragraph);

        let cursor_color = self.get_cursor_color(&palette, &theme);
        let selection_color = self.get_selection_color(&palette, &theme);
        let blink_rate = self.blink_rate.unwrap_or(theme.blink_rate);

        widgets::TextArea::set_cursor_color(&mut widget, cursor_color);
        widgets::TextArea::set_selection_color(&mut widget, selection_color);
        widgets::TextArea::set_blink_rate(&mut widget, blink_rate);

        (widget.id(), id)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        let palette = cx.get_or_default::<Palette>();
        let text_theme = cx.get_or_default::<TextTheme>();
        let theme = cx.get_or_default::<ProseTheme>();

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.text != old.text
            || self.font_size != old.font_size
            || self.font_family != old.font_family
            || self.font_weight != old.font_weight
            || self.font_stretch != old.font_stretch
            || self.font_style != old.font_style
            || self.line_height != old.line_height
            || self.align != old.align
            || self.wrap != old.wrap
            || self.color != old.color
        {
            let paragraph = self.build_paragraph(
                &self.text,
                &palette,
                &text_theme,
                &theme,
            );

            widgets::TextArea::set_text(&mut widget, paragraph);
        }

        if self.cursor_color != old.cursor_color {
            let cursor_color = self.cursor_color.unwrap_or(palette.contrast);
            widgets::TextArea::set_cursor_color(&mut widget, cursor_color);
        }

        if self.selection_color != old.selection_color {
            let selection_color = self.selection_color.unwrap_or(palette.info);
            widgets::TextArea::set_selection_color(&mut widget, selection_color);
        }

        if self.blink_rate != old.blink_rate {
            let blink_rate = self.blink_rate.unwrap_or(theme.blink_rate);
            widgets::TextArea::set_blink_rate(&mut widget, blink_rate);
        }
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _id: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _event: &mut ori::Event,
    ) -> Action {
        Action::new()
    }

    fn teardown(&mut self, element: Self::Element, _state: Self::State, cx: &mut Context) {
        cx.remove_widget(element);
    }
}
