use dioxus::signals::Signal;

pub mod channel_edge;
pub mod node_edge;

#[derive(Clone, Copy, PartialEq)]
pub struct State {
    pub edge: Signal<node_edge::State>,
    pub channel: Signal<channel_edge::State>,
}
