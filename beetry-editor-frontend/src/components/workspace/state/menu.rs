use crate::ui::{channel as ui_channel, edge as ui_edge, node as ui_node};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct State {
    pub(crate) node: Signal<ui_node::menu::State>,
    pub(crate) edge: Signal<ui_edge::menu::State>,
    pub(crate) channel: Signal<ui_channel::menu::State>,
    pub(crate) port: Signal<ui_node::port::menu::State>,
}
