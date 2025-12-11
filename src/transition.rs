use std::{ops::Deref, time::Duration};

use crate::{Color, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransitionCurve {
    Linear,
    Ease,
}

impl TransitionCurve {
    pub fn apply(self, x: f32) -> f32 {
        match self {
            TransitionCurve::Linear => x,
            TransitionCurve::Ease => x * x * (3.0 - 2.0 * x),
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
}

pub trait Transitionable {
    fn interpolate(from: &Self, to: &Self, x: f32) -> Self;
}

pub struct Transitioned<T> {
    transition: Transition,
    current:    T,
    from:       T,
    to:         T,
    time:       f32,
}

impl<T> Transitioned<T>
where
    T: Transitionable + Clone + PartialEq,
{
    pub fn new(value: T, transition: Transition) -> Self {
        Self {
            current: value.clone(),
            from: value.clone(),
            to: value,
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

    pub fn set(&mut self, value: T) {
        self.to = value.clone();
        self.from = value.clone();
        self.current = value.clone();
        self.time = self.transition.duration;
    }

    /// Start transitioning to a value.
    ///
    /// Returns whether `request_animate` should be called.
    pub fn begin(&mut self, to: T) -> bool {
        if to == self.to {
            return false;
        }

        self.from = self.current.clone();
        self.to = to;
        self.time = 0.0;

        self.update_current();

        !self.is_complete()
    }

    pub fn animate(&mut self, delta_time: Duration) -> bool {
        self.time += delta_time.as_secs_f32();
        self.time = self.time.clamp(0.0, self.transition.duration);

        self.update_current();

        !self.is_complete()
    }

    pub fn is_complete(&self) -> bool {
        self.time >= self.transition.duration
    }

    fn update_current(&mut self) {
        if self.transition.duration == 0.0 {
            self.current = self.to.clone();
            return;
        }

        let fraction = self.time / self.transition.duration;
        let position = self.transition.curve.apply(fraction);

        self.current = T::interpolate(&self.from, &self.to, position);
    }
}

impl<T> Deref for Transitioned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl Transitionable for f32 {
    fn interpolate(from: &Self, to: &Self, x: f32) -> Self {
        *from * (1.0 - x) + *to * x
    }
}

impl Transitionable for Color {
    fn interpolate(from: &Self, to: &Self, x: f32) -> Self {
        Self {
            r: f32::interpolate(&from.r, &to.r, x),
            g: f32::interpolate(&from.g, &to.g, x),
            b: f32::interpolate(&from.b, &to.b, x),
            a: f32::interpolate(&from.a, &to.a, x),
        }
    }
}

impl Transitionable for Size {
    fn interpolate(from: &Self, to: &Self, x: f32) -> Self {
        Self {
            width:  f32::interpolate(&from.width, &to.width, x),
            height: f32::interpolate(&from.height, &to.height, x),
        }
    }
}
