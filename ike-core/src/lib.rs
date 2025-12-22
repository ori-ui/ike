#![warn(clippy::unwrap_used)]

mod arena;
mod axis;
mod border;
mod build;
mod canvas;
mod color;
mod context;
mod curve;
mod event;
mod image;
mod math;
mod painter;
mod record;
mod root;
mod svg;
mod text;
mod transition;
mod widget;
mod window;

pub mod widgets;

pub use arena::{Arena, WidgetMut, WidgetRef};
pub use axis::Axis;
pub use border::{BorderWidth, CornerRadius, Padding};
pub use build::BuildCx;
pub use canvas::{Blend, Canvas, Cap, Clip, Join, Paint, Shader, Stroke};
pub use color::Color;
pub use context::{ComposeCx, DrawCx, EventCx, LayoutCx, MutCx, RefCx, UpdateCx};
pub use curve::{Curve, CurveData, CurveSegment, CurveVerb, Fill, WeakCurve};
pub use event::{
    CursorIcon, Gesture, ImeEvent, Key, KeyEvent, KeyPressEvent, Modifiers, NamedKey, PanGesture,
    Pointer, PointerButton, PointerButtonEvent, PointerEvent, PointerId, PointerMoveEvent,
    PointerPropagate, PointerScrollEvent, Propagate, ScrollDelta, TapGesture, TextEvent,
    TextPasteEvent, Touch, TouchEvent, TouchId, TouchMoveEvent, TouchPressEvent, TouchPropagate,
};
pub use math::{Affine, Matrix, Offset, Point, Rect, Size, Space};
pub use painter::Painter;
pub use record::{Recorder, Recording, RecordingData, WeakRecording};
pub use root::{ImeSignal, Root, RootSignal, WindowUpdate};
pub use svg::{Svg, SvgData, WeakSvg};
pub use text::{
    FontStretch, FontStyle, FontWeight, GlyphCluster, Paragraph, TextAlign, TextDirection,
    TextLayoutLine, TextStyle, TextWrap, WeakParagraph,
};
pub use transition::{Transition, TransitionCurve, Transitionable, Transitioned};
pub use widget::{AnyWidget, AnyWidgetId, ChildUpdate, Update, Widget, WidgetId, WidgetState};
pub use window::{Window, WindowId, WindowSizing};
