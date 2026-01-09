use std::ops::Range;

use crate::{
    Affine, AnyWidget, AnyWidgetId, Clip, CursorIcon, ImeSignal, Painter, Paragraph, Point, Rect,
    Signal, Size, Space, Svg, TextLayoutLine, Widget, WidgetId, WidgetMut, WidgetRef, Window,
    passes,
    widget::{WidgetHierarchy, WidgetState},
    world::{Widgets, WorldState},
};

pub(crate) enum FocusUpdate {
    None,
    Take,
    Target(WidgetId),
    Next,
    Previous,
}

pub struct RefCx<'a> {
    pub(crate) widgets:   &'a Widgets,
    pub(crate) world:     &'a WorldState,
    pub(crate) state:     &'a WidgetState,
    pub(crate) hierarchy: &'a WidgetHierarchy,
}

pub struct MutCx<'a> {
    pub(crate) widgets:   &'a Widgets,
    pub(crate) world:     &'a mut WorldState,
    pub(crate) state:     &'a mut WidgetState,
    pub(crate) hierarchy: &'a WidgetHierarchy,
}

pub struct EventCx<'a> {
    pub(crate) widgets:   &'a Widgets,
    pub(crate) world:     &'a mut WorldState,
    pub(crate) state:     &'a mut WidgetState,
    pub(crate) hierarchy: &'a WidgetHierarchy,
    pub(crate) focus:     &'a mut FocusUpdate,
}

pub struct UpdateCx<'a> {
    pub(crate) widgets:   &'a Widgets,
    pub(crate) world:     &'a mut WorldState,
    pub(crate) state:     &'a mut WidgetState,
    pub(crate) hierarchy: &'a WidgetHierarchy,
}

pub struct LayoutCx<'a> {
    pub(crate) widgets:   &'a Widgets,
    pub(crate) world:     &'a mut WorldState,
    pub(crate) state:     &'a mut WidgetState,
    pub(crate) hierarchy: &'a WidgetHierarchy,
    pub(crate) painter:   &'a mut dyn Painter,
    pub(crate) scale:     f32,
}

pub struct ComposeCx<'a> {
    pub(crate) widgets:   &'a Widgets,
    pub(crate) world:     &'a mut WorldState,
    pub(crate) state:     &'a mut WidgetState,
    pub(crate) hierarchy: &'a WidgetHierarchy,
    pub(crate) scale:     f32,
}

pub struct DrawCx<'a> {
    pub(crate) widgets:   &'a Widgets,
    pub(crate) world:     &'a mut WorldState,
    pub(crate) state:     &'a mut WidgetState,
    pub(crate) hierarchy: &'a WidgetHierarchy,
}

