use dioxus::prelude::*;

use super::handlers;
use crate::ui::node;
use crate::{components::workspace, signals::RenderRequests};

#[component]
pub(super) fn WorkspaceContextProvider(
    render_requests: RenderRequests,
    element_spawn_point: Signal<crate::Point>,
    parameter_state: Signal<node::parameter::State>,
    children: Element,
) -> Element {
    let backend = use_context();
    let workspace_state = use_context_provider(workspace::State::new);
    use_context_provider(|| {
        handlers::handlers(
            workspace_state,
            element_spawn_point,
            backend,
            render_requests,
        )
    });

    let error_queue_state = use_context();
    use_context_provider(|| {
        handlers::node::handlers(
            workspace_state.drag,
            workspace_state.menu,
            workspace_state.svg,
            backend,
            error_queue_state,
        )
    });
    use_context_provider(|| {
        handlers::port::input_handlers(workspace_state.temp, backend, render_requests)
    });
    use_context_provider(|| handlers::port::output_handlers(workspace_state.temp));
    use_context_provider(|| {
        handlers::node::menu_handlers(
            workspace_state.menu,
            backend,
            render_requests,
            parameter_state,
        )
    });

    use_context_provider(|| handlers::edge::handlers(workspace_state.menu));
    use_context_provider(|| {
        handlers::edge::menu_handlers(workspace_state.menu, backend, render_requests)
    });
    use_context_provider(|| {
        handlers::channel::menu_handlers(workspace_state.menu, backend, render_requests)
    });

    use_context_provider(|| {
        handlers::port::sender_handlers(
            workspace_state.menu,
            workspace_state.temp,
            backend,
            error_queue_state,
        )
    });
    use_context_provider(|| {
        handlers::port::receiver_handlers(
            workspace_state.menu,
            workspace_state.temp,
            backend,
            error_queue_state,
        )
    });
    use_context_provider(|| handlers::port::menu_handlers(backend, render_requests));

    use_context_provider(|| {
        handlers::channel::handlers(
            workspace_state.drag,
            workspace_state.menu,
            workspace_state.temp,
            workspace_state.svg,
            backend,
            render_requests,
            error_queue_state,
        )
    });

    children
}
