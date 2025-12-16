use anyhow::{Result, anyhow};
use beetry_builder::Builder as BehaviorTreeBuilder;
use beetry_channel::external;
use beetry_core::{BoxNode, MessageHash, NonEmptyNodes, RegisterTask, Root, TaskHandle, Tree};
use beetry_plugin::channel::{self, BoxChannelPlugin, TypeErasedChannel};
use beetry_plugin::node::{
    self, ControlMetadata, ControlReconstructionData, LeafMetadata, LeafReconstructionData,
};
use beetry_plugin::{BoxPlugin, Named};
use beetry_plugin_types::node::{LeafKind, NodeName};
use beetry_reconstruction_types::{
    channel::{ChannelId, ChannelSnapshotMap},
    node::{NodeSnapshot, NodeSnapshotData},
    tree::TreeSnapshot,
};
use std::collections::HashMap;
use tracing::debug;

pub struct TreeReconstructor {
    ext_receivers: external::ReceiverRegistry,
    node_factory: NodeFactoryRegistry,
}

impl TreeReconstructor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ext_receivers: external::ReceiverRegistry::new(),
            node_factory: NodeFactoryRegistry::new()?,
        })
    }

    pub fn with_receiver_registry(ext_receivers: external::ReceiverRegistry) -> Result<Self> {
        Ok(Self {
            ext_receivers,
            node_factory: NodeFactoryRegistry::new()?,
        })
    }

    // Reconstruction criteria:
    // 1. Nodes exist in node factory registry.
    // 2. Channels exist in channel plugin registry.
    // 3. Each hash of leaf node matches with the corresponding node found in plugin registry.
    // 4. External receivers (if any) have been created when initializing Self instance
    pub fn try_reconstruct<RT, TH>(
        &mut self,
        snapshot: TreeSnapshot,
        builder: &BehaviorTreeBuilder<RT, TH>,
    ) -> Result<Tree<BoxNode>>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let channel_factory_map = ChannelHashToFactoryMap::new(channel::plugins());
        let mut channels = Self::try_reconstruct_channels(snapshot.channels, &channel_factory_map)?;

        let child = Self::try_reconstruct_tree(
            snapshot.root.into_child(),
            &self.node_factory,
            &mut channels,
            &mut self.ext_receivers,
            builder,
        )?;
        Ok(Tree::new(Root::new(child)))
    }

    fn try_reconstruct_channels(
        snapshot_map: ChannelSnapshotMap,
        factory_map: &ChannelHashToFactoryMap,
    ) -> Result<ChannelIdToChannelMap> {
        snapshot_map
            .into_iter()
            .map(|(k, v)| {
                let msg_hash = v.spec().msg_hash();
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

    fn try_reconstruct_tree<RT, TH>(
        mut node: NodeSnapshot,
        node_factory: &NodeFactoryRegistry,
        channel_map: &mut ChannelIdToChannelMap,
        ext_receivers_registry: &mut external::ReceiverRegistry,
        builder: &BehaviorTreeBuilder<RT, TH>,
    ) -> Result<BoxNode>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let node_name = node.name.clone();
        let parameters = node.take_parameters();
        match node.data {
            NodeSnapshotData::Control(control) => {
                let children: Vec<_> = control
                    .into_children_iter()
                    .into_iter()
                    .map(|child| {
                        Self::try_reconstruct_tree(
                            child,
                            node_factory,
                            channel_map,
                            ext_receivers_registry,
                            builder,
                        )
                    })
                    .collect::<Result<_>>()?;
                let children = NonEmptyNodes::try_from(children)
                    .map_err(|_| anyhow!("wrong export, no children found for control node"))?;

                let factory = node_factory.control.get(&node_name).ok_or_else(|| {
                    anyhow!("control factory for node: {node_name} does not exist")
                })?;
                let data = ControlReconstructionData::builder()
                    .inner(ControlMetadata::new(children))
                    .parameters(parameters)
                    .build();
                factory.try_create(data)
            }
            NodeSnapshotData::Leaf(mut leaf) => {
                let mut receivers: Vec<_> = leaf
                    .take_receivers()
                    .into_iter()
                    .map(|id| Self::try_get_channel_mut(channel_map, id)?.try_take_receiver())
                    .collect::<Result<_>>()?;

                let ext_receivers_snapshot = leaf.take_ext_receivers();
                ext_receivers_snapshot
                    .into_iter()
                    .map(|hash| ext_receivers_registry.take(hash))
                    .for_each(|o_external_receiver| {
                        if let Some(external_receiver) = o_external_receiver {
                            receivers.push(external_receiver);
                        }
                    });

                let senders: Vec<_> = leaf
                    .take_senders()
                    .into_iter()
                    .map(|id| Self::try_get_channel_mut(channel_map, id)?.try_take_sender())
                    .collect::<Result<_>>()?;

                let data = LeafReconstructionData::builder()
                    .inner(
                        LeafMetadata::builder()
                            .receivers(receivers)
                            .senders(senders)
                            .build(),
                    )
                    .parameters(parameters)
                    .build();

                match leaf.kind() {
                    LeafKind::Action => {
                        let factory = node_factory.action.get(&node_name).ok_or_else(|| {
                            anyhow!("action factory for node: {node_name} does not exist")
                        })?;
                        let action = factory.try_create(data)?;
                        Ok(builder.action(action))
                    }
                    LeafKind::Condition => {
                        let factory = node_factory.condition.get(&node_name).ok_or_else(|| {
                            anyhow!("condition factory for node: {node_name} does not exist")
                        })?;
                        let condition = factory.try_create(data)?;
                        Ok(builder.condition(condition))
                    }
                }
            }
        }
    }

    fn try_get_channel_mut(
        map: &mut ChannelIdToChannelMap,
        id: ChannelId,
    ) -> Result<&mut TypeErasedChannel> {
        map.get_mut(&id)
            .ok_or_else(|| anyhow!("channel id: {id:?} does not exist"))
    }
}

struct NodeFactoryRegistry {
    action: ActionNameToFactoryMap,
    condition: ConditionNameToFactoryMap,
    control: ControlNameToFactoryMap,
}

impl NodeFactoryRegistry {
    fn new() -> Result<Self> {
        Ok(Self {
            action: ActionNameToFactoryMap::new(node::ActionPluginConstructor::plugins()?),
            condition: ConditionNameToFactoryMap::new(node::ConditionPluginConstructor::plugins()?),
            control: ControlNameToFactoryMap::new(node::ControlPluginConstructor::plugins()?),
        })
    }
}

type ConditionNameToFactoryMap = NameToFactoryMap<node::ConditionFactory>;
type ActionNameToFactoryMap = NameToFactoryMap<node::ActionFactory>;
type ControlNameToFactoryMap = NameToFactoryMap<node::ControlFactory>;

struct NameToFactoryMap<F> {
    map: HashMap<NodeName, F>,
}

impl<F> NameToFactoryMap<F> {
    fn new<S>(plugins: impl IntoIterator<Item = BoxPlugin<S, F>>) -> Self
    where
        S: Named,
    {
        let map = plugins
            .into_iter()
            .map(|plugin| (NodeName(plugin.spec().name().to_string()), plugin.factory()))
            .collect();
        Self { map }
    }

    fn get(&self, name: &NodeName) -> Option<&F> {
        self.map.get(name)
    }
}

type ChannelIdToChannelMap = HashMap<ChannelId, TypeErasedChannel>;

#[derive(Debug)]
struct ChannelHashToFactoryMap {
    map: HashMap<MessageHash, channel::Factory>,
}

impl ChannelHashToFactoryMap {
    fn new(plugins: Vec<BoxChannelPlugin>) -> Self {
        let map = plugins
            .into_iter()
            .map(|plugin| (plugin.spec().msg_hash(), plugin.factory()))
            .collect();
        Self { map }
    }

    fn get(&self, hash: MessageHash) -> Option<&channel::Factory> {
        self.map.get(&hash)
    }
}
