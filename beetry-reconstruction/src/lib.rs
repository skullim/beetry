use anyhow::{Result, anyhow};
use beetry_node::NonEmptyNodes;
use std::collections::HashMap;
use tracing::debug;

use beetry_builder::Builder as BehaviorTreeBuilder;
use beetry_channel::external;
use beetry_core::{BehaviorTree, BoxNode, Node, RegisterTask, Root, TaskControl};
use beetry_node::{Fallback, Parallel, Sequence};

use beetry_definitions::{
    description::{LeafDescription, LeafKind, MessageHash, NodeHash},
    export::{ChannelId, ChannelIdToExportMap, ControlKind, NodeExport, TreeExport},
};
use beetry_plugin::{
    channel::{self, ChannelPlugin, TypeErasedChannel},
    node::{self, ActionFactory, ConditionNodePlugin, NodePlugin, NodeReconstructionData},
};

#[derive(Default)]
pub struct TreeReconstructor {
    external_receivers: external::ReceiverRegistry,
}

impl TreeReconstructor {
    pub fn new() -> Self {
        Self {
            external_receivers: external::ReceiverRegistry::new(),
        }
    }

    pub fn with_receiver_registry(external_receivers: external::ReceiverRegistry) -> Self {
        Self { external_receivers }
    }

    // Reconstruct criteria:
    // 1. Leaf nodes exist in node plugin registry.
    // 2. Channels exist in channel plugin registry.
    // 3. Each hash of leaf node matches with the corresponding node found in plugin registry.
    // 4. External receivers (if any) have been created when initializing Self instance
    pub fn try_reconstruct<R, T>(
        &mut self,
        export: TreeExport,
        builder: &BehaviorTreeBuilder<R, T>,
    ) -> Result<BehaviorTree<BoxNode>>
    where
        R: RegisterTask<T> + 'static,
        T: TaskControl + 'static,
    {
        let channel_factory_map = ChannelHashToFactoryMap::new(channel::plugins());
        let mut channels = Self::try_reconstruct_channels(export.channels, channel_factory_map)?;

        let action_factory_map =
            ActionHashToFactoryMap::new(node::ActionNodePluginConstructor::plugins());
        let condition_factory_map =
            ConditionHashToFactoryMap::new(node::ConditionNodePluginConstructor::plugins());
        let child = Self::try_reconstruct_tree(
            export.root.into_child(),
            &action_factory_map,
            &condition_factory_map,
            &mut channels,
            &mut self.external_receivers,
            builder,
        )?;
        Ok(BehaviorTree::new(Root::new(child)))
    }

    fn try_reconstruct_channels(
        export_map: ChannelIdToExportMap,
        factory_map: ChannelHashToFactoryMap,
    ) -> Result<ChannelIdToChannelMap> {
        export_map
            .into_iter()
            .map(|(k, v)| {
                let msg_hash = v.desc().msg_hash();
                debug!("{factory_map:?}");
                let factory = factory_map.get(msg_hash).ok_or_else(|| {
                    anyhow!(
                        "cannot create channel, did not find channel with required hash {msg_hash:?}"
                    )
                })?;
                Ok((k, factory.create(v.metadata().clone())))
            })
            .collect::<Result<_>>()
    }

