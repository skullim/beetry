mod components;
mod definitions;
mod editor;
mod sidebar;
mod signals;
mod toolbar;
mod ui;

use std::rc::Rc;

use beetry_editor_backend::{ChannelSpecMap, NodeSpecMap};
use beetry_editor_types::{output::ui::Point, spec::node::NodeSpec};
use beetry_plugin::{
    channel::ChannelPluginConstructor,
    node::{ActionPluginConstructor, ConditionPluginConstructor, ControlPluginConstructor},
};
use dioxus::prelude::*;
use dioxus::{desktop::WindowBuilder, logger::tracing::Level};

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
    let cfg = dioxus::desktop::Config::default().with_window(
        WindowBuilder::new()
            .with_always_on_top(false)
            .with_title("Beetry Editor 🌳"),
    );
    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(app);
}

#[component]
fn app() -> Element {
    rsx! {
        SpecsProvider { editor::Editor {} }
    }
}

#[derive(Debug, Clone)]
pub struct Specs {
    pub nodes: NodeSpecMap,
    pub channels: ChannelSpecMap,
}

#[derive(Debug, Clone)]
pub struct SharedSpecs {
    pub specs: Rc<Specs>,
}

impl std::ops::Deref for SharedSpecs {
    type Target = Specs;
    fn deref(&self) -> &Self::Target {
        &self.specs
    }
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
        let root_spec = NodeSpec::root();
        let iter = iter.chain(std::iter::once((root_spec.key().clone(), root_spec)));
        NodeSpecMap::from_iter(iter)
    };

    let channels = {
        let plugins = ChannelPluginConstructor::plugins()?;
        plugins
            .into_iter()
            .map(|p| {
                let spec = p.into_parts().0;
                (spec.msg_hash(), spec)
            })
            .collect::<ChannelSpecMap>()
    };

    use_context_provider(move || SharedSpecs {
        specs: Rc::new(Specs { nodes, channels }),
    });
    children
}
