use std::{marker::PhantomData, mem, time::Duration};

use crate::{
    Affine, AnyWidget, AnyWidgetId, BorderWidth, Canvas, ChildUpdate, Clip, CornerRadius, MutCx,
    Offset, Paint, Painter, Paragraph, Recording, Rect, RefCx, Size, Space, Svg, Update, Widget,
    WidgetId, WidgetRef,
};

pub struct WidgetMut<'a, T = dyn Widget>
where
    T: ?Sized + AnyWidget,
{
    pub widget: &'a mut T,
    pub cx:     MutCx<'a>,
}

impl<T> Drop for WidgetMut<'_, T>
where
    T: ?Sized + AnyWidget,
{
    fn drop(&mut self) {
        let id = self.cx.id();
        let entry = &self.cx.arena.entries[id.index as usize];
        entry.borrowed.set(false);
    }
}

impl<'a, T> WidgetMut<'a, T>
where
    T: ?Sized + AnyWidget,
{
    pub fn id(&self) -> WidgetId<T> {
        let id = self.cx.state.id;

        WidgetId {
            index:      id.index,
            generation: id.generation,
            marker:     PhantomData,
        }
    }

    pub fn upcast(&mut self) -> WidgetMut<'_, dyn Widget> {
        WidgetMut {
            widget: T::upcast_mut(self.widget),

            cx: MutCx {
                root:  self.cx.root,
                arena: self.cx.arena,
                state: self.cx.state,
            },
        }
    }

    pub fn to_ref(&self) -> WidgetRef<'_, T> {
        WidgetRef {
            widget: self.widget,

            cx: RefCx {
                root:  self.cx.root,
                arena: self.cx.arena,
                state: self.cx.state,
            },
        }
    }

    pub fn set_stashed(&mut self, is_stashed: bool) {
        self.set_stashed_without_propagate(is_stashed);
        self.cx.propagate_state();
    }

    pub fn set_disabled(&mut self, is_disabled: bool) {
        self.set_disabled_without_propagate(is_disabled);
        self.cx.propagate_state();
    }

    /// Add a child to widget.
    pub fn add_child(&mut self, child: impl AnyWidgetId) {
        let child = child.upcast();
        self.cx.arena.debug_validate_id(child);

        let window = self.cx.state.window;
        if let Some(mut child) = self.cx.get_mut(child) {
            child.cx.set_window_recursive(window);
        }

        let id = self.id();
        if let Some(child_state) = self.cx.arena.get_state_mut(child.index) {
            child_state.parent = Some(id.upcast());
        }

        let index = self.cx.state.children.len();
        self.cx.state.children.push(child.upcast());
        self.update_without_propagate(Update::Children(ChildUpdate::Added(
            index,
        )));
        self.cx.request_layout();
    }

    /// Swap two children of a widget.
    pub fn swap_children(&mut self, index_a: usize, index_b: usize) {
        self.cx.state.children.swap(index_a, index_b);
        self.update_without_propagate(Update::Children(ChildUpdate::Swapped(
            index_a, index_b,
        )));
        self.cx.request_layout();
    }

    /// Replace the child of a widget with another returning the previous child.
    #[must_use = "the previous child is not destroyed, see `MutCx::remove`"]
    pub fn replace_child(&mut self, index: usize, child: impl AnyWidgetId) -> WidgetId {
        let child = child.upcast();
        self.cx.arena.debug_validate_id(child);

        let window = self.cx.state.window;
        if let Some(mut child) = self.cx.get_mut(child) {
            child.cx.set_window_recursive(window);
        }

        let id = self.id();
        if let Some(child_state) = self.cx.arena.get_state_mut(child.index) {
            assert!(child_state.parent.is_none());
            child_state.parent = Some(id.upcast());
        }

        let prev_child = mem::replace(
            &mut self.cx.state.children[index],
            child,
        );

        let update = Update::Children(ChildUpdate::Replaced(index));
        self.update_without_propagate(update);
        self.cx.request_layout();

        let window = self.cx.state.window;
        if let Some(mut prev_child) = self.cx.get_mut(prev_child) {
            prev_child.cx.set_window_recursive(window);
        }

        prev_child
    }

    pub fn remove_child(&mut self, index: usize) {
        let id = self.cx.state.children.remove(index);

        let update = Update::Children(ChildUpdate::Removed(index));
        self.update_without_propagate(update);

        if !self.cx.arena.free_recursive(id) {
            tracing::error!(
                "`remove_child` was called, while a descendant of the removed widget was borrowed"
            );
        }

        self.cx.request_layout();
    }
}

