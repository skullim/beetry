use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{id::NodeId, output::node::Parameters};

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    parameters: HashMap<NodeId, Values>,
}

impl Store {
    pub fn new(parameters: impl IntoIterator<Item = (NodeId, Values)>) -> Self {
        Self {
            parameters: parameters.into_iter().collect(),
        }
    }

    pub fn get(&self, id: &NodeId) -> Option<&Values> {
        self.parameters.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &Values)> {
        self.parameters.iter()
    }

    pub fn take(&mut self, id: &NodeId) -> Option<Values> {
        self.parameters.remove(id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Values {
    pub params: Parameters,
}
