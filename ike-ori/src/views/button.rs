use ike_core::{
    BorderWidth, Builder, Color, CornerRadius, Padding, Transition, WidgetId, WidgetMut, widgets,
};
use ori::{Action, Event, Provider, Proxied, Proxy, View, ViewId, ViewMarker};

use crate::{Context, Palette};

pub fn button<T, V, A>(contents: V, on_click: impl FnMut(&mut T) -> A + 'static) -> Button<T, V>
where
    A: Into<Action>,
{
    Button::new(contents, on_click)
}

#[derive(Clone, Debug)]
pub struct ButtonTheme {
    pub padding:       Padding,
    pub border_width:  BorderWidth,
    pub corner_radius: CornerRadius,
    pub idle_color:    Option<Color>,
    pub hovered_color: Option<Color>,
    pub active_color:  Option<Color>,
    pub border_color:  Option<Color>,
    pub focus_color:   Option<Color>,
    pub transition:    Transition,
}

impl Default for ButtonTheme {
    fn default() -> Self {
        Self {
            padding:       Padding::all(8.0),
            border_width:  BorderWidth::all(1.0),
            corner_radius: CornerRadius::all(8.0),
            idle_color:    None,
            hovered_color: None,
            active_color:  None,
            border_color:  None,
            focus_color:   None,
            transition:    Transition::ease(0.05),
        }
    }
}

pub struct Button<T, V> {
    contents:   V,
    properties: Properties<T>,
}

impl<T, V> Button<T, V> {
    pub fn new<A>(contents: V, mut on_click: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        Button {
            contents,
            properties: Properties {
                on_click:      Box::new(move |data| on_click(data).into()),
                padding:       None,
                border_width:  None,
                corner_radius: None,
                idle_color:    None,
                hovered_color: None,
                active_color:  None,
                border_color:  None,
                focus_color:   None,
                transition:    None,
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

    pub fn transition(mut self, transition: Transition) -> Self {
        self.properties.transition = Some(transition);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.properties.idle_color = Some(color);
        self.properties.hovered_color = Some(color.lighten(0.04).desaturate(0.02));
        self.properties.active_color = Some(color.darken(0.04).desaturate(0.02));
        self
    }

    pub fn idle_color(mut self, color: Color) -> Self {
        self.properties.idle_color = Some(color);
        self
    }

    pub fn hovered_color(mut self, color: Color) -> Self {
        self.properties.hovered_color = Some(color);
        self
    }

    pub fn active_color(mut self, color: Color) -> Self {
        self.properties.active_color = Some(color);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.properties.border_color = Some(color);
        self
    }

    pub fn focus_color(mut self, color: Color) -> Self {
        self.properties.focus_color = Some(color);
        self
    }
}

enum ButtonEvent {
    Clicked,
}

pub struct Properties<T> {
    on_click: Box<dyn FnMut(&mut T) -> Action>,

    padding:       Option<Padding>,
    border_width:  Option<BorderWidth>,
    corner_radius: Option<CornerRadius>,
    idle_color:    Option<Color>,
    hovered_color: Option<Color>,
    active_color:  Option<Color>,
    border_color:  Option<Color>,
    focus_color:   Option<Color>,
    transition:    Option<Transition>,
}

impl<T> Properties<T> {
    fn get_padding(&self, theme: &ButtonTheme) -> Padding {
        self.padding.unwrap_or(theme.padding)
    }

    fn get_border_width(&self, theme: &ButtonTheme) -> BorderWidth {
        self.border_width.unwrap_or(theme.border_width)
    }

    fn get_corner_radius(&self, theme: &ButtonTheme) -> CornerRadius {
        self.corner_radius.unwrap_or(theme.corner_radius)
    }

    fn get_idle_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.idle_color
            .unwrap_or_else(|| theme.idle_color.unwrap_or_else(|| palette.surface(0)))
    }

    fn get_hovered_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.hovered_color.unwrap_or_else(|| {
            theme
                .hovered_color
                .unwrap_or_else(|| palette.surface(0).lighten(0.04).desaturate(0.02))
        })
    }

    fn get_active_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.active_color.unwrap_or_else(|| {
            theme
                .active_color
                .unwrap_or_else(|| palette.surface(0).darken(0.04).desaturate(0.02))
        })
    }

    fn get_border_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.border_color
            .unwrap_or_else(|| theme.border_color.unwrap_or(palette.outline))
    }

    fn get_focus_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.focus_color
            .unwrap_or_else(|| theme.focus_color.unwrap_or(palette.info))
    }

    fn get_transition(&self, theme: &ButtonTheme) -> Transition {
        self.transition.unwrap_or(theme.transition)
    }

    fn rebuild(
        &self,
        prev: &Self,
        widget: &mut WidgetMut<widgets::Button>,
        theme: &ButtonTheme,
        palette: &Palette,
    ) {
        if self.padding != prev.padding {
            let padding = self.get_padding(theme);
            widgets::Button::set_padding(widget, padding);
        }

        if self.border_width != prev.border_width {
            let border_width = self.get_border_width(theme);
            widgets::Button::set_border_width(widget, border_width);
        }

        if self.corner_radius != prev.corner_radius {
            let corner_radius = self.get_corner_radius(theme);
            widgets::Button::set_corner_radius(widget, corner_radius);
        }

        if self.idle_color != prev.idle_color {
            let idle_color = self.get_idle_color(theme, palette);
            widgets::Button::set_idle_color(widget, idle_color);
        }

        if self.hovered_color != prev.hovered_color {
            let hovered_color = self.get_hovered_color(theme, palette);
            widgets::Button::set_hovered_color(widget, hovered_color);
        }

        if self.active_color != prev.active_color {
            let active_color = self.get_active_color(theme, palette);
            widgets::Button::set_active_color(widget, active_color);
        }

        if self.border_color != prev.border_color {
            let border_color = self.get_border_color(theme, palette);
            widgets::Button::set_border_color(widget, border_color);
        }

        if self.focus_color != prev.focus_color {
            let focus_color = self.get_focus_color(theme, palette);
            widgets::Button::set_focus_color(widget, focus_color);
        }

        if self.transition != prev.transition {
            let transition = self.get_transition(theme);
            widgets::Button::set_transition(widget, transition);
        }
    }
}

