use beetry_editor_types::{id::ChannelId, output::ui::Point};
use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::ui::channel::temporary::ConnectionOrigin;
use crate::ui::error_dialog::ErrorQueueState;
use crate::{components::workspace::providers::WorkspaceSvgState, ui::channel};

use super::{Backend, DragChannelState, DragState, MenuState, RenderRequests, TempState};

pub(crate) fn handlers(
    mut drag: DragState,
    mut menus: MenuState,
    mut temp: TempState,
    svg: WorkspaceSvgState,
    mut backend: Backend,
    mut requests: RenderRequests,
    mut errors: ErrorQueueState,
) -> channel::Handlers {
    let receiver_on_mouse_up = move |id: ChannelId| -> Result<()> {
        if let Some(data) = temp.channel.take_dragged()
            && matches!(data.origin, ConnectionOrigin::Receiver)
        {
            match backend.with_mut(|s| {
                beetry_editor_backend::api::node::ports::connect(s, data.node_id, data.port_id, id)
            }) {
                Ok(()) => {
                    requests.channel_edges.request();
                    info!(
                        "connected channel {id} and node (id: {}, port_id: {})",
                        data.node_id, data.port_id
                    );
                }
                Err(err) => {
                    error!("{err}");
                    errors.with_mut(|q| q.push("channel-connect", err.to_string()));
                }
            }
        }
        Ok(())
    };

    let sender_on_mouse_up = move |id: ChannelId| -> Result<()> {
        if let Some(data) = temp.channel.take_dragged()
            && matches!(data.origin, ConnectionOrigin::Sender)
        {
            match backend.with_mut(|s| {
                beetry_editor_backend::api::node::ports::connect(s, data.node_id, data.port_id, id)
            }) {
                Ok(()) => {
                    requests.channel_edges.request();
                    info!(
                        "connected channel {id} and node (id: {}, port_id: {})",
                        data.node_id, data.port_id
                    );
                }
                Err(err) => {
                    error!("{err}");
                    errors.with_mut(|q| q.push("channel-connect", err.to_string()));
                }
            }
        }
        Ok(())
    };

    let on_context_menu = move |(channel_id, evt): (ChannelId, Event<MouseData>)| {
        evt.prevent_default();
        evt.stop_propagation();
        let click_point = Point {
            x: evt.element_coordinates().x,
            y: evt.element_coordinates().y,
        };

        menus.channel.set(channel::ContextMenuState::Visible {
            position: click_point,
            channel_id,
        });
        Ok(())
    };

    let on_mouse_down = move |(id, position, evt): (ChannelId, Point, Event<MouseData>)| {
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
            drag.channel.set(DragChannelState::Dragged { id, offset });
        }
        Ok(())
    };

    channel::Handlers::new(
        receiver_on_mouse_up,
        sender_on_mouse_up,
        on_context_menu,
        on_mouse_down,
    )
}

pub(crate) fn context_menu_handlers(
    mut menus: MenuState,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> channel::ContextMenuHandlers {
    let on_delete = move |id: ChannelId| -> Result<()> {
        backend.with_mut(|s| beetry_editor_backend::api::channel::remove(s, id))?;
        requests.channels.request();
        requests.channel_edges.request();
        Ok(())
    };

    let on_close = move |_| {
        menus.channel.set(channel::ContextMenuState::Idle);
        Ok(())
    };

    channel::ContextMenuHandlers::new(on_delete, on_close)
}
