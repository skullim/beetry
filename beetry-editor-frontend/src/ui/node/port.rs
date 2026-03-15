pub mod body;
pub mod menu;
pub mod renderer;

pub use body::Body;
pub use menu::Menu;
pub use renderer::{NodeDimensions, Renderer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionOrigin {
    Sender,
    Receiver,
}

pub mod layout {
    pub const HEIGHT: f64 = 20.0;
    pub const MIN_GAP: f64 = 1.0;
    pub const TEXT_BASELINE_OFFSET: f64 = 13.0;
}
