use ike_core::{BorderWidth, Builder, Color, CornerRadius, Padding, Transition, WidgetId, widgets};
use ori::{Action, Event, Provider, View, ViewMarker};

use crate::{Context, Palette};

pub fn vscroll<V>(contents: V) -> Scroll<V> {
    Scroll::new(contents).vertical(true)
}

pub fn hscroll<V>(contents: V) -> Scroll<V> {
    Scroll::new(contents).horizontal(true)
}

#[derive(Clone, Debug)]
pub struct ScrollTheme {
    pub bar_width:          f32,
    pub bar_padding:        Padding,
    pub bar_border_width:   BorderWidth,
    pub bar_corner_radius:  CornerRadius,
    pub knob_corner_radius: CornerRadius,
    pub transition:         Transition,
    pub bar_border_color:   Option<Color>,
    pub bar_color:          Option<Color>,
    pub knob_color:         Option<Color>,
}

impl Default for ScrollTheme {
    fn default() -> Self {
        Self {
            bar_width:          8.0,
            bar_padding:        Padding::all(4.0),
            bar_border_width:   BorderWidth::all(0.0),
            bar_corner_radius:  CornerRadius::all(0.0),
            knob_corner_radius: CornerRadius::all(4.0),
            transition:         Transition::ease(0.1),
            bar_border_color:   None,
            bar_color:          None,
            knob_color:         None,
        }
    }
}

pub struct Scroll<V> {
    contents:   V,
    properties: Properties,
}

impl<V> Scroll<V> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            properties: Properties {
                vertical:           false,
                horizontal:         false,
                overlay:            false,
                bar_width:          None,
                bar_padding:        None,
                bar_border_width:   None,
                bar_corner_radius:  None,
                knob_corner_radius: None,
                transition:         None,
                bar_border_color:   None,
                bar_color:          None,
                knob_color:         None,
            },
        }
    }

    pub fn vertical(mut self, vertical: bool) -> Self {
        self.properties.vertical = vertical;
        self
    }

    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.properties.horizontal = horizontal;
        self
    }

    pub fn overlay(mut self, overlay: bool) -> Self {
        self.properties.overlay = overlay;
        self
    }

    pub fn bar_width(mut self, width: f32) -> Self {
        self.properties.bar_width = Some(width);
        self
    }

    pub fn bar_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.properties.bar_padding = Some(padding.into());
        self
    }

    pub fn bar_border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.properties.bar_border_width = Some(border_width.into());
        self
    }

    pub fn bar_corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.properties.bar_corner_radius = Some(corner_radius.into());
        self
    }

    pub fn knob_corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.properties.knob_corner_radius = Some(corner_radius.into());
        self
    }

    pub fn bar_border_color(mut self, color: Color) -> Self {
        self.properties.bar_border_color = Some(color);
        self
    }

    pub fn bar_color(mut self, color: Color) -> Self {
        self.properties.bar_color = Some(color);
        self
    }

    pub fn knob_color(mut self, color: Color) -> Self {
        self.properties.knob_color = Some(color);
        self
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.properties.transition = Some(transition);
        self
    }
}

pub struct Properties {
    vertical:           bool,
    horizontal:         bool,
    overlay:            bool,
    bar_width:          Option<f32>,
    bar_padding:        Option<Padding>,
    bar_border_width:   Option<BorderWidth>,
    bar_corner_radius:  Option<CornerRadius>,
    knob_corner_radius: Option<CornerRadius>,
    transition:         Option<Transition>,
    bar_border_color:   Option<Color>,
    bar_color:          Option<Color>,
    knob_color:         Option<Color>,
}

impl Properties {
    fn get_bar_width(&self, theme: &ScrollTheme) -> f32 {
        self.bar_width.unwrap_or(theme.bar_width)
    }

    fn get_bar_padding(&self, theme: &ScrollTheme) -> Padding {
        self.bar_padding.unwrap_or(theme.bar_padding)
    }

    fn get_bar_border_width(&self, theme: &ScrollTheme) -> BorderWidth {
        self.bar_border_width.unwrap_or(theme.bar_border_width)
    }

    fn get_bar_corner_radius(&self, theme: &ScrollTheme) -> CornerRadius {
        self.bar_corner_radius.unwrap_or(theme.bar_corner_radius)
    }

    fn get_transition(&self, theme: &ScrollTheme) -> Transition {
        self.transition.unwrap_or(theme.transition)
    }

    fn get_bar_border_paint(&self, theme: &ScrollTheme, palette: &Palette) -> Color {
        self.bar_border_color
            .unwrap_or_else(|| theme.bar_border_color.unwrap_or(palette.outline))
    }

    fn get_bar_paint(&self, theme: &ScrollTheme, _palette: &Palette) -> Color {
        self.bar_color
            .unwrap_or_else(|| theme.bar_color.unwrap_or(Color::TRANSPARENT))
    }

    fn get_knob_paint(&self, theme: &ScrollTheme, palette: &Palette) -> Color {
        self.knob_color
            .unwrap_or_else(|| theme.knob_color.unwrap_or(palette.contrast))
    }

