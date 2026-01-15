use std::{
    cell::{Ref, RefMut},
    ops::Range,
};

use crate::{
    Affine, AnyWidget, AnyWidgetId, Clip, CursorIcon, GetError, ImeSignal, Painter, Paragraph,
    Point, Rect, Settings, Signal, Size, Space, Svg, TextLayoutLine, WidgetId, WidgetMut,
    WidgetRef, Window, WindowId, World, passes,
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
    pub(crate) state:     Ref<'a, WidgetState>,
    pub(crate) hierarchy: &'a WidgetHierarchy,
}

pub struct MutCx<'a> {
    pub(crate) widgets:   &'a Widgets,
    pub(crate) world:     &'a mut WorldState,
    pub(crate) state:     RefMut<'a, WidgetState>,
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

    pub fn set_subpixel(&mut self, subpixel: bool) {
        if self.is_subpixel() != subpixel {
            self.state.is_subpixel = subpixel;
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
            state:     &mut self.state,
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
            state: &mut self.state,
            hierarchy: self.hierarchy,
            painter,
            scale,
        }
    }

    pub(crate) fn as_compose_cx(&mut self, scale: f32) -> ComposeCx<'_> {
        ComposeCx {
            widgets: self.widgets,
            world: self.world,
            state: &mut self.state,
            hierarchy: self.hierarchy,
            scale,
        }
    }

    pub(crate) fn as_draw_cx(&mut self) -> DrawCx<'_> {
        DrawCx {
            widgets:   self.widgets,
            world:     self.world,
            state:     &mut self.state,
            hierarchy: self.hierarchy,
        }
    }

    pub(crate) fn enter_span(&self) -> Option<tracing::span::EnteredSpan> {
        self.world
            .settings
            .debug
            .trace_widgets
            .then(|| self.state.tracing_span.clone().entered())
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
            && let Ok(mut child) = self.widgets.get_mut(self.world, *child)
        {
            passes::hierarchy::set_stashed(&mut child, is_stashed);
        } else {
            tracing::error!("`LayoutCx::set_child_stashed` called on invalid child");
        }
    }

    pub fn layout_child(&mut self, child: impl AnyWidgetId, space: Space) -> Size {
        let id = self.id();
        if let Ok(mut child) = self.widgets.get_mut(self.world, child.upcast()) {
            debug_assert!(child.cx.parent() == Some(id));

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

    pub fn layout_nth_child(&mut self, index: usize, space: Space) -> Size {
        if let Some(child) = self.hierarchy.children.get(index)
            && let Ok(mut child) = self.widgets.get_mut(self.world, *child)
        {
            passes::layout::layout_widget(
                &mut child,
                space,
                self.painter,
                self.scale,
            )
        } else {
            tracing::error!("`LayoutCx::layout_nth_child` called on invalid child");
            Size::ZERO
        }
    }

    pub fn place_child(&mut self, child: impl AnyWidgetId, transform: impl Into<Affine>) {
        passes::layout::place_child(self, child.upcast(), transform.into());
    }

    pub fn place_nth_child(&mut self, index: usize, transform: impl Into<Affine>) {
        if let Some(child) = self.hierarchy.children.get(index) {
            passes::layout::place_child(self, *child, transform.into());
        } else {
            tracing::error!("`LayoutCx::place_nth_child` called on invalid child");
        }
    }

    pub fn set_clip(&mut self, rect: impl Into<Option<Clip>>) {
        self.state.clip = rect.into();
    }
}

impl LayoutCx<'_> {
    pub fn request_compose(&mut self) {
        self.hierarchy.request_compose();
    }

    pub fn request_draw(&mut self) {
        self.hierarchy.request_draw();
    }
}

impl ComposeCx<'_> {
    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn place_child(&mut self, child: impl AnyWidgetId, transform: impl Into<Affine>) {
        passes::compose::place_child(self, child.upcast(), transform.into());
    }

    pub fn place_nth_child(&mut self, index: usize, transform: impl Into<Affine>) {
        if let Some(child) = self.hierarchy.children.get(index) {
            passes::compose::place_child(self, *child, transform.into());
        } else {
            tracing::error!("`ComposeCx::place_nth_child` called on invalid child");
        }
    }
}

