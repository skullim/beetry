mod leaves;

pub use leaves::{
    CheckBattery, CheckBatteryParams, Drive, DrivePlugin, DriveReceivers, Localize, LocalizePlugin,
    ReadExternalDataReceivers,
};

use beetry_macros::{Message, submit_as_channel_plugin};
use beetry_plugin::Plugin;
use beetry_plugin::channel::{ChannelPluginConstructor, Factory};
use beetry_plugin::node::{ActionPluginConstructor, ConditionPluginConstructor};
use beetry_plugin_types::channel::{ChannelSpec, Message};

use type_hash::TypeHash;

use crate::leaves::CheckBatteryPlugin;

//#[submit_as_channel_plugin]
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

pub struct ChannelPose {
    spec: ChannelSpec,
    factory: Factory,
}

impl Plugin for ChannelPose {
    type Spec = ChannelSpec;
    type Factory = Factory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            spec: ChannelSpec::new::<Pose>(),
            factory: Factory::from_msg_type::<Pose>(),
        }
    }

    fn spec(&self) -> &ChannelSpec {
        &self.spec
    }

    fn factory(&self) -> &Factory {
        &self.factory
    }

    fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
        (self.spec, self.factory)
    }
}

beetry_plugin::submit!(ChannelPluginConstructor::new::<ChannelPose>());

#[derive(Debug, Clone, Copy, TypeHash)]
pub enum ChargeCommand {
    Start,
    Stop,
}

//@todo adapt macro to new interface
//#[submit_as_channel_plugin]
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

beetry_plugin::submit!(ActionPluginConstructor::new::<DrivePlugin>());
beetry_plugin::submit!(ActionPluginConstructor::new::<LocalizePlugin>());
// beetry_plugin::submit!(ActionPluginConstructor::new::<ReadExternalDataPlugin>());
beetry_plugin::submit!(ConditionPluginConstructor::new::<CheckBatteryPlugin>());
