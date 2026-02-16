mod context;
mod handlers;
mod state;

use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::components::sidebar::Sidebar;
use crate::components::toolbar::Toolbar;
use crate::components::workspace::Workspace;
use crate::signals::RenderRequests;
use crate::ui::theme::GlobalStyle;

pub(crate) use state::State;

#[component]
pub(crate) fn Editor() -> Element {
    let state = State::new();
    rsx! {
        context::Provider { state,
            Layout { state }
        }
    }
}

#[component]
fn Layout(state: State) -> Element {
    debug!("rendering");

    let render_requests = use_context::<RenderRequests>();

    rsx! {
        GlobalStyle {}
        div { class: "bt-editor-shell",
            div { class: "bt-editor-grid",
                div { class: "bt-panel",
                Sidebar { editor_state: state }
                }
                div { class: "bt-panel bt-workspace-shell",
                    div { class: "bt-workspace-header",
                        span { "Workspace" }
                        span { "Ctrl + Wheel to zoom" }
                    }
                Workspace { render_requests, editor_state: state }
                }
                div { class: "bt-panel", Toolbar {} }
            }
        }
    }
}
