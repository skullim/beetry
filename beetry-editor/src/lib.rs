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
use beetry_core::MessageHash;
use beetry_editor_types::{NodeSpec, NodeSpecKey};
use beetry_plugin::{
    channel::{ChannelPluginConstructor, ChannelPluginConstructor2},
    node::{
        ActionPluginConstructor, ActionPluginConstructor2, ConditionPluginConstructor,
        ConditionPluginConstructor2, ControlPluginConstructor, ControlPluginConstructor2,
    },
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
    dioxus::launch(app2);
}

#[component]
fn app() -> Element {
    rsx! {
        PluginsProvider { editor::Editor {} }
    }
}

#[component]
fn app2() -> Element {
    rsx! {
        Specs2Provider { editor::Editor2 {} }
    }
}

#[derive(Clone, PartialEq)]
pub struct SpecPlugins {
    leaves: Vec<LeafSpec>,
    controls: Vec<ControlSpec>,
    channels: Vec<ChannelSpec>,
}

#[derive(Clone)]
pub struct Specs2 {
    pub nodes: NodeSpecMap,
    pub channels: ChannelSpecMap,
}

#[derive(Clone)]
pub struct NodeSpecMap {
    map: HashMap<NodeSpecKey, NodeSpec>,
}

impl NodeSpecMap {
    pub fn spec(&self, key: &NodeSpecKey) -> Result<&NodeSpec> {
        self.map
            .get(key)
            .ok_or_else(|| anyhow!("failed to obtain node spec for key {key:?}"))
    }

    pub fn values(&self) -> impl Iterator<Item = &NodeSpec> {
        self.map.values()
    }
}

#[derive(Clone)]
pub struct ChannelSpecMap {
    map: HashMap<MessageHash, ChannelSpec>,
}

impl ChannelSpecMap {
    pub fn spec(&self, key: &MessageHash) -> Result<&ChannelSpec> {
        self.map
            .get(key)
            .ok_or_else(|| anyhow!("failed to obtain channel spec for key {key:?}"))
    }

    pub fn values(&self) -> impl Iterator<Item = &ChannelSpec> {
        self.map.values()
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

#[component]
pub fn Specs2Provider(children: Element) -> Element {
    let nodes: NodeSpecMap = {
        let action_plugins = ActionPluginConstructor2::plugins()?;
        let condition_plugins = ConditionPluginConstructor2::plugins()?;
        let control_plugins = ControlPluginConstructor2::plugins()?;

        let mut map = action_plugins
            .into_iter()
            .map(|p| {
                let spec = p.into_parts().0;
                (spec.key().clone(), spec)
            })
            .collect::<HashMap<_, _>>();

        map.extend(condition_plugins.into_iter().map(|p| {
            let spec = p.into_parts().0;
            (spec.key().clone(), spec)
        }));
        map.extend(control_plugins.into_iter().map(|p| {
            let spec = p.into_parts().0;
            (spec.key().clone(), spec)
        }));
        NodeSpecMap { map }
    };

    let channels = {
        let plugins = ChannelPluginConstructor2::plugins()?;
        let map = plugins
            .into_iter()
            .map(|p| {
                let spec = p.into_parts().0;
                (spec.msg_hash(), spec)
            })
            .collect::<HashMap<_, _>>();
        ChannelSpecMap { map }
    };

    use_context_provider(move || Specs2 { nodes, channels });
    children
}