    fn try_reconstruct_tree<R, T>(
        node: NodeExport,
        action_factory_map: &ActionHashToFactoryMap,
        condition_factory_map: &ConditionHashToFactoryMap,
        channel_map: &mut ChannelIdToChannelMap,
        receivers_registry: &mut external::ReceiverRegistry,
        builder: &BehaviorTreeBuilder<R, T>,
    ) -> Result<Box<dyn Node>>
    where
        R: RegisterTask<T> + 'static,
        T: TaskControl + 'static,
    {
        match node {
            NodeExport::Control(control) => {
                let control_kind = control.kind();
                let children: Vec<_> = control
                    .into_children_iter()
                    .into_iter()
                    .map(|child| {
                        Self::try_reconstruct_tree(
                            *child,
                            action_factory_map,
                            condition_factory_map,
                            channel_map,
                            receivers_registry,
                            builder,
                        )
                    })
                    .collect::<Result<_>>()?;
                let children = NonEmptyNodes::try_from(children)
                    .map_err(|_| anyhow!("wrong export, no children found for control node"))?;

                match control_kind {
                    ControlKind::Fallback => Ok(Box::new(Fallback::new(children))),
                    ControlKind::Sequence => Ok(Box::new(Sequence::new(children))),
                    ControlKind::Parallel => Ok(Box::new(Parallel::new(children))),
                }
            }
            NodeExport::Leaf(mut leaf) => {
                let mut receivers: Vec<_> = leaf
                    .take_receivers()
                    .into_iter()
                    .map(|id| Self::try_get_channel_mut(channel_map, &id)?.try_take_receiver())
                    .collect::<Result<_>>()?;

                if let Some(external_receivers_export) = leaf.take_external_receivers_export() {
                    external_receivers_export
                        .into_iter()
                        .map(|hash| receivers_registry.take(hash))
                        .for_each(|o_external_receiver| {
                            if let Some(external_receiver) = o_external_receiver {
                                receivers.push(external_receiver);
                            }
                        });
                }

                let senders: Vec<_> = leaf
                    .take_senders()
                    .into_iter()
                    .map(|id| Self::try_get_channel_mut(channel_map, &id)?.try_take_sender())
                    .collect::<Result<_>>()?;

                let data = NodeReconstructionData::builder()
                    .receivers(receivers)
                    .senders(senders)
                    .parameters(leaf.take_parameters())
                    .build();
                let leaf_hash = leaf.hash();
                match leaf.kind() {
                    LeafKind::Action => {
                        let factory = action_factory_map.get(&leaf_hash).ok_or_else(|| {
                            anyhow!(
                                "action factory of leaf with hash: {leaf_hash:?} does not exist"
                            )
                        })?;
                        let action = factory.try_create(data)?;
                        Ok(builder.action(action))
                    }
                    LeafKind::Condition => {
                        let factory = condition_factory_map.get(&leaf_hash).ok_or_else(|| {
                            anyhow!(
                                "condition factory of leaf with hash: {leaf_hash:?} does not exist"
                            )
                        })?;
                        let condition = factory.try_create(data)?;
                        Ok(builder.condition(condition))
                    }
                }
            }
        }
    }

    fn try_get_channel_mut<'a>(
        map: &'a mut ChannelIdToChannelMap,
        id: &ChannelId,
    ) -> Result<&'a mut TypeErasedChannel> {
        map.get_mut(id)
            .ok_or_else(|| anyhow!("channel id: {id:?} does not exist"))
    }
}

type ChannelIdToChannelMap = HashMap<ChannelId, TypeErasedChannel>;

struct ActionHashToFactoryMap {
    map: HashMap<NodeHash, node::ActionFactory>,
}

impl ActionHashToFactoryMap {
    fn new(
        plugins: Vec<Box<dyn NodePlugin<Description = LeafDescription, Factory = ActionFactory>>>,
    ) -> Self {
        let map = plugins
            .into_iter()
            .map(|plugin| (plugin.desc().hash(), plugin.factory()))
            .collect();
        Self { map }
    }

    fn get(&self, hash: &NodeHash) -> Option<&node::ActionFactory> {
        self.map.get(hash)
    }
}

struct ConditionHashToFactoryMap {
    map: HashMap<NodeHash, node::ConditionFactory>,
}

impl ConditionHashToFactoryMap {
    fn new(plugins: Vec<Box<ConditionNodePlugin>>) -> Self {
        let map = plugins
            .into_iter()
            .map(|plugin| (plugin.desc().hash(), plugin.factory()))
            .collect();
        Self { map }
    }

    fn get(&self, hash: &NodeHash) -> Option<&node::ConditionFactory> {
        self.map.get(hash)
    }
}

#[derive(Debug)]
struct ChannelHashToFactoryMap {
    map: HashMap<MessageHash, channel::Factory>,
}

impl ChannelHashToFactoryMap {
    fn new(plugins: Vec<Box<dyn ChannelPlugin>>) -> Self {
        let map = plugins
            .into_iter()
            .map(|plugin| (*plugin.desc().msg_hash(), plugin.factory()))
            .collect();
        Self { map }
    }

    fn get(&self, hash: &MessageHash) -> Option<&channel::Factory> {
        self.map.get(hash)
    }
}