impl MutCx<'_> {
    pub fn get_widget_mut<U>(&mut self, id: WidgetId<U>) -> Option<WidgetMut<'_, U>>
    where
        U: ?Sized + AnyWidget,
    {
        self.widgets.get_mut(self.world, id)
    }

    pub fn get_child_mut(&mut self, index: usize) -> Option<WidgetMut<'_, dyn Widget>> {
        self.widgets.get_mut(
            self.world,
            *self.hierarchy.children.get(index)?,
        )
    }

    pub fn get_parent_mut(&mut self) -> Option<WidgetMut<'_>> {
        self.widgets.get_mut(self.world, self.hierarchy.parent?)
    }

    pub fn for_each_child_mut(&mut self, mut f: impl FnMut(&mut WidgetMut)) {
        for &child in &self.hierarchy.children {
            if let Some(mut child) = self.widgets.get_mut(self.world, child) {
                f(&mut child);
            }
        }
    }

    pub fn request_compose(&mut self) {
        self.hierarchy.request_compose();
        passes::hierarchy::propagate_down(self.widgets, self.id());

        if let Some(window) = self.hierarchy.window {
            self.world.request_redraw(window);
        }
    }

    pub fn request_animate(&mut self) {
        self.hierarchy.request_animate();
        passes::hierarchy::propagate_down(self.widgets, self.id());

        if let Some(window) = self.hierarchy.window {
            self.world.request_animate(window);
        }
    }

    pub fn request_layout(&mut self) {
        self.hierarchy.request_layout();
        passes::hierarchy::propagate_down(self.widgets, self.id());

        if let Some(window) = self.hierarchy.window {
            self.world.request_redraw(window);
        }
    }

    pub fn request_draw(&mut self) {
        self.hierarchy.request_draw();
        passes::hierarchy::propagate_down(self.widgets, self.id());

        if let Some(window) = self.hierarchy.window {
            self.world.request_redraw(window);
        }
    }

    pub fn set_pixel_perfect(&mut self, pixel_perfect: bool) {
        if self.is_pixel_perfect() != pixel_perfect {
            self.state.is_pixel_perfect = pixel_perfect;
            self.request_layout();
        }
    }

    pub fn set_cursor(&mut self, cursor: CursorIcon) {
        self.state.cursor = cursor;
    }

    pub(crate) fn as_update_cx(&mut self) -> UpdateCx<'_> {
        UpdateCx {
            widgets:   self.widgets,
            world:     self.world,
            state:     self.state,
            hierarchy: self.hierarchy,
        }
    }

    pub(crate) fn as_layout_cx<'a>(
        &'a mut self,
        painter: &'a mut dyn Painter,
        scale: f32,
    ) -> LayoutCx<'a> {
        LayoutCx {
            widgets: self.widgets,
            world: self.world,
            state: self.state,
            hierarchy: self.hierarchy,
            painter,
            scale,
        }
    }

    pub(crate) fn as_compose_cx(&mut self, scale: f32) -> ComposeCx<'_> {
        ComposeCx {
            widgets: self.widgets,
            world: self.world,
            state: self.state,
            hierarchy: self.hierarchy,
            scale,
        }
    }

    pub(crate) fn as_draw_cx(&mut self) -> DrawCx<'_> {
        DrawCx {
            widgets:   self.widgets,
            world:     self.world,
            state:     self.state,
            hierarchy: self.hierarchy,
        }
    }

    pub(crate) fn enter_span(&self) -> Option<tracing::span::EnteredSpan> {
        (self.world.should_trace).then(|| self.state.tracing_span.clone().entered())
    }
}

impl EventCx<'_> {
    pub fn request_focus(&mut self) {
        *self.focus = FocusUpdate::Target(self.id());
    }

    pub fn request_unfocus(&mut self) {
        *self.focus = FocusUpdate::Take;
    }

    pub fn request_focus_next(&mut self) {
        *self.focus = FocusUpdate::Next;
    }

    pub fn request_focus_previous(&mut self) {
        *self.focus = FocusUpdate::Previous;
    }

    pub fn set_clipboard(&mut self, contents: String) {
        self.world.emit_signal(Signal::ClipboardSet(contents));
    }
}

impl LayoutCx<'_> {
    pub fn measure_text(&mut self, paragraph: &Paragraph, max_width: f32) -> Size {
        self.painter.measure_text(paragraph, max_width)
    }

    pub fn layout_text(&mut self, paragraph: &Paragraph, max_width: f32) -> Vec<TextLayoutLine> {
        self.painter.layout_text(paragraph, max_width)
    }

    pub fn measure_svg(&mut self, svg: &Svg) -> Size {
        self.painter.measure_svg(svg)
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn set_child_stashed(&mut self, index: usize, is_stashed: bool) {
        if let Some(child) = self.hierarchy.children.get(index)
            && let Some(mut child) = self.widgets.get_mut(self.world, *child)
        {
            passes::hierarchy::set_stashed(&mut child, is_stashed);
        } else {
            tracing::error!("`LayoutCx::set_child_stashed` called on invalid child");
        }
    }

    pub fn layout_child(&mut self, index: usize, space: Space) -> Size {
        if let Some(child) = self.hierarchy.children.get(index)
            && let Some(mut child) = self.widgets.get_mut(self.world, *child)
        {
            passes::layout::layout_widget(
                &mut child,
                space,
                self.painter,
                self.scale,
            )
        } else {
            tracing::error!("`LayoutCx::layout_child` called on invalid child");

            Size::ZERO
        }
    }

    pub fn place_child(&mut self, index: usize, transform: impl Into<Affine>) {
        if let Some(child) = self.hierarchy.children.get(index)
            && let Some(mut child) = self.widgets.get_mut(self.world, *child)
        {
            passes::layout::place_widget(&mut child, transform.into(), self.scale);
        } else {
            tracing::error!("`LayoutCx::place_child` called on invalid child");
        }
    }

    pub fn set_clip(&mut self, rect: impl Into<Option<Clip>>) {
        self.state.clip = rect.into();
    }
}

