mod context;
mod handlers;
mod state;

use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::components::workspace::Workspace;
use crate::sidebar::Sidebar;
use crate::signals::RenderRequests;
use crate::toolbar::Toolbar;
use crate::ui::node;

pub(crate) use state::State;

#[component]
pub(crate) fn Editor() -> Element {
    let state = State::new();
    rsx! {
        context::Provider {state, Layout {state} }
    }
}

#[component]
fn Layout(state: State) -> Element {
    debug!("rendering");

    let render_requests = use_context::<RenderRequests>();

    rsx! {
        div { style: "display: flex; flex-direction: row; gap: 10px;",
            div { style: "flex: 0 1 20%;",
                Sidebar { channel_config_state: state.channel_config }
            }
            div { style: "flex: 0 1 80%;",
                Workspace {
                    render_requests,
                    element_spawn_point: state.element_spawn_point,
                    parameter_state: state.parameter,
                }
            }
            div {
                node::parameter::Dialog { state: state.parameter }
            }
            div { style: "flex: 0 1 10%;",
                Toolbar { render_requests }
            }
        }
    }
}
