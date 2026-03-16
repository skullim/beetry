use dioxus::prelude::*;

mod context;
mod handlers;

use crate::{
    components::editor::state::{ReloadWorkspaceFlag, svg::DimensionState},
    ui::{error, transfer},
};

#[component]
pub(crate) fn Toolbar(dimensions: DimensionState, reload_ws: ReloadWorkspaceFlag) -> Element {
    rsx! {
        context::Provider { dimensions, reload_ws, Layout {} }
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
