mod context;
mod handlers;
pub mod state;

use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::components::sidebar::Sidebar;
use crate::components::toolbar::Toolbar;
use crate::components::topbar::Topbar;
use crate::components::workspace::Workspace;
use crate::signals::RenderRequests;
use crate::ui::theme::GlobalStyle;

pub use state::State;

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
                    Topbar {}
                    Workspace { render_requests, editor_state: state }
                }
                div { class: "bt-panel", Toolbar { dimensions: state.svg.dimensions } }
            }
        }
    }
}
