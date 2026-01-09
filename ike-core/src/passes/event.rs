use crate::{EventCx, Widget, WidgetId, WindowId, World, context::FocusUpdate, passes};

pub(crate) fn send_event<P>(
    world: &mut World,
    window: WindowId,
    target: WidgetId,
    bubble: P,
    on_event: impl Fn(&mut dyn Widget, &mut EventCx<'_>) -> P,
) -> P
where
    P: Eq + Copy,
{
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = bubble;

    while let Some(id) = current
        && let Some(widget) = world.widget_mut(id)
        && propagate == bubble
    {
        let mut cx = EventCx {
            widgets:   widget.cx.widgets,
            world:     widget.cx.world,
            state:     widget.cx.state,
            hierarchy: widget.cx.hierarchy,
            focus:     &mut focus,
        };

        propagate = on_event(widget.widget, &mut cx);
        current = widget.cx.parent();
    }

    passes::hierarchy::propagate_down(&world.widgets, target);
    passes::focus::update(world, window, focus);

    propagate
}
