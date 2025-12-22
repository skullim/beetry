mod definitions;
mod domain;
mod editor;
mod id;
mod project;
mod sidebar;
mod signals;
mod toolbar;
mod ui;
mod workspace;

use anyhow::{Result, anyhow};
use beetry_editor_types::{NodeSpec, NodeSpecKey};
use beetry_plugin::{
    channel::ChannelPluginConstructor,
    node::{ActionPluginConstructor, ConditionPluginConstructor, ControlPluginConstructor},
};
use beetry_plugin_types::{
    channel::ChannelSpec,
    node::{ControlSpec, LeafSpec},
};
use dioxus::logger::tracing::Level;
use dioxus::prelude::*;
pub use domain::service::editor::EditorService;
use std::collections::HashMap;

pub use project::ProjectData;

#[cfg(target_family = "wasm")]
unsafe extern "C" {
    fn __wasm_call_ctors();
}

pub fn launch() {
    #[cfg(target_family = "wasm")]
    unsafe {
        use dioxus::logger::tracing::info;
        info!("running wasm ctor");
        __wasm_call_ctors();
    }
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");
    dioxus::launch(app);
}

#[component]
fn app() -> Element {
    rsx! {
        PluginsProvider { editor::Editor {} }
    }
}

#[derive(Clone, PartialEq)]
pub struct SpecPlugins {
    leaves: Vec<LeafSpec>,
    controls: Vec<ControlSpec>,
    channels: Vec<ChannelSpec>,
}

#[derive(Clone)]
pub struct SpecPlugins2 {
    nodes: HashMap<NodeSpecKey, NodeSpec>,
    pub channels: Vec<ChannelSpec>,
}

impl SpecPlugins2 {
    pub fn get_node_spec(&self, key: &NodeSpecKey) -> Result<&NodeSpec> {
        self.nodes
            .get(key)
            .ok_or_else(|| anyhow!("failed to obtain node spec for key {key:?}"))
    }
}

#[component]
pub fn PluginsProvider(children: Element) -> Element {
    let leaves = {
        let action_plugins = ActionPluginConstructor::plugins()?;
        let condition_plugins = ConditionPluginConstructor::plugins()?;

        let mut plugins = action_plugins
            .into_iter()
            .map(|p| p.into_parts().0)
            .collect::<Vec<_>>();
        plugins.extend(condition_plugins.into_iter().map(|p| p.into_parts().0));
        plugins
    };

    let controls = ControlPluginConstructor::plugins()?
        .into_iter()
        .map(|p| p.into_parts().0)
        .collect();

    let channels = {
        let plugins = ChannelPluginConstructor::plugins()?;
        plugins.into_iter().map(|p| p.into_parts().0).collect()
    };

    use_context_provider(move || SpecPlugins {
        leaves,
        controls,
        channels,
    });
    children
}
