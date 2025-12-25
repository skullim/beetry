mod definitions;
mod editor;
mod sidebar;
mod signals;
mod toolbar;
mod ui;
mod workspace;

use beetry_editor_backend::{ChannelSpecMap, NodeSpecMap};
use beetry_plugin::{
    channel::ChannelPluginConstructor,
    node::{ActionPluginConstructor, ConditionPluginConstructor, ControlPluginConstructor},
};
use dioxus::logger::tracing::Level;
use dioxus::prelude::*;
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
    dioxus::launch(app);
}

#[component]
fn app() -> Element {
    rsx! {
        SpecsProvider { editor::Editor {} }
    }
}

#[derive(Clone)]
pub struct Specs {
    pub nodes: NodeSpecMap,
    pub channels: ChannelSpecMap,
}

#[component]
pub fn SpecsProvider(children: Element) -> Element {
    let nodes: NodeSpecMap = {
        let action_plugins = ActionPluginConstructor::plugins()?;
        let condition_plugins = ConditionPluginConstructor::plugins()?;
        let control_plugins = ControlPluginConstructor::plugins()?;

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
        NodeSpecMap::from_iter(iter)
    };

    let channels = {
        let plugins = ChannelPluginConstructor::plugins()?;
        let map = plugins
            .into_iter()
            .map(|p| {
                let spec = p.into_parts().0;
                (spec.msg_hash(), spec)
            })
            .collect::<HashMap<_, _>>();
        ChannelSpecMap::new(map)
    };

    use_context_provider(move || Specs { nodes, channels });
    children
}
