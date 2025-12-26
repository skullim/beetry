use crate::id::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeEdge {
    pub from: NodeId,
    pub to: NodeId,
}
