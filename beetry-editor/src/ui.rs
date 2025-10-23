pub(crate) mod channel;
pub(crate) mod curve;
pub(crate) mod edge;
pub(crate) mod node;
pub(crate) mod shadow;
pub(crate) mod text;
pub(crate) mod transfer;
pub(crate) mod viewport;

use beetry_core::MessageHash;
use beetry_serde::{
    de::{node::ControlKind, parameter::Parameters},
    ser::node::LeafSpec,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

use crate::definitions::{NodeId, Point};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum NodeKind {
    Root,
    Control(ControlKind),
    Leaf {
        desc: LeafSpec,
        params: Parameters,
        external_receivers: BTreeSet<MessageHash>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Node {
    pub(crate) kind: NodeKind,
    pub(crate) pos: Point,
}

impl Node {
    pub(crate) fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            pos: Point::default(),
        }
    }
}

pub(crate) type NodeMap = HashMap<NodeId, Node>;
