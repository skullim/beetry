use dioxus::prelude::*;

use crate::toolbar::ToolbarHandlers;
use crate::ui::transfer::{OperationResult, OperationStatus};

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
pub fn Import(result: ReadSignal<OperationResult>) -> Element {
    rsx! {
        button { onclick: move |_| { use_context::<ToolbarHandlers>().import.on_click.call(()) },
            "Import"
        }

        match result.read().status {
            OperationStatus::None => {
                rsx! {}
            }
            OperationStatus::Success => {
                rsx! {
                    div { style: {"color: green; margin-top: 5px; font-size: 12px;"}, {result.peek().message.clone()} }
                }
            }
            OperationStatus::Error => {
                rsx! {
                    div { style: {"color: red; margin-top: 5px; font-size: 12px;"}, {result.peek().message.clone()} }
                }
            }
        }
    }
}
