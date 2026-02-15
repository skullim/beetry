use beetry_editor_backend::api::NodeUiQueryApi;
use bon::Builder;
use dioxus::logger::tracing::debug;
use dioxus::prelude::*;

use crate::ui::channel;
use crate::ui::edge;

use super::{
    Backend, CanvasState, DragChannelState, DragNodeState, DragState, MenuState, RenderRequests,
    TempState,
};

#[derive(Debug, Clone, Builder)]
pub(crate) struct WorkspaceEventHandlers {
    pub(crate) on_mouse_move: EventHandler<Event<MouseData>>,
    pub(crate) on_mouse_up: EventHandler<Event<MouseData>>,
    pub(crate) on_wheel: EventHandler<Event<WheelData>>,
}

pub(crate) fn set_if_changed<T: Clone + PartialEq + 'static>(state: &mut Signal<T>, next: T) {
    if state.peek().ne(&next) {
        state.set(next);
    }
}

pub(crate) fn workspace_event_handlers(
    mut drag: DragState,
    mut menus: MenuState,
    mut canvas: CanvasState,
    mut temp: TempState,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> WorkspaceEventHandlers {
    let mut dimensions_ctx = canvas.dimensions;
    let on_mouse_move = move |evt: Event<MouseData>| -> Result<()> {
        evt.stop_propagation();
        if let DragNodeState::Dragged { id, offset } = *drag.node.peek() {
            debug!("inside dragged node");
            let mouse_coords = evt.client_coordinates();

            let zoom = canvas.viewport.zoom_level.peek().get();
            let updated_pos = crate::Point {
                x: mouse_coords.x / zoom - offset.x,
                y: mouse_coords.y / zoom - offset.y,
            };
            backend.with_mut(|s| {
                beetry_editor_backend::api::ui::node::update_position(s, id, updated_pos)
            })?;
            requests.nodes.request();
            requests.edges.request();
            requests.channel_edges.request();

            let read = backend.read();
            let query_api = beetry_editor_backend::api::ui::node::borrow(&(*read));
            let positions_iter = query_api.positions();
            dimensions_ctx.resize_if_needed(positions_iter);
        }

        if let DragChannelState::Dragged { id, offset } = *drag.channel.peek() {
            let mouse_coords = evt.client_coordinates();

            let zoom = canvas.viewport.zoom_level.peek().get();
            let updated_pos = crate::Point {
                x: mouse_coords.x / zoom - offset.x,
                y: mouse_coords.y / zoom - offset.y,
            };

            backend.with_mut(|s| {
                beetry_editor_backend::api::ui::channel::update_position(s, id, updated_pos)
            })?;
            requests.channels.request();
            requests.channel_edges.request();
        }

        temp.edge.update_end_if_dragged(&evt);
        temp.channel.update_end_if_dragged(&evt);
        Ok(())
    };

    let on_mouse_up = move |evt: Event<MouseData>| {
        debug!("detected mouse up in workspace");
        evt.stop_propagation();

        set_if_changed(&mut drag.node, DragNodeState::Idle);
        set_if_changed(&mut drag.channel, DragChannelState::Idle);

        set_if_changed(&mut menus.node, crate::ui::node::ContextMenuState::Idle);
        set_if_changed(&mut menus.edge, edge::ContextMenuState::Idle);
        set_if_changed(&mut menus.channel, channel::ContextMenuState::Idle);
        set_if_changed(
            &mut menus.port,
            crate::ui::node::port_context_menu::State::Idle,
        );

        temp.edge.reset();
        temp.channel.reset();
    };

    let on_wheel = move |evt: Event<WheelData>| {
        if evt.modifiers().ctrl() {
            evt.prevent_default();
            let delta = evt.delta();
            canvas
                .viewport
                .zoom_level
                .with_mut(|level| level.update(&delta));
        }
    };

    WorkspaceEventHandlers::builder()
        .on_mouse_move(EventHandler::new(on_mouse_move))
        .on_mouse_up(EventHandler::new(on_mouse_up))
        .on_wheel(EventHandler::new(on_wheel))
        .build()
}

pub(crate) fn grid_style_defs() -> Element {
    rsx! {
        defs {
            pattern {
                id: "grid",
                width: "50",
                height: "50",
                pattern_units: "userSpaceOnUse",

                path {
                    d: "M 50 0 L 0 0 0 50",
                    fill: "none",
                    stroke: "#d0d0d0",
                    stroke_width: "2",
                }
            }
        }
    }
}
