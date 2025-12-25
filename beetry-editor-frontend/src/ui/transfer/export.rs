use dioxus::prelude::*;

use crate::toolbar::ToolbarHandlers;
use crate::ui::transfer::{OperationResult, OperationStatus};

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
pub fn ExportProject(result: ReadSignal<OperationResult>) -> Element {
    rsx! {
        button { onclick: move |_| { use_context::<ToolbarHandlers>().export.on_project.call(()) },
            "Export project"
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

#[component]
pub fn ExportValidTree(result: ReadSignal<OperationResult>) -> Element {
    rsx! {
        button { onclick: move |_| { use_context::<ToolbarHandlers>().export.on_valid_tree.call(()) },
            "Export valid tree"
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
