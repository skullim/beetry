use crate::Point;
use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct State {
    pub(crate) dimensions: DimensionState,
    pub(crate) zoom: ZoomState,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            dimensions: DimensionState::new(),
            zoom: ZoomState::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DimensionState {
    width: Signal<f64>,
    height: Signal<f64>,
}

impl DimensionState {
    const DEFAULT_SIZE: f64 = 1600.0;
    const MARGIN: f64 = 400.0;

    pub(crate) fn new() -> Self {
        Self {
            width: Signal::new(Self::DEFAULT_SIZE),
            height: Signal::new(Self::DEFAULT_SIZE),
        }
    }

    pub(crate) fn width(&self) -> f64 {
        *self.width.read()
    }

    pub(crate) fn height(&self) -> f64 {
        *self.height.read()
    }

    pub(crate) fn resize<'a>(&mut self, positions: impl Iterator<Item = &'a Point>) {
        let (new_width, new_height) = positions
            .map(|node_pos| (node_pos.x + Self::MARGIN, node_pos.y + Self::MARGIN))
            .fold(
                (Self::DEFAULT_SIZE, Self::DEFAULT_SIZE),
                |(width, height), (x, y)| (width.max(x), height.max(y)),
            );

        self.width.set(new_width);
        self.height.set(new_height);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ZoomState(Signal<f64>);

impl ZoomState {
    pub(crate) fn new() -> Self {
        Self(Signal::new(1.0))
    }

    pub(crate) fn get(&self) -> f64 {
        *self.0.peek()
    }

    pub(crate) fn update(&mut self, wheel_delta: &WheelDelta) {
        let zoom_factor = match wheel_delta {
            WheelDelta::Pixels(vector) => {
                if vector.y > 0.0 {
                    0.95
                } else {
                    1.05
                }
            }
            WheelDelta::Lines(vector) => {
                if vector.y > 0.0 {
                    0.9
                } else {
                    1.1
                }
            }
            WheelDelta::Pages(vector) => {
                if vector.y > 0.0 {
                    0.8
                } else {
                    1.25
                }
            }
        };

        self.0
            .with_mut(|zoom| *zoom = (*zoom * zoom_factor).clamp(0.1, 10.0));
    }
}

impl Deref for ZoomState {
    type Target = Signal<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZoomState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
