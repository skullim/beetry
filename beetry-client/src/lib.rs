mod leaves;

pub use leaves::{
    CheckBattery, CheckBatteryParams, Drive, DriveInput, DrivePlugin, Localize, LocalizePlugin,
    ReadExternalDataInput,
};

use beetry_definitions::description::{ChannelDescription, Message};
use beetry_macros::{Message, submit_as_channel_plugin};
use beetry_plugin::{
    channel::{ChannelPlugin, ChannelPluginConstructor, Factory},
    node::{ActionNodePluginConstructor, ConditionNodePluginConstructor},
};
use type_hash::TypeHash;

use crate::leaves::{CheckBatteryPlugin, ReadExternalDataPlugin};

#[submit_as_channel_plugin]
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

#[derive(Debug, Clone, Copy, TypeHash)]
pub enum ChargeCommand {
    Start,
    Stop,
}

#[submit_as_channel_plugin]
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

inventory::submit!(ActionNodePluginConstructor::new::<DrivePlugin>());
inventory::submit!(ActionNodePluginConstructor::new::<LocalizePlugin>());
inventory::submit!(ActionNodePluginConstructor::new::<ReadExternalDataPlugin>());

inventory::submit!(ConditionNodePluginConstructor::new::<CheckBatteryPlugin>());
