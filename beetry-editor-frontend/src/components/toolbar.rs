use dioxus::prelude::*;

mod context;
mod handlers;

use crate::{
    components::editor::state::svg::DimensionState,
    ui::{error, transfer},
};

#[component]
pub(crate) fn Toolbar(dimensions: DimensionState) -> Element {
    rsx! {
        context::Provider { dimensions, Layout {} }
    }
}

#[component]
fn Layout() -> Element {
    rsx! {
        div { class: "bt-toolbar-actions",
            p { class: "bt-panel-title", "Toolbar" }
            transfer::ExportProject {}
            transfer::ExportValidTree {}
            transfer::Import {}
        }
        error::Dialog {}
    }
}
