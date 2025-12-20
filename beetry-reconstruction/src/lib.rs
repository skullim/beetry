use anyhow::{Context, Result, anyhow};
use beetry_builder::Builder as BehaviorTreeBuilder;
use beetry_channel::external;
use beetry_core::{BoxNode, MessageHash, NonEmptyNodes, RegisterTask, Root, TaskHandle, Tree};
use beetry_editor_types::{
    ChannelStore, NodeId, NodeKind, NodePortConnection, NodePortKind, NodePortStore, NodeStore,
    ParameterValueStore,
};
use beetry_plugin::channel::{
    self, BoxChannelPlugin, BoxChannelPlugin2, ChannelPluginConstructor, ChannelPluginConstructor2,
    TypeErasedChannel,
};
use beetry_plugin::node::{
    self, BoxActionPlugin, BoxActionPlugin2, BoxConditionPlugin, BoxConditionPlugin2,
    BoxControlPlugin, BoxControlPlugin2, ControlMetadata, ControlReconstructionData, LeafMetadata,
    LeafReconstructionData,
};
use beetry_plugin::{BoxPlugin, Named, Plugin};
use beetry_plugin_types::node::{LeafKind, NodeName};
use beetry_reconstruction_types::node::{ControlSnapshot, LeafSnapshot, RootSnapshot};
use beetry_reconstruction_types::{
    channel::{ChannelId, ChannelSnapshotMap},
    node::{NodeSnapshot, NodeSnapshotData},
    tree::TreeSnapshot,
};
use std::collections::{BTreeSet, HashMap};
use tracing::debug;

pub struct TreeReconstructor {
    ext_receivers: external::ReceiverRegistry,
    node_factory: NodePluginRegistry,
}

