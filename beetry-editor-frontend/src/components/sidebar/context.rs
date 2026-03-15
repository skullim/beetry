use dioxus::prelude::*;

use super::handlers;
use crate::{Backend, SharedSpecs, components::editor, signals::RenderRequests};

#[component]
pub(super) fn Provider(editor_state: editor::State, children: Element) -> Element {
    let specs = use_context::<SharedSpecs>();
    let backend = use_context::<Backend>();
    let render_requests = use_context::<RenderRequests>();

    use_context_provider(move || {
        handlers::handlers(
            specs,
            render_requests.nodes,
            backend,
            editor_state,
            editor_state.default_channel_config,
        )
    });

    children
}