impl<T, V> ViewMarker for Button<T, V> {}
impl<T, V> View<Context, T> for Button<T, V>
where
    V: crate::View<T>,
{
    type Element = WidgetId<widgets::Button>;
    type State = (
        ViewId,
        Properties<T>,
        V::Element,
        V::State,
    );

    fn build(self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<ButtonTheme>();
        let proxy = cx.proxy();
        let id = ViewId::next();

        let padding = self.properties.get_padding(&theme);
        let border_width = self.properties.get_border_width(&theme);
        let corner_radius = self.properties.get_corner_radius(&theme);
        let idle_color = self.properties.get_idle_color(&theme, &palette);
        let hovered_color = self.properties.get_hovered_color(&theme, &palette);
        let active_color = self.properties.get_active_color(&theme, &palette);
        let border_color = self.properties.get_border_color(&theme, &palette);
        let focus_color = self.properties.get_focus_color(&theme, &palette);
        let transition = self.properties.get_transition(&theme);

        let (contents, state) = self.contents.build(cx, data);
        let mut widget = widgets::Button::new(cx, contents);

        widgets::Button::set_padding(&mut widget, padding);
        widgets::Button::set_border_width(&mut widget, border_width);
        widgets::Button::set_corner_radius(&mut widget, corner_radius);
        widgets::Button::set_idle_color(&mut widget, idle_color);
        widgets::Button::set_hovered_color(&mut widget, hovered_color);
        widgets::Button::set_active_color(&mut widget, active_color);
        widgets::Button::set_border_color(&mut widget, border_color);
        widgets::Button::set_focus_color(&mut widget, focus_color);
        widgets::Button::set_transition(&mut widget, transition);

        widgets::Button::set_on_click(&mut widget, move || {
            proxy.event(Event::new(ButtonEvent::Clicked, id));
        });

        (
            widget.id(),
            (id, self.properties, contents, state),
        )
    }

    fn rebuild(
        self,
        element: &mut Self::Element,
        (_id, properties, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        (self.contents).rebuild(contents, state, cx, data);

        let palette = cx.get_or_default::<Palette>();
        let theme = cx.get_or_default::<ButtonTheme>();

        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        self.properties.rebuild(
            properties,
            &mut widget,
            &theme,
            &palette,
        );

        *properties = self.properties;
    }

    fn event(
        _element: &mut Self::Element,
        (id, properties, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let action = V::event(contents, state, cx, data, event);

        match event.take_targeted(*id) {
            Some(ButtonEvent::Clicked) => action | (properties.on_click)(data),
            None => action,
        }
    }

    fn teardown(
        element: Self::Element,
        (_id, _properties, contents, state): Self::State,
        cx: &mut Context,
    ) {
        V::teardown(contents, state, cx);
        cx.remove_widget(element);
    }
}
