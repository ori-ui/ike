use ike_core::{
    Builder, Color, WidgetId,
    widgets::{self, Fit, Picturable},
};
use ori::{Action, Event, View, ViewMarker};

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
impl<C, T> View<C, T> for Picture
where
    C: Builder,
{
    type Element = WidgetId<widgets::Picture>;
    type State = ();

    fn build(&mut self, cx: &mut C, _data: &mut T) -> (Self::Element, Self::State) {
        let mut widget = widgets::Picture::new(cx, self.contents.clone());
        widgets::Picture::set_fit(&mut widget, self.fit);
        widgets::Picture::set_color(&mut widget, self.color);

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
        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.contents != old.contents {
            widgets::Picture::set_contents(&mut widget, self.contents.clone());
        }

        if self.fit != old.fit {
            widgets::Picture::set_fit(&mut widget, self.fit);
        }

        if self.color != old.color {
            widgets::Picture::set_color(&mut widget, self.color);
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
