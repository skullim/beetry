use beetry_editor_types::output::ui::ChannelUiData;
use dioxus::prelude::*;

use super::super::state;
use crate::{Backend, SharedSpecs, signals::RequestChannelRender, ui::channel};

pub(crate) fn handlers(
    specs: SharedSpecs,
    mut request: RequestChannelRender,
    mut backend: Backend,
    mut state: state::State,
) -> channel::dialog::Handlers {
    let on_confirm = move |action: channel::dialog::ConfirmAction| -> Result<()> {
        match action {
            channel::dialog::ConfirmAction::Create { spec_key, input } => {
                let spec = specs.channels.spec(&spec_key)?;
                let ui_data = state
                    .element_spawn_point
                    .with_peek(|p| ChannelUiData { position: *p });
                let id = backend.with_mut(|s| {
                    beetry_editor_backend::api::channel::create(s, spec, input, ui_data)
                })?;

                info!(
                    "created channel {id} with message type {}",
                    spec.msg_type_name()
                );
            }
            channel::dialog::ConfirmAction::Update { channel_id, update } => {
                backend.with_mut(|s| {
                    beetry_editor_backend::api::channel::update_config(s, channel_id, update)
                })?;
            }
        }
        state.channel_dialog_state.take();
        request.request();
        Ok(())
    };

    let on_cancel = move |_| {
        state.channel_dialog_state.take();
        Ok(())
    };

    channel::dialog::Handlers::new(on_confirm, on_cancel)
}
