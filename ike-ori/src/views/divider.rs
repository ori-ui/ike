use ike_core::{Axis, Builder, Color, CornerRadius, WidgetId, widgets};
use ori::{Action, Event, Provider, View, ViewMarker};

use crate::{Context, Palette};

pub fn divider(axis: Axis) -> Divider {
    Divider::new(axis)
}

pub fn hdivider() -> Divider {
    divider(Axis::Horizontal)
}

pub fn vdivider() -> Divider {
    divider(Axis::Vertical)
}

#[derive(Clone, Debug)]
pub struct DividerTheme {
    pub thickness:     f32,
    pub inset:         f32,
    pub padding:       f32,
    pub corner_radius: CornerRadius,
    pub color:         Option<Color>,
}

impl Default for DividerTheme {
    fn default() -> Self {
        Self {
            thickness:     1.0,
            inset:         8.0,
            padding:       8.0,
            corner_radius: CornerRadius::all(0.0),
            color:         None,
        }
    }
}

pub struct Divider {
    properties: Properties,
}

impl Divider {
    pub fn new(axis: Axis) -> Self {
        Self {
            properties: Properties {
                axis,
                thickness: None,
                inset: None,
                padding: None,
                corner_radius: None,
                color: None,
            },
        }
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.properties.thickness = Some(thickness);
        self
    }

    pub fn inset(mut self, inset: f32) -> Self {
        self.properties.inset = Some(inset);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.properties.padding = Some(padding);
        self
    }

    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.properties.corner_radius = Some(corner_radius.into());
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.properties.color = Some(color);
        self
    }
}

pub struct Properties {
    axis:          Axis,
    thickness:     Option<f32>,
    inset:         Option<f32>,
    padding:       Option<f32>,
    corner_radius: Option<CornerRadius>,
    color:         Option<Color>,
}

impl Properties {
    fn get_thickness(&self, theme: &DividerTheme) -> f32 {
        self.thickness.unwrap_or(theme.thickness)
    }

    fn get_inset(&self, theme: &DividerTheme) -> f32 {
        self.inset.unwrap_or(theme.inset)
    }

    fn get_padding(&self, theme: &DividerTheme) -> f32 {
        self.padding.unwrap_or(theme.padding)
    }

    fn get_corner_radius(&self, theme: &DividerTheme) -> CornerRadius {
        self.corner_radius.unwrap_or(theme.corner_radius)
    }

    fn get_color(&self, theme: &DividerTheme, palette: &Palette) -> Color {
        self.color
            .unwrap_or_else(|| theme.color.unwrap_or(palette.outline))
    }
}

impl ViewMarker for Divider {}
impl<T> View<Context, T> for Divider {
    type Element = WidgetId<widgets::Divider>;
    type State = Properties;

    fn build(self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<DividerTheme>();

        let mut widget = widgets::Divider::new(cx);

        let thickness = self.properties.get_thickness(&theme);
        let inset = self.properties.get_inset(&theme);
        let padding = self.properties.get_padding(&theme);
        let corner_radius = self.properties.get_corner_radius(&theme);
        let color = self.properties.get_color(&theme, &palette);

        widgets::Divider::set_axis(&mut widget, self.properties.axis);
        widgets::Divider::set_thickness(&mut widget, thickness);
        widgets::Divider::set_inset(&mut widget, inset);
        widgets::Divider::set_padding(&mut widget, padding);
        widgets::Divider::set_corner_radius(&mut widget, corner_radius);
        widgets::Divider::set_color(&mut widget, color);

        (widget.id(), self.properties)
    }

    fn rebuild(
        self,
        element: &mut Self::Element,
        properties: &mut Self::State,
        cx: &mut Context,
        _data: &mut T,
    ) {
        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<DividerTheme>();

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.properties.axis != properties.axis {
            widgets::Divider::set_axis(&mut widget, self.properties.axis);
        }

        if self.properties.thickness != properties.thickness {
            let thickness = self.properties.get_thickness(&theme);
            widgets::Divider::set_thickness(&mut widget, thickness);
        }

        if self.properties.inset != properties.inset {
            let inset = self.properties.get_inset(&theme);
            widgets::Divider::set_inset(&mut widget, inset);
        }

        if self.properties.padding != properties.padding {
            let padding = self.properties.get_padding(&theme);
            widgets::Divider::set_padding(&mut widget, padding);
        }

        if self.properties.corner_radius != properties.corner_radius {
            let corner_radius = self.properties.get_corner_radius(&theme);
            widgets::Divider::set_corner_radius(&mut widget, corner_radius);
        }

        if self.properties.color != properties.color {
            let color = self.properties.get_color(&theme, &palette);
            widgets::Divider::set_color(&mut widget, color);
        }

        *properties = self.properties;
    }

    fn event(
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _event: &mut Event,
    ) -> Action {
        Action::new()
    }

    fn teardown(element: Self::Element, _state: Self::State, cx: &mut Context) {
        cx.remove_widget(element);
    }
}
