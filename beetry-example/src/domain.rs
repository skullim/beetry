use beetry::{Message, type_hash, type_hash::TypeHash};

pub mod parking;

/// This modules defines nodes that are related to checking out how UI works.
pub mod ui;

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
}

impl Pose {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

beetry::plugin::channel! {PoseChannel: Pose}
