use dioxus::prelude::*;

use super::{handlers, state};
use crate::ui::error::ErrorQueueState;
use crate::{Backend, SharedSpecs};

#[component]
pub(super) fn EditorContextProvider(children: Element) -> Element {
    let specs = use_context::<SharedSpecs>();

    let backend = use_context_provider(|| Backend::new(specs.nodes.clone()));

    let state = use_context_provider(state::State::new);
    use_context_provider(|| state.render_requests);
    use_context_provider(ErrorQueueState::new);

    use_context_provider({
        let specs = specs.clone();
        move || handlers::channel_config::handlers(specs, backend, state)
    });
    use_context_provider({
        let specs = specs.clone();
        move || handlers::sidebar::handlers(specs, backend, state)
    });
    use_context_provider(|| {
        handlers::parameter::handlers(backend, state.parameter, state.render_requests.nodes)
    });

    children
}
