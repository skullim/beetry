mod definitions;
mod domain;
mod editor;
mod id;
mod sidebar;
mod signals;
mod toolbar;
mod ui;
mod workspace;

use anyhow::{Result, anyhow};
use beetry_core::MessageHash;
use beetry_editor_types::{NodeSpec, NodeSpecKey};
use beetry_plugin::{
    channel::ChannelPluginConstructor2,
    node::{ActionPluginConstructor2, ConditionPluginConstructor2, ControlPluginConstructor2},
};
use beetry_plugin_types::channel::ChannelSpec;
use dioxus::logger::tracing::Level;
use dioxus::prelude::*;
pub use domain::service::editor::EditorService;
use std::collections::HashMap;

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
fn app2() -> Element {
    rsx! {
        Specs2Provider { editor::Editor2 {} }
    }
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
    fn new(iter: impl Iterator<Item = (NodeSpecKey, NodeSpec)>) -> Self {
        let mut map: HashMap<_, _> = iter.into_iter().collect();
        map.insert(NodeSpecKey::root(), NodeSpec::root());
        Self { map }
    }

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
pub fn Specs2Provider(children: Element) -> Element {
    let nodes: NodeSpecMap = {
        let action_plugins = ActionPluginConstructor2::plugins()?;
        let condition_plugins = ConditionPluginConstructor2::plugins()?;
        let control_plugins = ControlPluginConstructor2::plugins()?;

        let iter = action_plugins.into_iter().map(|p| {
            let spec = p.into_parts().0;
            (spec.key().clone(), spec)
        });
        let iter = iter.chain(condition_plugins.into_iter().map(|p| {
            let spec = p.into_parts().0;
            (spec.key().clone(), spec)
        }));

        let iter = iter.chain(control_plugins.into_iter().map(|p| {
            let spec = p.into_parts().0;
            (spec.key().clone(), spec)
        }));
        NodeSpecMap::new(iter)
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