impl TreeReconstructor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ext_receivers: external::ReceiverRegistry::new(),
            node_factory: NodePluginRegistry::new()?,
        })
    }

    pub fn with_receiver_registry(ext_receivers: external::ReceiverRegistry) -> Result<Self> {
        Ok(Self {
            ext_receivers,
            node_factory: NodePluginRegistry::new()?,
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
        let channel_factory_map = ChannelHashToPluginMap::new(ChannelPluginConstructor::plugins()?);
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
        factory_map: &ChannelHashToPluginMap,
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
        node_factory: &NodePluginRegistry,
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

                let factory = node_factory
                    .control
                    .get(&node_name)
                    .with_context(|| {
                        anyhow!("control factory for node: {node_name} does not exist")
                    })?
                    .factory();
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
                        let factory = node_factory
                            .action
                            .get(&node_name)
                            .with_context(|| {
                                anyhow!("action factory for node: {node_name} does not exist")
                            })?
                            .factory();
                        let action = factory.try_create(data)?;
                        Ok(builder.action(action))
                    }
                    LeafKind::Condition => {
                        let factory = node_factory
                            .condition
                            .get(&node_name)
                            .with_context(|| {
                                anyhow!("condition factory for node: {node_name} does not exist")
                            })?
                            .factory();
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

struct NodePluginRegistry {
    action: ActionToPluginMap,
    condition: ConditionToPluginMap,
    control: ControlToPluginMap,
}

impl NodePluginRegistry {
    fn new() -> Result<Self> {
        Ok(Self {
            action: ActionToPluginMap::new(node::ActionPluginConstructor::plugins()?),
            condition: ConditionToPluginMap::new(node::ConditionPluginConstructor::plugins()?),
            control: ControlToPluginMap::new(node::ControlPluginConstructor::plugins()?),
        })
    }
}

type ConditionToPluginMap = NodeNameToPluginMap<BoxConditionPlugin>;
type ActionToPluginMap = NodeNameToPluginMap<BoxActionPlugin>;
type ControlToPluginMap = NodeNameToPluginMap<BoxControlPlugin>;

struct NodeNameToPluginMap<P> {
    map: HashMap<NodeName, P>,
}

impl<P> NodeNameToPluginMap<P> {
    fn new<S, F>(plugins: impl IntoIterator<Item = BoxPlugin<S, F>>) -> Self
    where
        S: Named,
        HashMap<NodeName, P>: FromIterator<(NodeName, Box<dyn Plugin<Spec = S, Factory = F>>)>,
    {
        let map = plugins
            .into_iter()
            .map(|plugin| (NodeName(plugin.spec().name().to_string()), plugin))
            .collect();
        Self { map }
    }

    fn get(&self, name: &NodeName) -> Result<&P> {
        self.map
            .get(name)
            .ok_or_else(|| anyhow!("failed to obtain node plugin with name {name:?}"))
    }
}

type ChannelIdToChannelMap = HashMap<ChannelId, TypeErasedChannel>;

#[derive(Debug)]
struct ChannelHashToPluginMap {
    map: HashMap<MessageHash, BoxChannelPlugin>,
}

impl ChannelHashToPluginMap {
    fn new(plugins: Vec<BoxChannelPlugin>) -> Self {
        let map = plugins
            .into_iter()
            .map(|plugin| (plugin.spec().msg_hash(), plugin))
            .collect();
        Self { map }
    }

    fn get(&self, hash: MessageHash) -> Option<&channel::Factory> {
        self.map.get(&hash).map(|plugin| plugin.factory())
    }
}

pub struct TreeReconstructor2 {
    ext_receivers: external::ReceiverRegistry,
    node_plugins: NodePluginRegistry2,
}

impl TreeReconstructor2 {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ext_receivers: external::ReceiverRegistry::new(),
            node_plugins: NodePluginRegistry2::new()?,
        })
    }

    pub fn with_receiver_registry(ext_receivers: external::ReceiverRegistry) -> Result<Self> {
        Ok(Self {
            ext_receivers,
            node_plugins: NodePluginRegistry2::new()?,
        })
    }

    // Reconstruction criteria:
    // 1. Nodes exist in node factory registry.
    // 2. Channels exist in channel plugin registry.
    // 3. Each hash of leaf node matches with the corresponding node found in plugin registry.
    // 4. External receivers (if any) have been created when initializing Self instance
    pub fn try_reconstruct<RT, TH>(
        &mut self,
        tree: beetry_editor_types::ValidTree,
        builder: &BehaviorTreeBuilder<RT, TH>,
    ) -> Result<Tree<BoxNode>>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let tree = tree.into_inner();
        //@todo pass plugins to leave it up to user how constructors are provided
        let channel_factory_map =
            ChannelHashToPluginMap2::new(ChannelPluginConstructor2::plugins()?);
        let mut channels = Self::try_reconstruct_channels(tree.channel, &channel_factory_map)?;
        let root = Self::try_create_root_snapshot(
            tree.node,
            tree.parameter,
            tree.ports,
            &self.node_plugins,
        )?;

        let child = Self::try_reconstruct_tree(
            root.child,
            &self.node_plugins,
            &mut channels,
            &mut self.ext_receivers,
            builder,
        )?;
        Ok(Tree::new(Root::new(child)))
    }

    fn try_reconstruct_channels(
        store: ChannelStore,
        channel_plugin_map: &ChannelHashToPluginMap2,
    ) -> Result<ChannelIdToChannelMap> {
        let spec_map: HashMap<_, _> = store
            .specs
            .into_iter()
            .map(|record| (record.id, record.spec))
            .collect();
        store.channels.into_iter().map(|record| {
                let id = record.id;
                let data = record.data;
                let msg_hash = spec_map.get(&data.spec_id).ok_or_else(|| anyhow!("failed to get channel spec with id {}", data.spec_id))?.msg_hash();
                let factory = channel_plugin_map.get(msg_hash).ok_or_else(|| {
                    anyhow!(
                        "cannot create channel, failed to find channel constructor with required hash {msg_hash:?}"
                    )
                })?.factory();
                Ok((id, factory.create(data.config)))
            }
            ).collect::<Result<HashMap<_, _>>>()
    }

    fn try_create_root_snapshot(
        node_store: NodeStore,
        mut param_store: ParameterValueStore,
        mut port_store: NodePortStore,
        node_plugins: &NodePluginRegistry2,
    ) -> Result<RootSnapshot> {
        let root_id = node_store
            .nodes
            .iter()
            .filter_map(|record| {
                if node_store
                    .specs
                    .get(&record.value.spec_id())
                    .map(|spec_key| spec_key.kind())
                    == Some(NodeKind::Root)
                {
                    Some(record.id)
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| anyhow!("failed to find root id"))?;

        let root_child = *node_store
            .nodes
            .get(root_id)
            .expect("root id should exist")
            .children()
            .next()
            .expect("root should have a child node");

        let node_snapshot = Self::try_create_node_snapshot(
            root_child,
            &node_store,
            &mut param_store,
            &mut port_store,
            node_plugins,
        )?;
        Ok(RootSnapshot::new(node_snapshot))
    }

    fn try_create_node_snapshot(
        node_id: NodeId,
        node_store: &NodeStore,
        param_store: &mut ParameterValueStore,
        port_store: &mut NodePortStore,
        node_plugins: &NodePluginRegistry2,
    ) -> Result<NodeSnapshot> {
        let kind = node_store.specs.get(&node_id).unwrap().kind();
        let name = node_store.specs.get(&node_id).unwrap().name().clone();
        match kind {
            NodeKind::Control => {
                let children_id = node_store.nodes.get(&node_id).unwrap().children();
                //@todo should it also be IndexSet or BTreeSet?
                let mut children = vec![];
                for child_id in children_id {
                    children.push(Self::try_create_node_snapshot(
                        *child_id,
                        node_store,
                        param_store,
                        port_store,
                        node_plugins,
                    )?);
                }
                Ok(NodeSnapshot::builder()
                    .name(name)
                    .data(NodeSnapshotData::Control(ControlSnapshot::new(children)?))
                    .build())
            }
            NodeKind::Action => {
                let mut receivers = BTreeSet::new();
                let mut senders = BTreeSet::new();
                let mut ext_receivers = Vec::new();
                let mut ext_senders = Vec::new();

                for (port_id, conn) in port_store
                    .take(&node_id)
                    .into_iter()
                    .flat_map(|state| state.conns.into_iter())
                {
                    match conn {
                        //@todo create new type to convert NodePortConnection into ValidNodePortConnection
                        NodePortConnection::Unconnected => panic!("invalid state"),
                        NodePortConnection::External => {
                            let plugin = node_plugins.action.get(&name)?;
                            let spec = plugin.spec();
                            let ports_spec = spec.ports();
                            let port_spec = ports_spec.spec(port_id)?;
                            match port_spec.kind {
                                NodePortKind::Receiver => {
                                    ext_receivers.push(port_spec.msg_spec.hash())
                                }
                                NodePortKind::Sender => {
                                    ext_senders.push(port_spec.msg_spec.hash());
                                }
                            }
                        }
                        NodePortConnection::Internal(connections) => {
                            let plugin = node_plugins.action.get(&name)?;
                            let spec = plugin.spec();
                            let ports_spec = spec.ports();
                            let port_spec = ports_spec.spec(port_id)?;
                            match port_spec.kind {
                                NodePortKind::Receiver => receivers.extend(connections),
                                NodePortKind::Sender => {
                                    senders.extend(connections);
                                }
                            }
                        }
                    }
                }
                let leaf_snapshot = LeafSnapshot::builder()
                    .kind(LeafKind::Action)
                    .receivers(receivers)
                    .senders(senders)
                    .ext_receivers(ext_receivers)
                    .ext_senders(ext_senders)
                    .build();

                let params = param_store
                    .take(&node_id)
                    .map(|value| value.params)
                    .unwrap_or_default();

                Ok(NodeSnapshot::builder()
                    .name(name)
                    .data(leaf_snapshot)
                    .parameters(params)
                    .build())
            }
            _ => {
                todo!()
            }
        }
    }

    fn try_reconstruct_tree<RT, TH>(
        mut node: NodeSnapshot,
        node_plugins: &NodePluginRegistry2,
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
                            node_plugins,
                            channel_map,
                            ext_receivers_registry,
                            builder,
                        )
                    })
                    .collect::<Result<_>>()?;
                let children = NonEmptyNodes::try_from(children)
                    .map_err(|_| anyhow!("wrong export, no children found for control node"))?;

                let factory = node_plugins
                    .control
                    .get(&node_name)
                    .with_context(|| {
                        anyhow!("control factory for node: {node_name} does not exist")
                    })?
                    .factory();
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
                        let factory = node_plugins
                            .action
                            .get(&node_name)
                            .with_context(|| {
                                anyhow!("action factory for node: {node_name} does not exist")
                            })?
                            .factory();
                        let action = factory.try_create(data)?;
                        Ok(builder.action(action))
                    }
                    LeafKind::Condition => {
                        let factory = node_plugins
                            .condition
                            .get(&node_name)
                            .with_context(|| {
                                anyhow!("condition factory for node: {node_name} does not exist")
                            })?
                            .factory();
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

struct NodePluginRegistry2 {
    action: ActionToPluginMap2,
    condition: ConditionToPluginMap2,
    control: ControlToPluginMap2,
}

impl NodePluginRegistry2 {
    fn new() -> Result<Self> {
        Ok(Self {
            action: ActionToPluginMap2::new(node::ActionPluginConstructor2::plugins()?),
            condition: ConditionToPluginMap2::new(node::ConditionPluginConstructor2::plugins()?),
            control: ControlToPluginMap2::new(node::ControlPluginConstructor2::plugins()?),
        })
    }
}

type ConditionToPluginMap2 = NodeNameToPluginMap<BoxConditionPlugin2>;
type ActionToPluginMap2 = NodeNameToPluginMap<BoxActionPlugin2>;
type ControlToPluginMap2 = NodeNameToPluginMap<BoxControlPlugin2>;

struct ChannelHashToPluginMap2 {
    map: HashMap<MessageHash, BoxChannelPlugin2>,
}

impl ChannelHashToPluginMap2 {
    fn new(plugins: Vec<BoxChannelPlugin2>) -> Self {
        todo!()
    }

    fn get(&self, hash: MessageHash) -> Option<&BoxChannelPlugin2> {
        self.map.get(&hash)
    }
}
