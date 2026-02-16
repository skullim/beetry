use dioxus::prelude::*;

use crate::specs;

#[component]
pub(super) fn Provider(children: Element) -> Element {
    let specs = specs::load_shared_specs()?;
    use_context_provider(|| specs);
    children
}
