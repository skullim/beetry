mod context;
mod handlers;
pub mod state;

use dioxus::prelude::*;
pub use state::State;

use crate::{
    components::{sidebar::Sidebar, toolbar::Toolbar, topbar::Topbar, workspace::Workspace},
    signals::RenderRequests,
    ui::theme::GlobalStyle,
};

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
                div { class: "bt-panel",
                    Toolbar { dimensions: state.svg.dimensions }
                }
            }
        }
    }
}
