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
        transfer::ExportProject {}
        transfer::ExportValidTree {}
        transfer::Import {}
        error::Dialog {}
    }
}
