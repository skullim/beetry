mod definitions;
mod editor;
mod project;
mod sidebar;
mod toolbar;
mod ui;
mod workspace;

use beetry_plugin::node::{
    ActionPluginConstructor, ConditionPluginConstructor, ControlPluginConstructor,
};
use beetry_serde::ser::{
    channel::ChannelSpec,
    node::{ControlNodeSpec, LeafNodeSpec},
};
use dioxus::{logger::tracing::Level, prelude::*};

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
struct Plugins {
    leaves: Vec<LeafNodeSpec>,
    controls: Vec<ControlNodeSpec>,
    channels: Vec<ChannelSpec>,
}

#[component]
pub fn PluginsProvider(children: Element) -> Element {
    let leaves = {
        let action_plugins = ActionPluginConstructor::plugins()?;
        let condition_plugins = ConditionPluginConstructor::plugins()?;

        let mut plugins = action_plugins
            .into_iter()
            .map(|p| p.spec())
            .collect::<Vec<_>>();
        plugins.extend(condition_plugins.into_iter().map(|p| p.spec()));
        plugins
    };

    let controls = ControlPluginConstructor::plugins()?
        .into_iter()
        .map(|p| p.spec())
        .collect();

    let channels = {
        let plugins = beetry_plugin::channel::plugins();
        plugins.into_iter().map(|p| p.spec()).collect()
    };

    use_context_provider(move || Plugins {
        leaves,
        controls,
        channels,
    });
    children
}
