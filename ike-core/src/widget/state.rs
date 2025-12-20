use crate::{Affine, Clip, CursorIcon, Size, Space, Widget, WidgetId, WindowId};

#[derive(Debug)]
pub struct WidgetState {
    pub(crate) id:               WidgetId,
    pub(crate) transform:        Affine,
    pub(crate) global_transform: Affine,
    pub(crate) size:             Size,
    pub(crate) parent:           Option<WidgetId>,
    pub(crate) children:         Vec<WidgetId>,
    pub(crate) previous_space:   Option<Space>,
    pub(crate) cursor:           CursorIcon,
    pub(crate) window:           Option<WindowId>,

    pub(crate) is_pixel_perfect: bool,
    pub(crate) stable_draws:     u32,
    pub(crate) clip:             Option<Clip>,

    pub(crate) is_hovered:  bool,
    pub(crate) has_hovered: bool,

    pub(crate) is_focused:  bool,
    pub(crate) has_focused: bool,

    pub(crate) is_active:  bool,
    pub(crate) has_active: bool,

    pub(crate) is_stashed:  bool,
    pub(crate) is_disabled: bool,

    pub(crate) needs_compose: bool,
    pub(crate) needs_animate: bool,
    pub(crate) needs_layout:  bool,
    pub(crate) needs_draw:    bool,

    pub(crate) accepts_pointer: bool,
    pub(crate) accepts_focus:   bool,
    pub(crate) accepts_text:    bool,

    pub(crate) tracing_span: tracing::Span,
    #[allow(dead_code, reason = "used for debug purposes")]
    pub(crate) short_name:   &'static str,
    #[allow(dead_code, reason = "used for debug purposes")]
    pub(crate) type_name:    &'static str,
}

impl WidgetState {
    pub(crate) fn new<T: Widget>(id: WidgetId) -> Self {
        #[allow(clippy::redundant_field_names)]
        Self {
            id: id,

            transform:        Affine::IDENTITY,
            global_transform: Affine::IDENTITY,
            size:             Size::new(0.0, 0.0),
            parent:           None,
            children:         Vec::new(),
            previous_space:   None,
            cursor:           CursorIcon::Default,
            window:           None,

            is_pixel_perfect: true,
            stable_draws:     0,
            clip:             None,

            is_hovered:  false,
            has_hovered: false,

            is_focused:  false,
            has_focused: false,

            is_active:  false,
            has_active: false,

            is_stashed:  false,
            is_disabled: false,

            needs_compose: false,
            needs_animate: false,
            needs_layout:  true,
            needs_draw:    true,

            accepts_pointer: T::accepts_pointer(),
            accepts_focus:   T::accepts_focus(),
            accepts_text:    T::accepts_text(),

            tracing_span: tracing::error_span!(
                "Widget",
                r#type = Self::short_type_name::<T>()
            ),
            short_name:   Self::short_type_name::<T>(),
            type_name:    std::any::type_name::<T>(),
        }
    }

    fn short_type_name<T>() -> &'static str {
        let name = std::any::type_name::<T>();
        name.split('<')
            .next()
            .unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or(name)
    }

    pub(crate) fn reset(&mut self) {
        self.has_hovered = self.is_hovered;
        self.has_active = self.is_active;
        self.has_focused = self.is_focused;
    }

    pub(crate) fn merge(&mut self, child: &Self) {
        self.has_hovered |= child.has_hovered;
        self.has_active |= child.has_active;
        self.has_focused |= child.has_focused;

        self.needs_compose |= child.needs_compose && !child.is_stashed;
        self.needs_animate |= child.needs_animate && !child.is_stashed;
        self.needs_layout |= child.needs_layout && !child.is_stashed;
        self.needs_draw |= child.needs_draw && !child.is_stashed;
    }

    pub fn accepts_focus(&self) -> bool {
        self.accepts_focus && !self.is_stashed
    }

    pub fn accepts_pointer(&self) -> bool {
        self.accepts_pointer && !self.is_stashed
    }
}
