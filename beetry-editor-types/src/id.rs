use derive_more::{AddAssign, Display};
use num_traits::One;
use serde::{Deserialize, Serialize};

macro_rules! derive_id {
    ($name:ident as $ty:ty) => {
        #[derive(
            Debug,
            Default,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            Display,
            AddAssign,
        )]
        #[serde(transparent)]
        pub struct $name {
            id: $ty,
        }

        impl $name {
            pub fn new(id: $ty) -> Self {
                Self { id }
            }

            pub fn next(&self) -> Self {
                Self { id: self.id + 1 }
            }

            pub fn raw_value(&self) -> $ty {
                self.id
            }
        }

        impl One for $name {
            fn one() -> Self {
                Self::new(1)
            }
        }

        impl std::ops::Mul for $name {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self {
                    id: self.id * rhs.id,
                }
            }
        }
    };
}

derive_id!(NodeId as u16);
derive_id!(NodeSpecId as u16);
derive_id!(NodePortId as u8);
derive_id!(EdgeId as u16);

derive_id!(ChannelId as u16);
derive_id!(ChannelSpecId as u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortConnectionId {
    pub node_id: NodeId,
    pub port_id: NodePortId,
    pub channel_id: ChannelId,
}

impl PortConnectionId {
    pub fn new(node_id: NodeId, port_id: NodePortId, channel_id: ChannelId) -> Self {
        Self {
            node_id,
            port_id,
            channel_id,
        }
    }
}
