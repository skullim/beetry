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
    pub(crate) reload_ws: ReloadWorkspaceFlag,
}

/// Requests dropping the current `Workspace` subtree so Dioxus mounts a fresh
/// one.
///
/// This is used after project import because imported nodes/channels may reuse
/// ids that already existed in the UI. Without remounting, Dioxus can preserve
/// component-local state for those ids and stale frontend data may remain
/// visible.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct ReloadWorkspaceFlag {
    flag: Signal<bool>,
}

impl ReloadWorkspaceFlag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self) {
        self.flag.set(true);
    }

    pub fn read_val(&self) -> bool {
        (self.flag)()
    }

    pub fn clear(&mut self) {
        self.flag.toggle();
    }
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
            reload_ws: ReloadWorkspaceFlag::new(),
        }
    }
}
