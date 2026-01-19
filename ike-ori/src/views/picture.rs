use ike_core::{
    Builder, Color, WidgetId,
    widgets::{self, Fit, Picturable},
};
use ori::{Action, Event, View, ViewMarker};

use crate::Context;

pub fn picture(fit: Fit, content: impl Into<Picturable>) -> Picture {
    Picture::new(fit, content)
}

pub struct Picture {
    contents: Picturable,
    fit:      Fit,
    color:    Option<Color>,
}

impl Picture {
    pub fn new(fit: Fit, content: impl Into<Picturable>) -> Self {
        Self {
            contents: content.into(),
            fit,
            color: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl ViewMarker for Picture {}
impl<T> View<Context, T> for Picture {
    type Element = WidgetId<widgets::Picture>;
    type State = Self;

    fn build(self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let mut widget = widgets::Picture::new(cx, self.contents.clone());
        widgets::Picture::set_fit(&mut widget, self.fit);
        widgets::Picture::set_color(&mut widget, self.color);

        (widget.id(), self)
    }

    fn rebuild(
        self,
        element: &mut Self::Element,
        picture: &mut Self::State,
        cx: &mut Context,
        _data: &mut T,
    ) {
        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.contents != picture.contents {
            widgets::Picture::set_contents(&mut widget, self.contents.clone());
        }

        if self.fit != picture.fit {
            widgets::Picture::set_fit(&mut widget, self.fit);
        }

        if self.color != picture.color {
            widgets::Picture::set_color(&mut widget, self.color);
        }

        *picture = self;
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
