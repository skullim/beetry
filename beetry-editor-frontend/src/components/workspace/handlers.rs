pub mod channel;
pub mod channel_edge;
pub mod edge;
pub mod node;
pub mod pin;

use beetry_editor_backend::{api, api::NodeUiQuery};
use dioxus::prelude::*;
pub use state::drag::{DragChannelState, DragNodeState};

use crate::{
    Backend, Point,
    components::{
        editor::state::svg::State as SvgState,
        workspace,
        workspace::state::{self, temporary},
    },
    signals::RenderRequests,
    ui::{channel as ui_channel, edge as ui_edge},
};

pub fn handlers(
    state: &workspace::State,
    mut element_spawn_point: Signal<Point>,
    svg: SvgState,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> super::Handlers {
    let mut drag = state.drag;
    let mut menu = state.menu;
    let mut temp = state.temp;
    let mut dimensions = svg.dimensions;
    let mut zoom = svg.zoom;

    let on_mouse_move = move |evt: Event<MouseData>| {
        evt.stop_propagation();
        if let DragNodeState::Dragged { id, offset } = *drag.node.peek() {
            let mouse_coords = evt.client_coordinates();

            let zoom = zoom.get();
            let updated_pos = Point {
                x: mouse_coords.x / zoom - offset.x,
                y: mouse_coords.y / zoom - offset.y,
            };
            backend.with_mut(|s| api::ui::node::update_position(s, id, updated_pos))?;
            requests.nodes.request();
            requests.edges.request();
            requests.channel_edges.request();

            backend.with_peek(|s| {
                let query = api::ui::node::query(s);
                dimensions.resize(query.positions());
            });
        }

        if let DragChannelState::Dragged { id, offset } = *drag.channel.peek() {
            let mouse_coords = evt.client_coordinates();

            let zoom = zoom.get();
            let updated_pos = Point {
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

        set_if_changed(&mut menu.node, crate::ui::node::menu::State::Idle);
        set_if_changed(&mut menu.edge, ui_edge::menu::State::Idle);
        set_if_changed(&mut menu.channel, ui_channel::menu::State::Idle);
        set_if_changed(&mut menu.channel_edge, ui_channel::edge_menu::State::Idle);
        set_if_changed(&mut menu.port, crate::ui::node::port::menu::State::Idle);

        temp.edge.with_mut(temporary::node_edge::State::reset);
        temp.channel.with_mut(temporary::channel_edge::State::reset);
        Ok(())
    };

    let on_wheel = move |evt: Event<WheelData>| {
        if evt.modifiers().ctrl() {
            evt.prevent_default();
            let delta = evt.delta();
            zoom.update(&delta);
        }
        Ok(())
    };

    let on_scroll = move |evt: Event<ScrollData>| {
        let zoom_level = zoom.get();
        let x = evt.scroll_left() / zoom_level;
        let y = evt.scroll_top() / zoom_level;
        element_spawn_point.set(Point { x, y });
        Ok(())
    };

    super::Handlers::new(on_mouse_move, on_mouse_up, on_wheel, on_scroll)
}

pub fn set_if_changed<T: Clone + PartialEq + 'static>(state: &mut Signal<T>, to: T) {
    if state.peek().ne(&to) {
        state.set(to);
    }
}
