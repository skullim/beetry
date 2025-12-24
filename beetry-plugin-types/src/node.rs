use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, From,
)]
pub struct NodeName(pub String);

impl NodeName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl From<&'static str> for NodeName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

//@todo harmonize with NodeKind
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LeafKind {
    Action,
    Condition,
}
