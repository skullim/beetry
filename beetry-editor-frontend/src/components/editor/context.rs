use dioxus::prelude::*;

use super::handlers;
use crate::components::editor;
use crate::signals::RenderRequests;
use crate::ui::error::ErrorQueueState;
use crate::{Backend, SharedSpecs};

#[component]
pub(super) fn Provider(state: editor::State, children: Element) -> Element {
    let specs = use_context::<SharedSpecs>();
    let backend = use_context_provider(|| Backend::new(specs.nodes.clone()));

    let render_requests = use_context_provider(RenderRequests::default);
    use_context_provider(ErrorQueueState::new);

    use_context_provider({
        let specs = specs.clone();
        move || handlers::channel_dialog::handlers(specs, render_requests.channels, backend, state)
    });
    use_context_provider(|| {
        handlers::parameter::handlers(backend, state.parameter, render_requests.nodes)
    });

    children
}
