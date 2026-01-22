use std::{f32::consts::PI, ops::Deref, time::Duration};

use crate::{Color, Offset, Padding, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransitionCurve {
    Linear,
    Ease,
    ElasticIn,
    ElasticOut,
    BackIn,
    BackOut,
    Back,
}

impl TransitionCurve {
    pub fn apply(self, t: f32) -> f32 {
        const C: f32 = 1.70158;
        const C2: f32 = C * 1.525;

        match self {
            TransitionCurve::Linear => t,

            TransitionCurve::Ease => t * t * (3.0 - 2.0 * t),

            TransitionCurve::ElasticIn => {
                -f32::powf(2.0, 10.0 * t - 10.0) * f32::sin((10.0 * t - 10.75) * PI * 2.0 / 3.0)
            }

            TransitionCurve::ElasticOut => {
                1.0 + f32::powf(2.0, -10.0 * t) * f32::sin((10.0 * t - 0.75) * PI * 2.0 / 3.0)
            }

            TransitionCurve::Back => {
                if t < 0.5 {
                    (f32::powi(2.0 * t, 2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
                } else {
                    (f32::powi(2.0 * t - 2.0, 2) * ((C2 + 1.0) * (2.0 * t - 2.0) + C2) + 2.0) / 2.0
                }
            }

            TransitionCurve::BackIn => (C + 1.0) * f32::powi(t, 3) - C * f32::powi(t, 2),

            TransitionCurve::BackOut => {
                1.0 + (C + 1.0) * f32::powi(t - 1.0, 3) + C * f32::powi(t - 1.0, 2)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub curve:    TransitionCurve,
    pub duration: f32,
}

impl Transition {
    /// Instant transition.
    pub const INSTANT: Self = Self::linear(0.0);

    pub const fn linear(duration: f32) -> Self {
        Self {
            curve: TransitionCurve::Linear,
            duration,
        }
    }

    pub const fn ease(duration: f32) -> Self {
        Self {
            curve: TransitionCurve::Ease,
            duration,
        }
    }

    pub const fn elastic_in(duration: f32) -> Self {
        Self {
            curve: TransitionCurve::ElasticIn,
            duration,
        }
    }

    pub const fn elastic_out(duration: f32) -> Self {
        Self {
            curve: TransitionCurve::ElasticOut,
            duration,
        }
    }

    pub const fn back_in(duration: f32) -> Self {
        Self {
            curve: TransitionCurve::BackIn,
            duration,
        }
    }

    pub const fn back_out(duration: f32) -> Self {
        Self {
            curve: TransitionCurve::BackOut,
            duration,
        }
    }

    pub const fn back(duration: f32) -> Self {
        Self {
            curve: TransitionCurve::Back,
            duration,
        }
    }
}

pub trait Interpolate {
    fn interpolate(start: &Self, end: &Self, x: f32) -> Self;
}

pub struct Transitioned<T> {
    transition: Transition,
    current:    T,
    start:      T,
    end:        T,
    time:       f32,
}

impl<T> Transitioned<T>
where
    T: Interpolate + Clone + PartialEq,
{
    pub fn new(value: T, transition: Transition) -> Self {
        Self {
            current: value.clone(),
            start: value.clone(),
            end: value,
            time: transition.duration,
            transition,
        }
    }

    pub fn set_transition(&mut self, transition: Transition) {
        if self.is_complete() {
            self.time = transition.duration;
        }

        self.transition = transition;

        self.update_current();
    }

    /// Set a concrete value, and cancel the current transition.
    pub fn set(&mut self, value: T) {
        self.end = value.clone();
        self.start = value.clone();
        self.current = value.clone();
        self.time = self.transition.duration;
    }

    /// Get the starting value.
    pub fn start(&self) -> T {
        self.start.clone()
    }

    /// Get the end value.
    pub fn end(&self) -> T {
        self.end.clone()
    }

    /// Get the current value.
    pub fn get(&self) -> T {
        self.current.clone()
    }

    /// Start transitioning to a value.
    ///
    /// Returns whether `request_animate` should be called.
    pub fn begin(&mut self, target: T) -> bool {
        if target == self.end {
            return false;
        }

        self.start = self.current.clone();
        self.end = target;
        self.time = 0.0;

        self.update_current();

        !self.is_complete()
    }

    /// Animate the value.
    ///
    /// Returns whether `request_animate` should be called.
    pub fn animate(&mut self, dt: Duration) -> bool {
        self.time += dt.as_secs_f32();
        self.time = self.time.clamp(0.0, self.transition.duration);

        self.update_current();

        !self.is_complete()
    }

    /// Check if the transition has reached the end.
    pub fn is_complete(&self) -> bool {
        self.time >= self.transition.duration
    }

    fn update_current(&mut self) {
        if self.transition.duration == 0.0 {
            self.current = self.end.clone();
            return;
        }

        let fraction = self.time / self.transition.duration;
        let position = self.transition.curve.apply(fraction);

        self.current = T::interpolate(&self.start, &self.end, position);
    }
}

impl<T> Deref for Transitioned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl Interpolate for f32 {
    fn interpolate(start: &Self, end: &Self, x: f32) -> Self {
        *start * (1.0 - x) + *end * x
    }
}

impl Interpolate for Color {
    fn interpolate(start: &Self, end: &Self, x: f32) -> Self {
        Self {
            r: f32::interpolate(&start.r, &end.r, x),
            g: f32::interpolate(&start.g, &end.g, x),
            b: f32::interpolate(&start.b, &end.b, x),
            a: f32::interpolate(&start.a, &end.a, x),
        }
    }
}

impl Interpolate for Offset {
    fn interpolate(from: &Self, to: &Self, x: f32) -> Self {
        Self {
            x: f32::interpolate(&from.x, &to.x, x),
            y: f32::interpolate(&from.y, &to.y, x),
        }
    }
}

impl Interpolate for Size {
    fn interpolate(from: &Self, to: &Self, x: f32) -> Self {
        Self {
            width:  f32::interpolate(&from.width, &to.width, x),
            height: f32::interpolate(&from.height, &to.height, x),
        }
    }
}

impl Interpolate for Padding {
    fn interpolate(start: &Self, end: &Self, x: f32) -> Self {
        Self {
            left:   f32::interpolate(&start.left, &end.left, x),
            top:    f32::interpolate(&start.top, &end.top, x),
            right:  f32::interpolate(&start.right, &end.right, x),
            bottom: f32::interpolate(&start.bottom, &end.bottom, x),
        }
    }
}
