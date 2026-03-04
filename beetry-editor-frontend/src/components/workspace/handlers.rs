pub(crate) mod channel;
pub(crate) mod channel_edge;
pub(crate) mod edge;
pub(crate) mod node;
pub(crate) mod pin;

use crate::Backend;
use crate::components::workspace::state;
use crate::signals::RenderRequests;
use crate::{Point, components::workspace};
use beetry_editor_backend::api;
use beetry_editor_backend::api::NodeUiQueryApi;
use dioxus::logger::tracing::debug;
use dioxus::prelude::*;

use crate::ui::{channel as ui_channel, edge as ui_edge};

pub(crate) use state::drag::{DragChannelState, DragNodeState};

pub(crate) fn handlers(
    state: &workspace::State,
    mut element_spawn_point: Signal<Point>,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> super::Handlers {
    let mut drag = state.drag;
    let mut menus = state.menu;
    let mut svg = state.svg;
    let mut temp = state.temp;

    let mut dimensions_state = svg.dimensions;
    let on_mouse_move = move |evt: Event<MouseData>| {
        evt.stop_propagation();
        if let DragNodeState::Dragged { id, offset } = *drag.node.peek() {
            let mouse_coords = evt.client_coordinates();

            let zoom = svg.zoom.get();
            let updated_pos = crate::Point {
                x: mouse_coords.x / zoom - offset.x,
                y: mouse_coords.y / zoom - offset.y,
            };
            backend.with_mut(|s| api::ui::node::update_position(s, id, updated_pos))?;
            requests.nodes.request();
            requests.edges.request();
            requests.channel_edges.request();

            backend.with_peek(|s| {
                let query = api::ui::node::borrow(s);
                dimensions_state.resize_if_needed(query.positions());
            });
        }

        if let DragChannelState::Dragged { id, offset } = *drag.channel.peek() {
            let mouse_coords = evt.client_coordinates();

            let zoom = svg.zoom.get();
            let updated_pos = crate::Point {
                x: mouse_coords.x / zoom - offset.x,
                y: mouse_coords.y / zoom - offset.y,
            };

            backend.with_mut(|s| api::ui::channel::update_position(s, id, updated_pos))?;
            requests.channels.request();
            requests.channel_edges.request();
        }

        let mouse_coords = evt.element_coordinates();
        let cursor = Point {
            x: mouse_coords.x,
            y: mouse_coords.y,
        };
        temp.edge.with_mut(|e| e.update_end_if_dragged(cursor));
        temp.channel.with_mut(|c| c.update_end_if_dragged(cursor));
        Ok(())
    };

    let on_mouse_up = move |evt: Event<MouseData>| {
        debug!("detected mouse up in workspace");
        evt.stop_propagation();

        set_if_changed(&mut drag.node, DragNodeState::Idle);
        set_if_changed(&mut drag.channel, DragChannelState::Idle);

        set_if_changed(&mut menus.node, crate::ui::node::menu::State::Idle);
        set_if_changed(&mut menus.edge, ui_edge::menu::State::Idle);
        set_if_changed(&mut menus.channel, ui_channel::menu::State::Idle);
        set_if_changed(&mut menus.channel_edge, ui_channel::edge_menu::State::Idle);
        set_if_changed(&mut menus.port, crate::ui::node::port::menu::State::Idle);

        temp.edge.with_mut(|e| e.reset());
        temp.channel.with_mut(|c| c.reset());
        Ok(())
    };

    let on_wheel = move |evt: Event<WheelData>| {
        if evt.modifiers().ctrl() {
            evt.prevent_default();
            let delta = evt.delta();
            svg.zoom.update(&delta);
        }
        Ok(())
    };

    let on_scroll = move |evt: Event<ScrollData>| {
        let zoom_level = svg.zoom.get();
        let x = evt.scroll_left() / zoom_level;
        let y = evt.scroll_top() / zoom_level;
        element_spawn_point.set(Point { x, y });
        Ok(())
    };

    super::Handlers::new(on_mouse_move, on_mouse_up, on_wheel, on_scroll)
}

pub(crate) fn set_if_changed<T: Clone + PartialEq + 'static>(state: &mut Signal<T>, to: T) {
    if state.peek().ne(&to) {
        state.set(to);
    }
}
