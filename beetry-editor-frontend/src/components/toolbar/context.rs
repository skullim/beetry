use dioxus::prelude::*;

use super::handlers;
use crate::Backend;
use crate::signals::RenderRequests;
use crate::ui::error::ErrorQueueState;

#[component]
pub(super) fn Provider(children: Element) -> Element {
    let error_queue = use_context::<ErrorQueueState>();
    let backend = use_context::<Backend>();
    let render_requests = use_context::<RenderRequests>();

    use_context_provider(|| handlers::export_handlers(error_queue, backend));
    use_context_provider(|| handlers::import_handlers(error_queue, backend, render_requests));

    children
}
