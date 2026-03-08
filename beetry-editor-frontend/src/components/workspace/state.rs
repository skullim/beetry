pub mod drag;
pub mod menu;
pub mod temporary;

use dioxus::signals::Signal;

use crate::ui::{channel as ui_channel, edge as ui_edge, node as ui_node};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct State {
    pub(crate) drag: drag::State,
    pub(crate) menu: menu::State,
    pub(crate) temp: temporary::State,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            drag: drag::State {
                node: Signal::new(drag::DragNodeState::Idle),
                channel: Signal::new(drag::DragChannelState::Idle),
            },
            menu: menu::State {
                node: Signal::new(ui_node::menu::State::default()),
                edge: Signal::new(ui_edge::menu::State::default()),
                channel: Signal::new(ui_channel::menu::State::default()),
                channel_edge: Signal::new(ui_channel::edge_menu::State::default()),
                port: Signal::new(ui_node::port::menu::State::default()),
            },
            temp: temporary::State {
                edge: Signal::new(temporary::node_edge::State::new()),
                channel: Signal::new(temporary::channel_edge::State::new()),
            },
        }
    }
}
