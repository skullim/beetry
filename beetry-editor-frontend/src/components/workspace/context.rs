use dioxus::prelude::*;

use super::handlers;
use crate::ui::node;
use crate::{components::workspace, signals::RenderRequests};

#[component]
pub(super) fn Provider(
    state: workspace::State,
    render_requests: RenderRequests,
    element_spawn_point: Signal<crate::Point>,
    parameter_state: Signal<node::parameter::State>,
    children: Element,
) -> Element {
    let backend = use_context();
    use_context_provider(|| {
        handlers::handlers(state, element_spawn_point, backend, render_requests)
    });

    let error_queue_state = use_context();
    use_context_provider(|| {
        handlers::node::handlers(
            state.drag,
            state.menu,
            state.svg,
            backend,
            error_queue_state,
        )
    });
    use_context_provider(|| handlers::port::input_handlers(state.temp, backend, render_requests));
    use_context_provider(|| handlers::port::output_handlers(state.temp));
    use_context_provider(|| {
        handlers::node::menu_handlers(state.menu, backend, render_requests, parameter_state)
    });

    use_context_provider(|| handlers::edge::handlers(state.menu));
    use_context_provider(|| handlers::edge::menu_handlers(state.menu, backend, render_requests));
    use_context_provider(|| handlers::channel::menu_handlers(state.menu, backend, render_requests));

    use_context_provider(|| {
        handlers::port::sender_handlers(state.menu, state.temp, backend, error_queue_state)
    });
    use_context_provider(|| {
        handlers::port::receiver_handlers(state.menu, state.temp, backend, error_queue_state)
    });
    use_context_provider(|| handlers::port::menu_handlers(backend, render_requests));

    use_context_provider(|| {
        handlers::channel::handlers(
            state,
            backend,
            render_requests,
            error_queue_state,
        )
    });

    children
}
