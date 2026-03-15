pub mod svg;

use beetry_editor_types::output::channel::{
    ChannelConfig, ChannelConfigInput, ChannelKind, TokioChannelKind,
};
use dioxus::prelude::*;

use crate::{
    Point,
    ui::{channel, node},
};

#[derive(Clone, Copy, PartialEq)]
pub struct State {
    pub(crate) channel_dialog: Signal<channel::dialog::State>,
    pub(crate) parameter: Signal<node::parameter::State>,
    pub(crate) element_spawn_point: Signal<Point>,
    pub(crate) svg: svg::State,
    pub(crate) default_channel_config: CopyValue<ChannelConfig>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            channel_dialog: Signal::new(channel::dialog::State::default()),
            parameter: Signal::new(node::parameter::State::default()),
            element_spawn_point: Signal::new(Point::default()),
            svg: svg::State::new(),
            default_channel_config: CopyValue::new(ChannelConfig::new(ChannelConfigInput::new(
                1,
                ChannelKind::Tokio(TokioChannelKind::Mpsc),
            ))),
        }
    }
}
