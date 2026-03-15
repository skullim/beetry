use dioxus::prelude::*;

use super::handlers;
use crate::{
    Backend, components::editor::state::svg::DimensionState, signals::RenderRequests,
    ui::error::ErrorQueueState,
};

#[component]
pub(super) fn Provider(dimensions: DimensionState, children: Element) -> Element {
    let error_queue = use_context::<ErrorQueueState>();
    let backend = use_context::<Backend>();
    let render_requests = use_context::<RenderRequests>();

    use_context_provider(|| handlers::export_handlers(error_queue, backend));
    use_context_provider(|| {
        handlers::import_handlers(dimensions, error_queue, backend, render_requests)
    });

    children
}
