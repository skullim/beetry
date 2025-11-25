use crate::domain::models::{ChannelId, EdgeId, ExternalReceivers, ExternalSenders, NodeEdge};

use super::models::{NodeId, NodeKind, NodePosition};
use beetry_core::MessageHash;
use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::{
    de::parameter::Parameters,
    ser::node::{ControlSpec, DecoratorSpec, NodeName},
};

use anyhow::Result;
use serde_value::Value;

pub trait NodeRepository {
    fn create_root(&mut self, id: NodeId) -> Result<()>;
    fn create_action(&mut self, id: NodeId) -> Result<()>;
    fn create_condition(&mut self, id: NodeId) -> Result<()>;
    fn create_control(&mut self, id: NodeId) -> Result<()>;
    fn create_decorator(&mut self, id: NodeId) -> Result<()>;

    fn remove(&mut self, id: NodeId) -> Result<()>;

    fn kind(&self, id: NodeId) -> Option<NodeKind>;

    fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()>;

    fn position(&self, id: NodeId) -> Option<&NodePosition>;

    fn nodes(&self) -> &[NodeId];
}

pub trait SpecRepository {
    fn register_action(&mut self, spec: ActionSpec) -> Result<()>;
    fn register_condition(&mut self, spec: ConditionSpec) -> Result<()>;
    fn register_control(&mut self, spec: ControlSpec) -> Result<()>;
    fn register_decorator(&mut self, spec: DecoratorSpec) -> Result<()>;

    fn bind_action_id(&mut self, name: NodeName, id: NodeId) -> Result<()>;
    fn bind_condition_id(&mut self, name: NodeName, id: NodeId) -> Result<()>;
    fn bind_control_id(&mut self, name: NodeName, id: NodeId) -> Result<()>;
    fn bind_decorator_id(&mut self, name: NodeName, id: NodeId) -> Result<()>;

    fn action(&self, id: NodeId) -> Option<&ActionSpec>;
    fn condition(&self, id: NodeId) -> Option<&ConditionSpec>;
    fn control(&self, id: NodeId) -> Option<&ControlSpec>;
    fn decorator(&self, id: NodeId) -> Option<&DecoratorSpec>;
}

pub trait ParamRepository {
    fn insert(&mut self, id: NodeId, params: Parameters);
    fn update(&mut self, id: NodeId, field_name: &str, value: Value) -> Result<()>;
    fn params(&self, id: NodeId) -> &Parameters;
}

pub trait ExternalPortRepository {
    fn insert_sender(&mut self, id: NodeId, sender: MessageHash);
    fn senders(&self, id: NodeId) -> &ExternalSenders;

    fn insert_receiver(&mut self, id: NodeId, receiver: MessageHash);
    fn receivers(&self, id: NodeId) -> &ExternalReceivers;
}

pub trait EdgeRepository {
    fn create(&mut self, edge: NodeEdge) -> Result<EdgeId>;
    fn remove(&mut self, id: EdgeId) -> Result<()>;
    fn edges(&self) -> &[EdgeId];
}

pub trait ChannelRepository {
    fn create(&mut self) -> Result<ChannelId>;
    fn remove(&mut self, id: ChannelId) -> Result<()>;
    fn channels(&self) -> &[ChannelId];
}
