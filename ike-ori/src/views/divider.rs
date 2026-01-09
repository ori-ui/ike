use ike_core::{Axis, Builder, Color, CornerRadius, WidgetId, widgets};
use ori::{Action, Event, Providable, View, ViewMarker};

use crate::Palette;

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
    pub indent:        f32,
    pub corner_radius: CornerRadius,
    pub color:         Option<Color>,
}

impl Default for DividerTheme {
    fn default() -> Self {
        Self {
            thickness:     2.0,
            indent:        8.0,
            corner_radius: CornerRadius::all(0.0),
            color:         None,
        }
    }
}

pub struct Divider {
    axis:          Axis,
    thickness:     Option<f32>,
    indent:        Option<f32>,
    corner_radius: Option<CornerRadius>,
    color:         Option<Color>,
}

impl Divider {
    pub fn new(axis: Axis) -> Self {
        Self {
            axis,
            thickness: None,
            indent: None,
            corner_radius: None,
            color: None,
        }
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = Some(thickness);
        self
    }

    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = Some(indent);
        self
    }

    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = Some(corner_radius.into());
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    fn get_thickness(&self, theme: &DividerTheme) -> f32 {
        self.thickness.unwrap_or(theme.thickness)
    }

    fn get_indent(&self, theme: &DividerTheme) -> f32 {
        self.indent.unwrap_or(theme.indent)
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
impl<C, T> View<C, T> for Divider
where
    C: Builder + Providable,
{
    type Element = WidgetId<widgets::Divider>;
    type State = ();

    fn build(&mut self, cx: &mut C, _data: &mut T) -> (Self::Element, Self::State) {
        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<DividerTheme>();

        let mut widget = widgets::Divider::new(cx);

        let thickness = self.get_thickness(&theme);
        let indent = self.get_indent(&theme);
        let corner_radius = self.get_corner_radius(&theme);
        let color = self.get_color(&theme, &palette);

        widgets::Divider::set_axis(&mut widget, self.axis);
        widgets::Divider::set_thickness(&mut widget, thickness);
        widgets::Divider::set_indent(&mut widget, indent);
        widgets::Divider::set_corner_radius(&mut widget, corner_radius);
        widgets::Divider::set_color(&mut widget, color);

        (widget.id(), ())
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        cx: &mut C,
        _data: &mut T,
        old: &mut Self,
    ) {
        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<DividerTheme>();

        let Some(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.axis != old.axis {
            widgets::Divider::set_axis(&mut widget, self.axis);
        }

        if self.thickness != old.thickness {
            let thickness = self.get_thickness(&theme);
            widgets::Divider::set_thickness(&mut widget, thickness);
        }

        if self.indent != old.indent {
            let indent = self.get_indent(&theme);
            widgets::Divider::set_indent(&mut widget, indent);
        }

        if self.corner_radius != old.corner_radius {
            let corner_radius = self.get_corner_radius(&theme);
            widgets::Divider::set_corner_radius(&mut widget, corner_radius);
        }

        if self.color != old.color {
            let color = self.get_color(&theme, &palette);
            widgets::Divider::set_color(&mut widget, color);
        }
    }

    fn teardown(&mut self, element: Self::Element, _state: Self::State, cx: &mut C, _data: &mut T) {
        cx.remove_widget(element);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
        _event: &mut Event,
    ) -> Action {
        Action::new()
    }
}