    fn get_knob_corner_radius(&self, theme: &ScrollTheme) -> CornerRadius {
        self.knob_corner_radius.unwrap_or(theme.knob_corner_radius)
    }
}

impl<V> ViewMarker for Scroll<V> {}
impl<T, V> View<Context, T> for Scroll<V>
where
    V: crate::View<T>,
{
    type Element = WidgetId<widgets::Scroll>;
    type State = (Properties, V::Element, V::State);

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.contents.build(cx, data);

        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<ScrollTheme>();

        let mut widget = widgets::Scroll::new(cx, element);

        let bar_width = self.properties.get_bar_width(&theme);
        let bar_padding = self.properties.get_bar_padding(&theme);
        let bar_border_width = self.properties.get_bar_border_width(&theme);
        let bar_corner_radius = self.properties.get_bar_corner_radius(&theme);
        let knob_corner_radius = self.properties.get_knob_corner_radius(&theme);
        let transition = self.properties.get_transition(&theme);
        let bar_border_color = self.properties.get_bar_border_paint(&theme, &palette);
        let bar_color = self.properties.get_bar_paint(&theme, &palette);
        let knob_color = self.properties.get_knob_paint(&theme, &palette);

        widgets::Scroll::set_overlay(&mut widget, self.properties.overlay);
        widgets::Scroll::set_vertical(&mut widget, self.properties.vertical);
        widgets::Scroll::set_horizontal(&mut widget, self.properties.horizontal);
        widgets::Scroll::set_bar_thickness(&mut widget, bar_width);
        widgets::Scroll::set_bar_padding(&mut widget, bar_padding);
        widgets::Scroll::set_bar_border_width(&mut widget, bar_border_width);
        widgets::Scroll::set_bar_corner_radius(&mut widget, bar_corner_radius);
        widgets::Scroll::set_knob_corner_radius(&mut widget, knob_corner_radius);
        widgets::Scroll::set_transition(&mut widget, transition);
        widgets::Scroll::set_bar_border_paint(&mut widget, bar_border_color.into());
        widgets::Scroll::set_bar_paint(&mut widget, bar_color.into());
        widgets::Scroll::set_knob_paint(&mut widget, knob_color.into());

        (
            widget.id(),
            (self.properties, element, state),
        )
    }

    fn rebuild(
        self,
        element: &mut Self::Element,
        (properties, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.rebuild(contents, state, cx, data);

        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<ScrollTheme>();

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.properties.overlay != properties.overlay {
            widgets::Scroll::set_overlay(&mut widget, self.properties.overlay);
        }

        if self.properties.vertical != properties.vertical {
            widgets::Scroll::set_vertical(&mut widget, self.properties.vertical);
        }

        if self.properties.horizontal != properties.horizontal {
            widgets::Scroll::set_horizontal(&mut widget, self.properties.horizontal);
        }

        if self.properties.bar_width != properties.bar_width {
            let bar_width = self.properties.get_bar_width(&theme);
            widgets::Scroll::set_bar_thickness(&mut widget, bar_width);
        }

        if self.properties.bar_padding != properties.bar_padding {
            let bar_padding = self.properties.get_bar_padding(&theme);
            widgets::Scroll::set_bar_padding(&mut widget, bar_padding);
        }

        if self.properties.bar_border_width != properties.bar_border_width {
            let bar_border_width = self.properties.get_bar_border_width(&theme);
            widgets::Scroll::set_bar_border_width(&mut widget, bar_border_width);
        }

        if self.properties.bar_corner_radius != properties.bar_corner_radius {
            let bar_corner_radius = self.properties.get_bar_corner_radius(&theme);
            widgets::Scroll::set_bar_corner_radius(&mut widget, bar_corner_radius);
        }

        if self.properties.knob_corner_radius != properties.knob_corner_radius {
            let knob_corner_radius = self.properties.get_knob_corner_radius(&theme);
            widgets::Scroll::set_knob_corner_radius(&mut widget, knob_corner_radius);
        }

        if self.properties.transition != properties.transition {
            let transition = self.properties.get_transition(&theme);
            widgets::Scroll::set_transition(&mut widget, transition);
        }

        if self.properties.bar_border_color != properties.bar_border_color {
            let bar_border_color = self.properties.get_bar_border_paint(&theme, &palette);
            widgets::Scroll::set_bar_border_paint(&mut widget, bar_border_color.into());
        }

        if self.properties.bar_color != properties.bar_color {
            let bar_color = self.properties.get_bar_paint(&theme, &palette);
            widgets::Scroll::set_bar_paint(&mut widget, bar_color.into());
        }

        if self.properties.knob_color != properties.knob_color {
            let knob_color = self.properties.get_knob_paint(&theme, &palette);
            widgets::Scroll::set_knob_paint(&mut widget, knob_color.into());
        }

        *properties = self.properties;
    }

    fn event(
        _element: &mut Self::Element,
        (_properties, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        V::event(contents, state, cx, data, event)
    }

    fn teardown(
        element: Self::Element,
        (_properties, contents, state): Self::State,
        cx: &mut Context,
    ) {
        V::teardown(contents, state, cx);
        cx.remove_widget(element);
    }
}
