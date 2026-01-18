use ike_core::{
    AnyWidgetId, BorderWidth, Builder, Color, CornerRadius, Padding, WidgetId, widgets,
};
use ori::{Action, Event, Provider, View, ViewMarker};

use crate::Palette;

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
    contents:         V,
    padding:          Option<Padding>,
    border_width:     Option<BorderWidth>,
    corner_radius:    Option<CornerRadius>,
    background_color: Option<Color>,
    border_color:     Option<Color>,
}

impl<V> Container<V> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,

            padding: None,
            border_width: None,
            corner_radius: None,
            background_color: None,
            border_color: None,
        }
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.border_width = Some(border_width.into());
        self
    }

    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = Some(corner_radius.into());
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

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
impl<C, T, V> View<C, T> for Container<V>
where
    C: Builder + Provider,
    V: View<C, T, Element: AnyWidgetId>,
{
    type Element = WidgetId<widgets::Container>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<ContainerTheme>();

        let mut widget = widgets::Container::new(cx, contents.upcast());

        let padding = self.get_padding(&theme);
        let border_width = self.get_border_width(&theme);
        let corner_radius = self.get_corner_radius(&theme);
        let background_color = self.get_background_color(&theme, &palette);
        let border_color = self.get_border_color(&theme, &palette);

        widgets::Container::set_padding(&mut widget, padding);
        widgets::Container::set_border_width(&mut widget, border_width);
        widgets::Container::set_corner_radius(&mut widget, corner_radius);
        widgets::Container::set_background_color(&mut widget, background_color);
        widgets::Container::set_border_color(&mut widget, border_color);

        (widget.id(), (contents, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.rebuild(
            contents,
            state,
            cx,
            data,
            &mut old.contents,
        );

        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<ContainerTheme>();

        if !cx.is_child(*element, *contents) {
            cx.set_child(*element, 0, *contents);
        }

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.padding != old.padding {
            let padding = self.get_padding(&theme);
            widgets::Container::set_padding(&mut widget, padding);
        }

        if self.border_width != old.border_width {
            let border_width = self.get_border_width(&theme);
            widgets::Container::set_border_width(&mut widget, border_width);
        }

        if self.corner_radius != old.corner_radius {
            let corner_radius = self.get_corner_radius(&theme);
            widgets::Container::set_corner_radius(&mut widget, corner_radius);
        }

        if self.background_color != old.background_color {
            let background = self.get_background_color(&theme, &palette);
            widgets::Container::set_background_color(&mut widget, background);
        }

        if self.border_color != old.border_color {
            let border_color = self.get_border_color(&theme, &palette);
            widgets::Container::set_border_color(&mut widget, border_color);
        }
    }

    fn teardown(&mut self, element: Self::Element, (contents, state): Self::State, cx: &mut C) {
        self.contents.teardown(contents, state, cx);
        cx.remove_widget(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let action = self.contents.event(contents, state, cx, data, event);

        if !cx.is_child(*element, *contents) {
            cx.set_child(*element, 0, *contents);
        }

        action
    }
}