impl ComposeCx<'_> {
    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn place_child(&mut self, index: usize, transform: impl Into<Affine>) {
        if let Some(child) = self.hierarchy.children.get(index)
            && let Some(mut child) = self.widgets.get_mut(self.world, *child)
        {
            passes::compose::place_widget(&mut child, transform.into(), self.scale);
        } else {
            tracing::error!("`ComposeCx::place_child` called on invalid child");
        }
    }
}

macro_rules! impl_contexts {
    ($cx:ident <'_ $(, $t:ident)?> { $($tt:tt)* }) => {
        impl $(<$t: ?Sized + AnyWidget>)? $cx<'_ $(, $t)?> {
            $($tt)*
        }
    };
    (
        $cx:ident <'_ $(, $t:ident)?>,
        $($cxs:ident <'_ $(, $u:ident)?>),* $(,)*
        { $($tt:tt)* }
    ) => {
        impl_contexts!($cx <'_ $(, $t)?> { $($tt)* });
        impl_contexts!($($cxs <'_ $(, $u)?>),* { $($tt)* });
    }
}

impl_contexts! {
    RefCx<'_>,
    MutCx<'_>,
    EventCx<'_>,
    UpdateCx<'_>,
    LayoutCx<'_>,
    ComposeCx<'_>,
    DrawCx<'_> {
        pub fn id(&self) -> WidgetId {
            self.state.id
        }
    }
}

impl_contexts! {
    RefCx<'_>,
    MutCx<'_>,
    EventCx<'_>,
    UpdateCx<'_>,
    LayoutCx<'_>,
    ComposeCx<'_>,
    DrawCx<'_> {
        pub fn get_widget<U>(&self, id: WidgetId<U>) -> Option<WidgetRef<'_, U>>
        where
            U: ?Sized + AnyWidget,
        {
            self.widgets.get(self.world, id)
        }

        pub fn children(&self) -> &[WidgetId] {
            &self.hierarchy.children
        }

        pub fn iter_children(&self) -> impl DoubleEndedIterator<Item = WidgetRef<'_>> {
            self.hierarchy.children.iter().filter_map(|child| {
                self.widgets.get(self.world, *child)
            })
        }

        pub fn is_child(&self, id: impl AnyWidgetId) -> bool {
            self.get_widget(id.upcast()).is_some_and(|w| w.cx.hierarchy.parent == Some(self.id().upcast()))
        }

        pub fn get_child(&self, index: usize) -> Option<WidgetRef<'_>> {
            self.widgets.get(self.world, self.hierarchy.children[index])
        }

        pub fn parent(&self) -> Option<WidgetId> {
            self.hierarchy.parent
        }

        pub fn get_parent(&self) -> Option<WidgetRef<'_>> {
            self.widgets.get(self.world, self.hierarchy.parent?)
        }

        pub fn get_window(&self) -> Option<&Window> {
            self.world.get_window(self.hierarchy.window?)
        }

        pub fn is_window_focused(&self) -> bool {
            self.get_window().is_some_and(|window| window.is_focused())
        }

        pub fn is_pixel_perfect(&self) -> bool {
            self.state.is_pixel_perfect
        }

        pub fn is_hovered(&self) -> bool {
            self.hierarchy.is_hovered()
        }

        pub fn is_active(&self) -> bool {
            self.hierarchy.is_active()
        }

        pub fn is_focused(&self) -> bool {
            self.hierarchy.is_focused()
        }

        pub fn is_stashed(&self) -> bool {
            self.hierarchy.is_stashed()
        }

        pub fn is_disabled(&self) -> bool {
            self.hierarchy.is_disabled()
        }

        pub fn has_hovered(&self) -> bool {
            self.hierarchy.has_hovered()
        }

        pub fn has_active(&self) -> bool {
            self.hierarchy.has_active()
        }

        pub fn has_focused(&self) -> bool {
            self.hierarchy.has_focused()
        }

        pub fn clip(&self) -> Option<&Clip> {
            self.state.clip.as_ref()
        }

        pub fn cursor(&self) -> CursorIcon{
            self.state.cursor
        }
    }
}

