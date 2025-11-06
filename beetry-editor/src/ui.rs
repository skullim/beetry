pub mod channel;
pub mod curve;
pub mod edge;
pub mod node;
pub mod shadow;
pub mod text;
pub mod transfer;
pub mod viewport;

use beetry_core::MessageHash;
use beetry_serde::{
    de::parameter::Parameters,
    ser::{
        node::{LeafSchema, NodeName},
        parameter,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

use crate::definitions::{NodeId, Point};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Root,
    Control {
        params_schema: parameter::Schema,
    },
    Leaf {
        schema: LeafSchema,
        external_receivers: BTreeSet<MessageHash>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub(crate) name: NodeName,
    pub(crate) kind: NodeKind,
    // placeholder for future selection (if any)
    pub(crate) selected_params: Parameters,
    pub(crate) pos: Point,
}

impl Node {
    pub(crate) fn new(name: NodeName, kind: NodeKind) -> Self {
        Self {
            name,
            kind,
            selected_params: Parameters::default(),
            pos: Point::default(),
        }
    }

    pub(crate) fn with_params(mut self, params: Parameters) -> Self {
        self.selected_params = params;
        self
    }
}

pub type NodeMap = HashMap<NodeId, Node>;
