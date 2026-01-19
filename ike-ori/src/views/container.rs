use ike_core::{
    AnyWidgetId, BorderWidth, Builder, Color, CornerRadius, Padding, WidgetId, widgets,
};
use ori::{Action, Event, Provider, View, ViewMarker};

use crate::{Context, Palette};

pub fn container<V>(contents: V) -> Container<V> {
    Container::new(contents)
}

#[derive(Clone, Debug)]
pub struct ContainerTheme {
    pub padding:          Padding,
    pub border_width:     BorderWidth,
    pub corner_radius:    CornerRadius,
    pub background_color: Option<Color>,
    pub border_color:     Option<Color>,
}

impl Default for ContainerTheme {
    fn default() -> Self {
        Self {
            padding:          Padding::all(8.0),
            border_width:     BorderWidth::all(1.0),
            corner_radius:    CornerRadius::all(8.0),
            background_color: None,
            border_color:     None,
        }
    }
}

pub struct Container<V> {
    contents:   V,
    properties: Properties,
}

impl<V> Container<V> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,

            properties: Properties {
                padding:          None,
                border_width:     None,
                corner_radius:    None,
                background_color: None,
                border_color:     None,
            },
        }
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.properties.padding = Some(padding.into());
        self
    }

    pub fn border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.properties.border_width = Some(border_width.into());
        self
    }

    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.properties.corner_radius = Some(corner_radius.into());
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.properties.background_color = Some(color);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.properties.border_color = Some(color);
        self
    }
}

pub struct Properties {
    padding:          Option<Padding>,
    border_width:     Option<BorderWidth>,
    corner_radius:    Option<CornerRadius>,
    background_color: Option<Color>,
    border_color:     Option<Color>,
}

impl Properties {
    fn get_padding(&self, theme: &ContainerTheme) -> Padding {
        self.padding.unwrap_or(theme.padding)
    }

    fn get_border_width(&self, theme: &ContainerTheme) -> BorderWidth {
        self.border_width.unwrap_or(theme.border_width)
    }

    fn get_corner_radius(&self, theme: &ContainerTheme) -> CornerRadius {
        self.corner_radius.unwrap_or(theme.corner_radius)
    }

    fn get_background_color(&self, theme: &ContainerTheme, palette: &Palette) -> Color {
        self.background_color
            .unwrap_or_else(|| theme.background_color.unwrap_or(palette.surface))
    }

    fn get_border_color(&self, theme: &ContainerTheme, palette: &Palette) -> Color {
        self.border_color
            .unwrap_or_else(|| theme.border_color.unwrap_or(palette.outline))
    }
}

impl<V> ViewMarker for Container<V> {}
impl<T, V> View<Context, T> for Container<V>
where
    V: crate::View<T>,
{
    type Element = WidgetId<widgets::Container>;
    type State = (Properties, V::Element, V::State);

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<ContainerTheme>();

        let mut widget = widgets::Container::new(cx, contents.upcast());

        let padding = self.properties.get_padding(&theme);
        let border_width = self.properties.get_border_width(&theme);
        let corner_radius = self.properties.get_corner_radius(&theme);
        let background_color = self.properties.get_background_color(&theme, &palette);
        let border_color = self.properties.get_border_color(&theme, &palette);

        widgets::Container::set_padding(&mut widget, padding);
        widgets::Container::set_border_width(&mut widget, border_width);
        widgets::Container::set_corner_radius(&mut widget, corner_radius);
        widgets::Container::set_background_color(&mut widget, background_color);
        widgets::Container::set_border_color(&mut widget, border_color);

        (
            widget.id(),
            (self.properties, contents, state),
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
        let theme = cx.get_or_default::<ContainerTheme>();

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.properties.padding != properties.padding {
            let padding = self.properties.get_padding(&theme);
            widgets::Container::set_padding(&mut widget, padding);
        }

        if self.properties.border_width != properties.border_width {
            let border_width = self.properties.get_border_width(&theme);
            widgets::Container::set_border_width(&mut widget, border_width);
        }

        if self.properties.corner_radius != properties.corner_radius {
            let corner_radius = self.properties.get_corner_radius(&theme);
            widgets::Container::set_corner_radius(&mut widget, corner_radius);
        }

        if self.properties.background_color != properties.background_color {
            let background = self.properties.get_background_color(&theme, &palette);
            widgets::Container::set_background_color(&mut widget, background);
        }

        if self.properties.border_color != properties.border_color {
            let border_color = self.properties.get_border_color(&theme, &palette);
            widgets::Container::set_border_color(&mut widget, border_color);
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
