use dioxus::prelude::*;

use crate::toolbar::ToolbarHandlers;

#[derive(Debug, Clone)]
pub struct Handlers {
    on_click: EventHandler<()>,
}

impl Handlers {
    pub(crate) fn new(on_click: impl FnMut(()) + 'static) -> Self {
        Self {
            on_click: EventHandler::new(on_click),
        }
    }
}

#[component]
pub fn Import() -> Element {
    rsx! {
        button { onclick: move |_| { use_context::<ToolbarHandlers>().import.on_click.call(()) },
            "Import project"
        }
    }
}