impl_contexts! {
    EventCx<'_>,
    UpdateCx<'_> {
        pub fn request_compose(&mut self) {
            self.hierarchy.request_compose();

            if let Some(window) = self.hierarchy.window {
                self.world.request_redraw(window);
            }
        }

        pub fn request_animate(&mut self) {
            self.hierarchy.request_animate();

            if let Some(window) = self.hierarchy.window {
                self.world.request_animate(window);
            }
        }

        pub fn request_layout(&mut self) {
            self.hierarchy.request_layout();

            if let Some(window) = self.hierarchy.window {
                self.world.request_redraw(window);
            }
        }


        pub fn request_draw(&mut self) {
            self.hierarchy.request_draw();

            if let Some(window) = self.hierarchy.window {
                self.world.request_redraw(window);
            }
        }

        pub fn set_pixel_perfect(&mut self, pixel_perfect: bool) {
            if self.is_pixel_perfect() != pixel_perfect {
                self.state.is_pixel_perfect = pixel_perfect;
                self.request_layout();
            }
        }

        pub fn set_cursor(&mut self, cursor: CursorIcon) {
            self.state.cursor = cursor;
        }
    }
}

impl_contexts! {
    MutCx<'_>,
    EventCx<'_>,
    UpdateCx<'_> {
        pub fn restart_ime(&mut self) {
            if !self.is_focused() || !self.hierarchy.accepts_text() {
                tracing::warn!("`restart_ime` can only be called on a focused view that accepts text");
                return;
            }

            self.world.emit_signal(Signal::Ime(ImeSignal::Start));
        }

        pub fn set_ime_text(&mut self, text: String) {
            if !self.is_focused() || !self.hierarchy.accepts_text() {
                tracing::warn!("`set_ime_text` can only be called on a focused view that accepts text");
                return;
            }

            self.world.emit_signal(Signal::Ime(ImeSignal::Text(text)));
        }

        pub fn set_ime_selection(
            &mut self,
            selection: Range<usize>,
            composing: Option<Range<usize>>,
        ) {
            if !self.is_focused() || !self.hierarchy.accepts_text() {
                tracing::warn!("`set_ime_selection` can only be called on a focused view that accepts text");
                return;
            }

            self.world.emit_signal(Signal::Ime(ImeSignal::Selection {
                selection,
                composing,
            }));
        }
    }
}

impl_contexts! {
    RefCx<'_>,
    MutCx<'_>,
    EventCx<'_>,
    UpdateCx<'_>,
    ComposeCx<'_>,
    DrawCx<'_> {
        pub fn transform(&self) -> Affine {
            self.state.transform
        }

        pub fn global_transform(&self) -> Affine {
            self.state.global_transform
        }

        pub fn size(&self) -> Size {
            self.state.size
        }

        pub fn width(&self) -> f32 {
            self.size().width
        }

        pub fn height(&self) -> f32 {
            self.size().height
        }

        pub fn rect(&self) -> Rect {
            Rect::min_size(Point::all(0.0), self.size())
        }
    }
}
