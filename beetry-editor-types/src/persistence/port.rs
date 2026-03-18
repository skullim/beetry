use std::collections::{HashMap, HashSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    id::{NodeId, NodePortId, PortConnectionId},
    output::node::PortState,
};

pub type StateMap = HashMap<NodePortId, PortState>;

pub struct StateRecord {
    id: NodeId,
    state: StateMap,
}

impl StateRecord {
    pub fn new(id: NodeId, state: StateMap) -> Self {
        Self { id, state }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Store {
    states: HashMap<NodeId, StateMap>,
    connections: HashSet<PortConnectionId>,
}

impl Store {
    pub fn new(
        records: impl IntoIterator<Item = StateRecord>,
        connections: impl IntoIterator<Item = PortConnectionId>,
    ) -> Self {
        Self {
            states: records.into_iter().map(|r| (r.id, r.state)).collect(),
            connections: connections.into_iter().collect(),
        }
    }

    pub fn take_state(&mut self, id: &NodeId) -> Option<StateMap> {
        self.states.remove(id)
    }

    pub fn take_connections(&mut self) -> HashSet<PortConnectionId> {
        std::mem::take(&mut self.connections)
    }

    pub fn state(&self, node_id: &NodeId, port_id: &NodePortId) -> Option<&PortState> {
        self.states.get(node_id)?.get(port_id)
    }

    pub fn states(&self, node_id: &NodeId) -> Option<&StateMap> {
        self.states.get(node_id)
    }

    pub fn states_iter(&self) -> impl Iterator<Item = (&NodeId, &StateMap)> {
        self.states.iter()
    }

    pub fn connections_iter(&self) -> impl Iterator<Item = &PortConnectionId> {
        self.connections.iter()
    }
}
