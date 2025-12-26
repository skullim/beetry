use beetry_editor_types::{output::channel::ChannelConfig, spec::channel::ChannelSpec};
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct ChannelSnapshot {
    spec: ChannelSpec,
    config: ChannelConfig,
}

impl ChannelSnapshot {
    pub fn new(spec: ChannelSpec, config: ChannelConfig) -> Self {
        Self { spec, config }
    }
}
