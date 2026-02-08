use dioxus::prelude::*;

use crate::toolbar::ToolbarHandlers;

#[derive(Debug, Clone)]
pub struct Handlers {
    on_project: EventHandler<()>,
    on_valid_tree: EventHandler<()>,
}

impl Handlers {
    pub(crate) fn new(
        on_project: impl FnMut(()) + 'static,
        on_valid_tree: impl FnMut(()) + 'static,
    ) -> Self {
        Self {
            on_project: EventHandler::new(on_project),
            on_valid_tree: EventHandler::new(on_valid_tree),
        }
    }
}

#[component]
pub fn ExportProject() -> Element {
    rsx! {
        button { onclick: move |_| { use_context::<ToolbarHandlers>().export.on_project.call(()) },
            "Export project"
        }
    }
}

#[component]
pub fn ExportValidTree() -> Element {
    rsx! {
        button { onclick: move |_| { use_context::<ToolbarHandlers>().export.on_valid_tree.call(()) },
            "Export valid tree"
        }
    }
}