impl ComposeCx<'_> {
    pub fn request_draw(&mut self) {
        self.hierarchy.request_draw();
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
    MutCx<'_>,
    EventCx<'_>,
    UpdateCx<'_> {
        pub fn get_parent_mut(&mut self) -> Result<WidgetMut<'_>, GetError> {
            let parent = self.hierarchy.parent.ok_or(GetError::InvalidParent)?;
            self.widgets.get_mut(self.world, parent)
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

        pub fn set_subpixel(&mut self, subpixel: bool) {
            if self.is_subpixel() != subpixel {
                self.state.is_subpixel = subpixel;
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
    UpdateCx<'_>,
    ComposeCx<'_> {
        pub fn get_widget_mut<U>(&mut self, id: WidgetId<U>) -> Result<WidgetMut<'_, U>, GetError>
        where
            U: ?Sized + AnyWidget,
        {
            self.widgets.get_mut(self.world, id)
        }

        /// Get a child
        pub fn get_child_mut<T>(&mut self, child: WidgetId<T>) -> Result<WidgetMut<'_, T>, GetError>
        where
            T: AnyWidget + ?Sized,
        {
            let id = self.id();
            let child = self.widgets.get_mut(self.world, child)?;
            debug_assert!(child.cx.parent() == Some(id));
            Ok(child)
        }

        pub fn get_nth_child_mut<T>(&mut self, index: usize) -> Result<WidgetMut<'_>, GetError> {
            let child = self.hierarchy.children.get(index).ok_or(GetError::InvalidChild)?;
            self.widgets.get_mut(self.world, *child)
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
    ComposeCx<'_> {
        pub fn get_parent(&self) -> Result<WidgetRef<'_>, GetError> {
            let parent = self.hierarchy.parent.ok_or(GetError::InvalidParent)?;
            self.widgets.get(self.world, parent)
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
        pub fn id(&self) -> WidgetId {
            self.state.id
        }

        /// Mutate the [`World`] at an unspecified time in the future.
        ///
        /// This is useful for when a widget needs to insert or remove other widgets, but use with
        /// care, as this can cause unwanted side effects.
        pub fn defer(&self, f: impl FnOnce(&mut World) + Send + 'static) {
            self.world.emit_signal(Signal::Mutate(Box::new(f)));
        }

        pub fn get_widget<U>(&self, id: WidgetId<U>) -> Result<WidgetRef<'_, U>, GetError>
        where
            U: ?Sized + AnyWidget,
        {
            self.widgets.get(self.world, id)
        }

        pub fn children(&self) -> &[WidgetId] {
            &self.hierarchy.children
        }

        pub fn iter_children(&self) -> impl DoubleEndedIterator<Item = Result<WidgetRef<'_>, GetError>> {
            self.hierarchy.children.iter().map(|child| {
                self.widgets.get(self.world, *child)
            })
        }

        pub fn is_child(&self, id: impl AnyWidgetId) -> bool {
            self.widgets.get_hierarchy(id.upcast())
                .is_some_and(|child| child.parent == Some(self.id()))
        }

        pub fn get_child<T>(&self, child: WidgetId<T>) -> Result<WidgetRef<'_, T>, GetError>
        where
            T: AnyWidget + ?Sized,
        {
            let id = self.id();
            let child = self.widgets.get(self.world, child)?;
            debug_assert!(child.cx.parent() == Some(id));
            Ok(child)
        }

        pub fn get_nth_child(&self, index: usize) -> Result<WidgetRef<'_>, GetError> {
            let child = self.hierarchy.children.get(index).ok_or(GetError::InvalidChild)?;
            self.widgets.get(self.world, *child)
        }

        pub fn parent(&self) -> Option<WidgetId> {
            self.hierarchy.parent
        }

        pub fn window(&self) -> Option<WindowId> {
            self.hierarchy.window
        }

        pub fn get_window(&self) -> Option<&Window> {
            self.world.get_window(self.window()?)
        }

        pub fn is_window_focused(&self) -> bool {
            self.get_window().is_some_and(|window| window.is_focused())
        }

        pub fn settings(&self) -> &Settings {
            &self.world.settings
        }

        pub fn is_subpixel(&self) -> bool {
            self.state.is_subpixel
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
