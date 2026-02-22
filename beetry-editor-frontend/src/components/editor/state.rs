use dioxus::prelude::*;
use beetry_editor_types::output::channel::{
    ChannelConfig, ChannelConfigInput, ChannelKind, TokioChannelKind,
};

use crate::{Point, ui::channel, ui::node};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct State {
    pub(crate) channel_dialog_state: Signal<channel::dialog::State>,
    pub(crate) parameter: Signal<node::parameter::State>,
    pub(crate) element_spawn_point: Signal<Point>,
    pub(crate) default_channel_config: CopyValue<ChannelConfig>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            channel_dialog_state: Signal::new(channel::dialog::State::default()),
            parameter: Signal::new(node::parameter::State::default()),
            element_spawn_point: Signal::new(Point::default()),
            default_channel_config: CopyValue::new(ChannelConfig::new(ChannelConfigInput::new(
                1,
                ChannelKind::Tokio(TokioChannelKind::Mpsc),
            ))),
        }
    }
}