impl<'a, T> WidgetMut<'a, T>
where
    T: ?Sized + AnyWidget,
{
    pub(crate) fn set_stashed_without_propagate(&mut self, is_stashed: bool) {
        if self.cx.is_stashed() == is_stashed {
            return;
        }

        if !is_stashed && self.cx.get_parent().is_some_and(|p| p.cx.is_stashed()) {
            return;
        }

        self.set_stashed_recursive(is_stashed);
    }

    fn set_stashed_recursive(&mut self, is_stashed: bool) {
        self.cx.for_each_child(|child| {
            child.set_stashed_recursive(is_stashed);
        });

        self.cx.state.is_stashed = is_stashed;
        self.cx.state.needs_layout = true;

        if !is_stashed {
            self.cx.state.is_focused = false;
            self.cx.state.is_hovered = false;
            self.cx.state.is_active = false;

            self.update_without_propagate(Update::Focused(false));
            self.update_without_propagate(Update::Hovered(false));
            self.update_without_propagate(Update::Active(false));
        }

        self.update_without_propagate(Update::Stashed(is_stashed));
        self.cx.update_state();
    }

    pub(crate) fn set_disabled_without_propagate(&mut self, is_disabled: bool) {
        if self.cx.is_disabled() == is_disabled {
            return;
        }

        if !is_disabled && self.cx.get_parent().is_some_and(|p| p.cx.is_disabled()) {
            return;
        }

        self.set_disabled_recursive(is_disabled);
    }

    fn set_disabled_recursive(&mut self, is_disabled: bool) {
        self.cx.for_each_child(|child| {
            child.set_disabled_recursive(is_disabled);
        });

        self.cx.state.is_disabled = is_disabled;

        if !is_disabled {
            self.cx.state.is_focused = false;
            self.cx.state.is_hovered = false;
            self.cx.state.is_active = false;

            self.update_without_propagate(Update::Focused(false));
            self.update_without_propagate(Update::Hovered(false));
            self.update_without_propagate(Update::Active(false));
        }

        self.update_without_propagate(Update::Disabled(is_disabled));
        self.cx.update_state();
    }

    pub(crate) fn set_hovered(&mut self, is_hovered: bool) {
        self.cx.state.is_hovered = is_hovered;
        self.update_without_propagate(Update::Hovered(is_hovered));
        self.cx.propagate_state();
    }

    pub(crate) fn set_active(&mut self, is_active: bool) {
        self.cx.state.is_active = is_active;
        self.update_without_propagate(Update::Active(is_active));
        self.cx.propagate_state();
    }

    pub(crate) fn set_focused(&mut self, is_focused: bool) {
        self.cx.state.is_focused = is_focused;
        self.update_without_propagate(Update::Focused(is_focused));
        self.cx.propagate_state();
    }

    pub(crate) fn update_without_propagate(&mut self, update: Update) {
        self.widget.update(&mut self.cx.as_update_cx(), update);
    }

    pub(crate) fn update_recursive(&mut self, update: Update) {
        let _span = self.cx.enter_span();

        self.cx.for_each_child(|child| {
            child.update_recursive(update.clone());
        });

        self.cx.update_state();

        if let Update::WindowScaleChanged(..) = update
            && self.cx.is_pixel_perfect()
        {
            self.cx.state.needs_layout = true;
            self.cx.state.recording = None;
        }

        self.update_without_propagate(update);
    }

    pub(crate) fn animate_recursive(&mut self, dt: Duration) {
        if self.cx.is_stashed() || !self.cx.state.needs_animate {
            return;
        }

        let _span = self.cx.enter_span();

        self.cx.for_each_child(|child| child.animate_recursive(dt));

        self.cx.state.needs_animate = false;
        self.cx.update_state();

        self.widget.animate(&mut self.cx.as_update_cx(), dt);
    }

    pub(crate) fn layout_recursive(
        &mut self,
        scale: f32,
        painter: &mut dyn Painter,
        space: Space,
    ) -> Size {
        if self.cx.state.is_stashed {
            self.cx.state.transform = Affine::IDENTITY;
            self.cx.state.size = space.min;
            return space.min;
        }

        if self.cx.state.previous_space == Some(space) && !self.cx.state.needs_layout {
            return self.cx.state.size;
        }

        let _span = self.cx.enter_span();
        self.cx.state.needs_layout = false;

        let mut size = self.widget.layout(
            &mut self.cx.as_layout_cx(scale),
            painter,
            space,
        );

        if self.cx.is_pixel_perfect() {
            size = size.ceil_to_scale(scale);
        }

        if !size.is_finite() {
            tracing::warn!(size = ?size, "size is not finite");
        }

        self.cx.update_state();
        self.cx.state.size = size;
        self.cx.state.previous_space = Some(space);

        size
    }

    pub(crate) fn compose_recursive(&mut self, scale: f32, transform: Affine) {
        if self.cx.is_stashed() {
            return;
        }

        let _span = self.cx.enter_span();

        let global_transform = transform * self.cx.transform();

        if self.cx.global_transform() == global_transform && !self.cx.state.needs_compose {
            return;
        }

        self.cx.state.global_transform = global_transform;
        self.cx.state.needs_compose = false;

        self.widget.compose(&mut self.cx.as_compose_cx(scale));

        self.cx.for_each_child(|child| {
            child.compose_recursive(scale, global_transform);
        });

        self.cx.update_state();
    }

    pub(crate) fn draw_recursive(&mut self, canvas: &mut dyn Canvas) {
        if self.cx.is_stashed() {
            return;
        }

        self.cx.state.stable_draws += 1;

        if self.cx.state.needs_draw {
            self.cx.state.stable_draws = 0;
        }

        if let Some(ref recording) = self.cx.state.recording
            && !self.cx.state.needs_draw
        {
            canvas.transform(self.cx.transform(), &mut |canvas| {
                canvas.draw_recording(recording);
            });

            return;
        }

        let _span = self.cx.enter_span();

        if self.cx.state.is_recording_draw {
            let recording = canvas.record(self.cx.size(), &mut |canvas| {
                self.draw_update_recording_heuristic(canvas, |this, canvas| {
                    this.draw_recursive_impl(canvas);
                });
            });

            canvas.transform(self.cx.transform(), &mut |canvas| {
                canvas.draw_recording(&recording);
            });

            self.cx.state.recording = Some(recording);
        } else {
            self.draw_update_recording_heuristic(canvas, |this, canvas| {
                this.draw_recursive_transformed(canvas);
            })
        }
    }

    fn draw_update_recording_heuristic(
        &mut self,
        canvas: &mut dyn Canvas,
        f: impl FnOnce(&mut Self, &mut dyn Canvas),
    ) {
        if self.cx.state.stable_draws < 10 {
            self.cx.state.is_recording_draw = false;
            self.cx.state.recording = None;
            f(self, canvas);
            return;
        }

        let mut tracked = TrackedCanvas::new(canvas);
        f(self, &mut tracked);

        let scale = self.cx.get_window().map_or(1.0, |w| w.scale());
        let mut record_cost = self.cx.size().area() * scale * scale / 400.0;

        if self.cx.state.is_recording_draw {
            record_cost *= 0.8;
        } else {
            record_cost *= 1.2;
        }

        let draw_cost = tracked.cost_heuristic;
        let should_record = record_cost < draw_cost;

        if self.cx.state.is_recording_draw != should_record {
            tracing::trace!(
                widget = ?self.cx.state.id,
                record = ?should_record,
                "draw recording changed",
            );

            self.cx.state.is_recording_draw = should_record;
            self.cx.state.recording = None;
        }
    }

    fn draw_recursive_transformed(&mut self, canvas: &mut dyn Canvas) {
        canvas.transform(self.cx.transform(), &mut |canvas| {
            if let Some(ref clip) = self.cx.state.clip {
                canvas.clip(&clip.clone(), &mut |canvas| {
                    self.draw_recursive_impl(canvas);
                });
            } else {
                self.draw_recursive_impl(canvas);
            }
        });
    }

    fn draw_recursive_impl(&mut self, canvas: &mut dyn Canvas) {
        let mut cx = self.cx.as_draw_cx();
        self.widget.draw(&mut cx, canvas);

        self.cx.for_each_child(|child| {
            child.draw_recursive(canvas);
        });
    }

    pub(crate) fn draw_over_recursive(&mut self, canvas: &mut dyn Canvas) {
        self.cx.state.needs_draw = false;

        if self.cx.is_stashed() {
            return;
        }

        let _span = self.cx.enter_span();

        canvas.transform(self.cx.transform(), &mut |canvas| {
            if let Some(ref clip) = self.cx.state.clip {
                canvas.clip(&clip.clone(), &mut |canvas| {
                    let mut cx = self.cx.as_draw_cx();
                    self.widget.draw_over(&mut cx, canvas);

                    self.cx.for_each_child(|child| {
                        child.draw_over_recursive(canvas);
                    });
                });
            } else {
                let mut cx = self.cx.as_draw_cx();
                self.widget.draw_over(&mut cx, canvas);

                self.cx.for_each_child(|child| {
                    child.draw_over_recursive(canvas);
                });
            }
        });
    }
}

