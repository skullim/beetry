use beetry_editor_backend::api::ChannelQueryView;
use beetry_editor_types::{id::ChannelId, output::ui::Point};
use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::components::workspace::{self, state::menu};
use crate::ui::channel;
use crate::ui::channel::config::DEFAULT_DIALOG_POSITION;
use crate::ui::error::ErrorQueueState;
use crate::ui::node::port::ConnectionOrigin;

use super::{Backend, DragChannelState, RenderRequests};

pub(crate) fn handlers(
    mut state: workspace::State,
    mut backend: Backend,
    mut requests: RenderRequests,
    mut errors: ErrorQueueState,
) -> channel::Handlers {
    let receiver_on_mouse_up = move |id: ChannelId| -> Result<()> {
        let mut channel = state.temp.channel.write();
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
                Err(e) => {
                    errors.push(e);
                }
            }
        }
        Ok(())
    };

    let sender_on_mouse_up = move |id: ChannelId| -> Result<()> {
        let mut channel = state.temp.channel.write();
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
                Err(e) => {
                    errors.push(e);
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

        state.menu.channel.set(channel::menu::State::Visible {
            position: click_point,
            channel_id,
        });
        Ok(())
    };

    let on_mouse_down = move |(id, position, evt): (ChannelId, Point, Event<MouseData>)| {
        if evt.held_buttons().contains(MouseButton::Primary) {
            let mouse_coords = evt.client_coordinates();
            let zoom_level = state.svg.zoom.get();

            let svg_mouse_coords = Point {
                x: mouse_coords.x / zoom_level,
                y: mouse_coords.y / zoom_level,
            };

            let offset = Point {
                x: svg_mouse_coords.x - position.x,
                y: svg_mouse_coords.y - position.y,
            };
            state
                .drag
                .channel
                .set(DragChannelState::Dragged { id, offset });
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
    mut dialog_state: Signal<channel::config::State>,
    mut errors: ErrorQueueState,
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

    let on_update = move |id: ChannelId| -> Result<()> {
        let position = match *menu.channel.peek() {
            channel::menu::State::Visible { position, .. } => position,
            channel::menu::State::Idle => DEFAULT_DIALOG_POSITION,
        };

        let Some(config) = backend.with(|s| {
            let query = beetry_editor_backend::api::channel::borrow(s);
            query.config(id).map_err(|e| errors.push(e)).ok().cloned()
        }) else {
            return Ok(());
        };

        dialog_state.set(channel::config::State::Visible {
            position,
            mode: channel::config::Mode::Update { channel_id: id },
            config: CopyValue::new(config),
        });
        menu.channel.set(channel::menu::State::Idle);
        Ok(())
    };

    channel::menu::Handlers::new(on_delete, on_update, on_close)
}
