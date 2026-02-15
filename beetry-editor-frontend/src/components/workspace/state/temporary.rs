use dioxus::signals::Signal;

pub mod channel_edge;
pub mod node_edge;

#[derive(Clone, Copy)]
pub(crate) struct State {
    pub(crate) edge: Signal<node_edge::State>,
    pub(crate) channel: Signal<channel_edge::State>,
}