/// [`Canvas`] tracking a heuristic of the cost drawing.
struct TrackedCanvas<'a> {
    canvas:         &'a mut dyn Canvas,
    cost_heuristic: f32,
}

impl<'a> TrackedCanvas<'a> {
    const TRANSFORM_COST: f32 = 0.0;
    const LAYER_COST: f32 = 16.0;
    const CLIP_COST: f32 = 2.0;
    const FILL_COST: f32 = 1.0;
    const RECT_COST: f32 = 1.0;
    const BORDER_COST: f32 = 1.0;
    const TEXT_COST: f32 = 8.0;
    const SVG_COST: f32 = 8.0;
    const RECORDING_COST: f32 = 1.0;

    fn new(canvas: &'a mut dyn Canvas) -> Self {
        Self {
            canvas,
            cost_heuristic: 0.0,
        }
    }

    fn wrap(
        &mut self,
        f: &mut dyn FnMut(&mut dyn Canvas),
        g: impl FnOnce(&mut dyn Canvas, &mut dyn FnMut(&mut dyn Canvas)),
    ) {
        g(self.canvas, &mut |canvas| {
            let mut tracked = TrackedCanvas::new(canvas);
            f(&mut tracked);
            self.cost_heuristic += tracked.cost_heuristic;
        })
    }
}

