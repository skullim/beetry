use dioxus::{html::geometry::WheelDelta, prelude::*};

#[derive(Debug, Clone, Copy)]
pub struct ZoomLevel(f64);

impl ZoomLevel {
    pub fn new() -> Self {
        Self(1.0)
    }

    pub fn update(&mut self, wheel_delta: &WheelDelta) {
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

    pub fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportContext {
    pub zoom_level: Signal<ZoomLevel>,
}

impl ViewportContext {
    pub fn new() -> Self {
        Self {
            zoom_level: Signal::new(ZoomLevel::new()),
        }
    }
}
