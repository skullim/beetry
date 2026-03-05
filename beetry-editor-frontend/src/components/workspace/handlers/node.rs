use beetry_editor_backend::api;
use beetry_editor_backend::api::SpecByNodeIdQuery;
use beetry_editor_types::{id::NodeId, output::ui::Point};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::ui::node::parameter::DEFAULT_DIALOG_POSITION;
use crate::{
    components::workspace::state::{drag, menu, svg},
    ui::node::{self},
};

use super::{Backend, DragNodeState, RenderRequests};
use crate::ui::error::ErrorQueueState;

pub(crate) fn handlers(
    mut drag: drag::State,
    mut menu: menu::State,
    svg: svg::State,
    backend: Backend,
    mut errors: ErrorQueueState,
) -> node::base::Handlers {
    let on_menu = move |(id, position): (NodeId, Point)| {
        let can_edit_params = backend
            .with(|s| -> anyhow::Result<bool> {
                let spec_query = api::node::spec::by_node_id(s);
                Ok(spec_query.spec(id)?.has_params())
            })
            .unwrap_or_else(|e| {
                errors.push(e);
                false
            });

        menu.node.set(node::menu::State::Visible {
            position,
            id,
            can_edit_params,
        });
        Ok(())
    };

    let on_mouse_down = move |(id, position, evt): (NodeId, Point, Event<MouseData>)| {
        evt.stop_propagation();

        if evt.held_buttons().contains(MouseButton::Primary) {
            let mouse_coords = evt.client_coordinates();
            let zoom_level = svg.zoom.get();

            let svg_mouse_coords = Point {
                x: mouse_coords.x / zoom_level,
                y: mouse_coords.y / zoom_level,
            };

            let offset = Point {
                x: svg_mouse_coords.x - position.x,
                y: svg_mouse_coords.y - position.y,
            };

            drag.node.set(DragNodeState::Dragged { id, offset });
        }
        Ok(())
    };
    node::base::Handlers::new(on_menu, on_mouse_down)
}

pub(crate) fn menu_handlers(
    mut menu: menu::State,
    mut backend: Backend,
    mut requests: RenderRequests,
    mut parameter_state: Signal<node::parameter::State>,
) -> node::menu::Handlers {
    let on_delete = move |id: NodeId| -> Result<()> {
        backend.with_mut(|s| api::node::remove(s, id))?;
        requests.nodes.request();
        requests.edges.request();
        requests.channel_edges.request();
        Ok(())
    };

    let on_close = move |_| {
        menu.node.set(node::menu::State::Idle);
        Ok(())
    };
    let on_update = move |id: NodeId| {
        parameter_state.set(node::parameter::State::Visible {
            position: DEFAULT_DIALOG_POSITION,
            id,
            mode: node::parameter::Mode::Update,
        });
        Ok(())
    };

    node::menu::Handlers::new(on_delete, on_update, on_close)
}
