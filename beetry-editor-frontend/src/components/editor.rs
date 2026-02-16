mod context_provider;
mod handlers;
mod state;

use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::components::workspace::Workspace;
use crate::sidebar::Sidebar;
use crate::ui::node;
use crate::toolbar::Toolbar;
use context_provider::EditorContextProvider;

#[component]
pub(crate) fn Editor() -> Element {
    rsx! {
        EditorContextProvider { EditorLayout {} }
    }
}

#[component]
fn EditorLayout() -> Element {
    debug!("rendering");

    let state = use_context::<state::State>();

    rsx! {
        div { style: "display: flex; flex-direction: row; gap: 10px;",
            div { style: "flex: 0 1 20%;",
                Sidebar { channel_config_state: state.channel_config }
            }
            div { style: "flex: 0 1 80%;",
                Workspace {
                    render_requests: state.render_requests,
                    element_spawn_point: state.element_spawn_point,
                    parameter_state: state.parameter,
                }
            }
            div {
                node::parameter::Dialog { state: state.parameter }
            }
            div { style: "flex: 0 1 10%;",
                Toolbar { render_requests: state.render_requests }
            }
        }
    }
}