impl Canvas for TrackedCanvas<'_> {
    fn painter(&mut self) -> &mut dyn Painter {
        self.canvas.painter()
    }

    fn transform(&mut self, affine: Affine, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.wrap(f, |canvas, f| {
            canvas.transform(affine, f)
        });

        self.cost_heuristic += Self::TRANSFORM_COST;
    }

    fn layer(&mut self, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.wrap(f, |canvas, f| canvas.layer(f));
        self.cost_heuristic += Self::LAYER_COST;
    }

    fn record(&mut self, size: Size, f: &mut dyn FnMut(&mut dyn Canvas)) -> Recording {
        // NOTE: we intentionally don't track the cost of the record call
        self.canvas.record(size, f)
    }

    fn clip(&mut self, clip: &Clip, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.wrap(f, |canvas, f| canvas.clip(clip, f));
        self.cost_heuristic += Self::CLIP_COST;
    }

    fn fill(&mut self, paint: &Paint) {
        self.canvas.fill(paint);
        self.cost_heuristic += Self::FILL_COST;
    }

    fn draw_rect(&mut self, rect: Rect, corners: CornerRadius, paint: &Paint) {
        self.canvas.draw_rect(rect, corners, paint);
        self.cost_heuristic += Self::RECT_COST;
    }

    fn draw_border(&mut self, rect: Rect, width: BorderWidth, radius: CornerRadius, paint: &Paint) {
        self.canvas.draw_border(rect, width, radius, paint);
        self.cost_heuristic += Self::BORDER_COST;
    }

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset) {
        self.canvas.draw_text(paragraph, max_width, offset);
        self.cost_heuristic += Self::TEXT_COST;
    }

    fn draw_svg(&mut self, svg: &Svg) {
        self.canvas.draw_svg(svg);
        self.cost_heuristic += Self::SVG_COST;
    }

    fn draw_recording(&mut self, recording: &Recording) {
        self.canvas.draw_recording(recording);
        self.cost_heuristic += Self::RECORDING_COST;
    }
}
