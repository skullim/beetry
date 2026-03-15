use dioxus::prelude::*;

use crate::ui::{channel as ui_channel, edge as ui_edge, node as ui_node};

#[derive(Clone, Copy, PartialEq)]
pub struct State {
    pub node: Signal<ui_node::menu::State>,
    pub edge: Signal<ui_edge::menu::State>,
    pub channel: Signal<ui_channel::menu::State>,
    pub channel_edge: Signal<ui_channel::edge_menu::State>,
    pub port: Signal<ui_node::port::menu::State>,
}
