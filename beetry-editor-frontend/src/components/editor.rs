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

    let workspace = if state.reload_ws.read_val() {
        // Force here to drop old Workspace and rerender again.
        // Rerender will execute else branch as we are clearing the reload flag
        state.reload_ws.clear();
        rsx! {}
    } else {
        rsx! {Workspace { render_requests, editor_state: state  }}
    };

    rsx! {
        GlobalStyle {}
        div { class: "bt-editor-shell",
            div { class: "bt-editor-grid",
                div { class: "bt-panel",
                    Sidebar { editor_state: state }
                }
                div { class: "bt-panel bt-workspace-shell",
                    Topbar {}
                    {workspace}
                }
                div { class: "bt-panel",
                    Toolbar { dimensions: state.svg.dimensions, reload_ws: state.reload_ws }
                }
            }
        }
    }
}
