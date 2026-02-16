use dioxus::prelude::*;

mod context;
mod handlers;

use crate::ui::error;
use crate::ui::transfer;

#[component]
pub(crate) fn Toolbar() -> Element {
    rsx! {
        context::Provider { Layout {} }
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
