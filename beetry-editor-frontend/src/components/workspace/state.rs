pub mod drag;
pub mod menu;
pub mod svg;
pub mod temporary;

use dioxus::signals::Signal;

use crate::components::workspace::state::svg::{DimensionState, ZoomState};
use crate::ui::{channel as ui_channel, edge as ui_edge, node as ui_node};

#[derive(Clone, Copy)]
pub(crate) struct State {
    pub(crate) drag: drag::State,
    pub(crate) menu: menu::State,
    pub(crate) svg: svg::State,
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
                port: Signal::new(ui_node::port::menu::State::default()),
            },
            svg: svg::State {
                dimensions: DimensionState::new(),
                zoom: ZoomState::new(),
            },
            temp: temporary::State {
                edge: Signal::new(temporary::node_edge::State::new()),
                channel: Signal::new(temporary::channel_edge::State::new()),
            },
        }
    }
}
