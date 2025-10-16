use dioxus::{html::geometry::WheelDelta, prelude::*};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ZoomLevel(f64);

impl ZoomLevel {
    pub(crate) fn new() -> Self {
        Self(1.0)
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
        self.0 = (self.0 * zoom_factor).clamp(0.1, 10.0);
    }

    pub(crate) fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ViewportContext {
    pub(crate) zoom_level: Signal<ZoomLevel>,
}

impl ViewportContext {
    pub(crate) fn new() -> Self {
        Self {
            zoom_level: Signal::new(ZoomLevel::new()),
        }
    }
}
