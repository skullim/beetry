use dioxus::prelude::*;

use crate::{Point, signals::RenderRequests, ui::channel, ui::node};

#[derive(Clone, Copy)]
pub(crate) struct State {
    pub(crate) channel_config: Signal<channel::config::State>,
    pub(crate) parameter: Signal<node::parameter::State>,
    pub(crate) element_spawn_point: Signal<Point>,
    pub(crate) render_requests: RenderRequests,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            channel_config: Signal::new(channel::config::State::default()),
            parameter: Signal::new(node::parameter::State::default()),
            element_spawn_point: Signal::new(Point::default()),
            render_requests: RenderRequests::default(),
        }
    }
}
