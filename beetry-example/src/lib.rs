mod leaves;

use beetry_editor_types::spec::message::Message;
use beetry_macros::Message;
pub use leaves::{
    CheckBattery, CheckBatteryParams, CheckBatteryPlugin, Drive, DrivePlugin, DriveReceivers,
    Localize, LocalizePlugin, MultiPosePorts, MultiPosePortsPlugin, MultiPosePortsReceivers,
    ReadExternalDataPlugin, ReadExternalDataReceivers,
};
use type_hash::TypeHash;

#[derive(Debug, Clone, Copy, TypeHash, Message)]
pub struct Pose {
    x: f32,
    y: f32,
}

impl Pose {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

beetry_plugin::channel! {PoseChannel: Pose}

#[derive(Debug, Clone, Copy, TypeHash)]
pub enum ChargeCommand {
    Start,
    Stop,
}

#[derive(Debug, Clone, Copy, TypeHash, Message)]
pub struct ExternalData {
    pub charge_command: ChargeCommand,
    pub is_charger_present: bool,
}

impl ExternalData {
    pub fn new(charge_command: ChargeCommand, is_charger_present: bool) -> Self {
        Self {
            charge_command,
            is_charger_present,
        }
    }
}

beetry_plugin::channel! {ExternalDataChannel: ExternalData}
