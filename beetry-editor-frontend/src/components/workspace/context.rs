use dioxus::prelude::*;

use super::handlers;
use crate::components::editor;
use crate::{components::workspace, signals::RenderRequests};

#[component]
pub(super) fn Provider(
    state: workspace::State,
    editor_state: editor::State,
    render_requests: RenderRequests,
    children: Element,
) -> Element {
    let backend = use_context();
    use_context_provider(|| {
        handlers::handlers(
            &state,
            editor_state.element_spawn_point,
            backend,
            render_requests,
        )
    });

    let errors = use_context();
    use_context_provider(|| {
        handlers::node::handlers(state.drag, state.menu, state.svg, backend, errors)
    });
    use_context_provider(|| handlers::pin::input_handlers(state.temp, backend, render_requests));
    use_context_provider(|| handlers::pin::output_handlers(state.temp));
    use_context_provider(|| {
        handlers::node::menu_handlers(state.menu, backend, render_requests, editor_state.parameter)
    });

    use_context_provider(|| handlers::edge::handlers(state.menu));
    use_context_provider(|| handlers::edge::menu_handlers(state.menu, backend, render_requests));
    use_context_provider(|| handlers::channel_edge::handlers(state.menu));
    use_context_provider(|| {
        handlers::channel_edge::menu_handlers(state.menu, backend, render_requests)
    });
    use_context_provider(|| {
        handlers::channel::menu_handlers(
            state.menu,
            backend,
            render_requests,
            editor_state.channel_dialog_state,
            errors,
        )
    });

    use_context_provider(|| handlers::pin::body_handlers(state.menu, state.temp));
    use_context_provider(|| handlers::pin::menu_handlers(backend, errors));

    use_context_provider(|| handlers::channel::handlers(state, backend, render_requests, errors));

    children
}
