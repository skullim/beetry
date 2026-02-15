use beetry_editor_types::{id::ChannelId, output::ui::Point};
use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::components::workspace::state::{drag, menu, svg, temporary};
use crate::ui::channel;
use crate::ui::error::ErrorQueueState;
use crate::ui::node::port::ConnectionOrigin;

use super::{Backend, DragChannelState, RenderRequests};

pub(crate) fn handlers(
    mut drag: drag::State,
    mut menus: menu::State,
    mut temp: temporary::State,
    svg: svg::State,
    mut backend: Backend,
    mut requests: RenderRequests,
    mut errors: ErrorQueueState,
) -> channel::Handlers {
    let receiver_on_mouse_up = move |id: ChannelId| -> Result<()> {
        let mut channel = temp.channel.write();
        if let Some(data) = channel.take_dragged()
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
        let mut channel = temp.channel.write();
        if let Some(data) = channel.take_dragged()
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

    let on_menu = move |(channel_id, evt): (ChannelId, Event<MouseData>)| {
        evt.prevent_default();
        evt.stop_propagation();
        let click_point = Point {
            x: evt.element_coordinates().x,
            y: evt.element_coordinates().y,
        };

        menus.channel.set(channel::menu::State::Visible {
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
        on_menu,
        on_mouse_down,
    )
}

pub(crate) fn menu_handlers(
    mut menu: menu::State,
    mut backend: Backend,
    mut requests: RenderRequests,
) -> channel::menu::Handlers {
    let on_delete = move |id: ChannelId| -> Result<()> {
        backend.with_mut(|s| beetry_editor_backend::api::channel::remove(s, id))?;
        requests.channels.request();
        requests.channel_edges.request();
        Ok(())
    };

    let on_close = move |_| {
        menu.channel.set(channel::menu::State::Idle);
        Ok(())
    };

    channel::menu::Handlers::new(on_delete, on_close)
}
