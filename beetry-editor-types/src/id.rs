use derive_more::{AddAssign, Display};
use num_traits::One;
use serde::{Deserialize, Serialize};

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
pub struct ChannelId {
    id: u16,
}

impl ChannelId {
    pub fn new(id: u16) -> Self {
        Self { id }
    }

    pub fn next(&self) -> Self {
        ChannelId { id: self.id + 1 }
    }
}

impl One for ChannelId {
    fn one() -> Self {
        Self::new(1)
    }
}

impl std::ops::Mul for ChannelId {
    type Output = ChannelId;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            id: self.id * rhs.id,
        }
    }
}
