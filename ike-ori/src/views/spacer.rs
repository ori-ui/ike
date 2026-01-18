use ike_core::{Builder, Size, WidgetId, widgets};
use ori::{Action, Event, View, ViewMarker};

use crate::Context;

pub fn spacer(size: impl Into<Size>) -> Spacer {
    Spacer::new(size)
}

pub fn hspacer(size: f32) -> Spacer {
    Spacer::new(Size::new(size, 0.0))
}

pub fn vspacer(size: f32) -> Spacer {
    Spacer::new(Size::new(0.0, size))
}

pub struct Spacer {
    size: Size,
}

impl Spacer {
    pub fn new(size: impl Into<Size>) -> Self {
        Self { size: size.into() }
    }
}

impl ViewMarker for Spacer {}
impl<T> View<Context, T> for Spacer {
    type Element = WidgetId<widgets::Spacer>;
    type State = ();

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let mut widget = widgets::Spacer::new(cx);

        widgets::Spacer::set_size(&mut widget, self.size);

        (widget.id(), ())
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        let Ok(mut widget) = cx.get_widget_mut(*element) else {
            return;
        };

        if self.size != old.size {
            widgets::Spacer::set_size(&mut widget, self.size);
        }
    }

    fn teardown(&mut self, element: Self::Element, _state: Self::State, cx: &mut Context) {
        cx.remove_widget(element);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _event: &mut Event,
    ) -> Action {
        Action::new()
    }
}
