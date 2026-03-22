use std::rc::Rc;

use beetry_editor_types::spec::{
    channel::ChannelSpecMap,
    node::{NodeSpec, NodeSpecMap},
};
use beetry_plugin::{
    ChannelPluginConstructor,
    node::{
        ActionPluginConstructor, ConditionPluginConstructor, ControlPluginConstructor,
        DecoratorPluginConstructor,
    },
};
use dioxus::prelude::*;

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

pub fn load_shared_specs() -> Result<SharedSpecs> {
    Ok(SharedSpecs {
        specs: Rc::new(Specs {
            nodes: load_node_specs()?,
            channels: load_channel_specs()?,
        }),
    })
}

fn load_node_specs() -> Result<NodeSpecMap> {
    let action_plugins = ActionPluginConstructor::plugins()?;
    let condition_plugins = ConditionPluginConstructor::plugins()?;
    let control_plugins = ControlPluginConstructor::plugins()?;
    let decorator_plugins = DecoratorPluginConstructor::plugins()?;

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
    let iter = iter.chain(decorator_plugins.into_iter().map(|p| {
        let spec = p.into_parts().0;
        (spec.key().clone(), spec)
    }));
    let root_spec = NodeSpec::root();
    let iter = iter.chain(std::iter::once((root_spec.key().clone(), root_spec)));
    Ok(iter.collect())
}

fn load_channel_specs() -> Result<ChannelSpecMap> {
    let plugins = ChannelPluginConstructor::plugins()?;
    Ok(plugins
        .into_iter()
        .map(|p| {
            let spec = p.into_parts().0;
            (spec.msg_hash(), spec)
        })
        .collect::<ChannelSpecMap>())
}
