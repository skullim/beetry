use anyhow::{Context, Result, anyhow};
use beetry_builder::Builder as BehaviorTreeBuilder;
use beetry_channel::external;
use beetry_core::{BoxNode, MessageHash, NonEmptyNodes, RegisterTask, Root, TaskHandle, Tree};
use beetry_editor_types::id::{ChannelId, PortConnectionId};
use beetry_editor_types::output::node::{ParameterValue, Parameters};
use beetry_editor_types::spec::node::{LeafKind, NodeKind, NodeName, NodePortKind};
use beetry_editor_types::{
    id::NodeId,
    persistence::{ChannelStore, NodeStore, ParameterStore, PortStore, ValidTree},
};
use beetry_plugin::channel::{BoxChannelPlugin, ChannelPluginConstructor, TypeErasedChannel};
use beetry_plugin::node::{
    ActionPluginConstructor, BoxActionPlugin, BoxConditionPlugin, BoxControlPlugin,
    BoxDecoratorPlugin, ConditionPluginConstructor, ControlPluginConstructor,
    DecoratorPluginConstructor,
};
use beetry_plugin::{BoxPlugin, Named, Plugin};
use beetry_reconstruction_types::node::{
    ControlMetadata, ControlReconstructionData, ControlSnapshot, DecoratorMetadata,
    DecoratorReconstructionData, DecoratorSnapshot, LeafMetadata, LeafReconstructionData,
    LeafSnapshot, RootSnapshot,
};
use beetry_reconstruction_types::node::{NodeSnapshot, NodeSnapshotData};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use tracing::debug;

pub struct TreeReconstructor {
    ext_receivers: external::ReceiverRegistry,
    node_plugins: NodePluginRegistry,
}

impl TreeReconstructor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ext_receivers: external::ReceiverRegistry::new(),
            node_plugins: NodePluginRegistry::new()?,
        })
    }

    pub fn with_receiver_registry(ext_receivers: external::ReceiverRegistry) -> Result<Self> {
        Ok(Self {
            ext_receivers,
            node_plugins: NodePluginRegistry::new()?,
        })
    }

    // Reconstruction criteria:
    // 1. Nodes exist in node factory registry.
    // 2. Channels exist in channel plugin registry.
    // 3. Each hash of leaf node matches with the corresponding node found in plugin registry.
    // 4. External receivers (if any) have been created when initializing Self instance
    pub fn try_reconstruct<RT, TH>(
        &mut self,
        tree: ValidTree,
        builder: &BehaviorTreeBuilder<RT, TH>,
    ) -> Result<Tree<BoxNode>>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let tree = tree.into_inner();
        //@todo pass plugins to leave it up to user how constructors are provided
        let channel_factory_map = ChannelHashToPluginMap::new(ChannelPluginConstructor::plugins()?);
        let mut channels = Self::try_reconstruct_channels(tree.channel, &channel_factory_map)?;
        let root = Self::try_create_root_snapshot(
            tree.node,
            tree.parameter,
            tree.port,
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
        channel_plugin_map: &ChannelHashToPluginMap,
    ) -> Result<ChannelIdToChannelMap> {
        store.channels.into_records().map(|record| {
                let id = record.id;
                let data = record.data;
                let msg_hash = store.specs.get(&data.spec_id).ok_or_else(|| anyhow!("failed to get channel spec with id {}", data.spec_id))?.msg_hash();
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
        mut param_store: ParameterStore,
        mut port_store: PortStore,
        node_plugins: &NodePluginRegistry,
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
            .with_context(|| anyhow!("failed to find root id"))?;

        let root_node = node_store.nodes.get(root_id).with_context(|| {
            anyhow!("root node with id {root_id:?} does not exist in node store")
        })?;

        let root_child = *root_node
            .children()
            .next()
            .with_context(|| anyhow!("root node {root_id:?} does not have a child node"))?;
        let connections = port_store.take_connections();

        let node_snapshot = Self::try_create_node_snapshot(
            root_child,
            &node_store,
            &mut param_store,
            &mut port_store,
            &connections,
            node_plugins,
        )?;
        Ok(RootSnapshot::new(node_snapshot))
    }

    fn try_create_node_snapshot(
        node_id: NodeId,
        node_store: &NodeStore,
        param_store: &mut ParameterStore,
        port_store: &mut PortStore,
        connections: &std::collections::HashSet<PortConnectionId>,
        node_plugins: &NodePluginRegistry,
    ) -> Result<NodeSnapshot> {
        let node_record = node_store
            .nodes
            .get(&node_id)
            .with_context(|| anyhow!("node with id {node_id:?} does not exist in node store"))?;
        let spec_id = node_record.spec_id();
        let spec_key = node_store
            .specs
            .get(&spec_id)
            .with_context(|| anyhow!("node spec with id {spec_id} does not exist"))?;
        let kind = spec_key.kind();
        let name = spec_key.name().clone();
        match kind {
            NodeKind::Control => {
                let children_id = node_record.children();
                let mut children = vec![];
                for child_id in children_id {
                    children.push(Self::try_create_node_snapshot(
                        *child_id,
                        node_store,
                        param_store,
                        port_store,
                        connections,
                        node_plugins,
                    )?);
                }
                Ok(NodeSnapshot::builder()
                    .name(name)
                    .data(NodeSnapshotData::Control(ControlSnapshot::new(children)?))
                    .build())
            }
            NodeKind::Decorator => {
                let children: Vec<_> = node_record.children().copied().collect();
                if children.len() != 1 {
                    return Err(anyhow!(
                        "expected exactly one child for decorator node {name}, got {}",
                        children.len()
                    ));
                }

                let child = Self::try_create_node_snapshot(
                    children[0],
                    node_store,
                    param_store,
                    port_store,
                    connections,
                    node_plugins,
                )?;

                Ok(NodeSnapshot::builder()
                    .name(name)
                    .data(NodeSnapshotData::Decorator(DecoratorSnapshot::new(child)))
                    .build())
            }
            NodeKind::Leaf(leaf_kind) => {
                let mut receivers = BTreeSet::new();
                let mut senders = BTreeSet::new();
                let mut ext_receivers = Vec::new();
                let mut ext_senders = Vec::new();

                for (port_id, port_state) in port_store.take_state(&node_id).into_iter().flatten() {
                    let spec = {
                        match leaf_kind {
                            LeafKind::Action => {
                                let plugin = node_plugins.action.get(&name)?;
                                plugin.spec()
                            }
                            LeafKind::Condition => {
                                let plugin = node_plugins.condition.get(&name)?;
                                plugin.spec()
                            }
                        }
                    };
                    let ports_spec = spec
                        .ports()
                        .as_ref()
                        .ok_or_else(|| anyhow!("expected port specification for node {name}"))?;
                    let port_spec = ports_spec.spec(port_id)?;
                    if port_state.is_external() {
                        match port_spec.kind {
                            NodePortKind::Receiver => ext_receivers.push(port_spec.msg_spec.hash()),
                            NodePortKind::Sender => ext_senders.push(port_spec.msg_spec.hash()),
                        }
                        continue;
                    }

                    let port_connections = connections.iter().filter_map(|conn| {
                        (conn.node_id == node_id && conn.port_id == port_id)
                            .then_some(&conn.channel_id)
                    });
                    match port_spec.kind {
                        NodePortKind::Receiver => receivers.extend(port_connections),
                        NodePortKind::Sender => senders.extend(port_connections),
                    }
                }

                let leaf_snapshot = LeafSnapshot::builder()
                    .kind(leaf_kind)
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
        node_plugins: &NodePluginRegistry,
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
            NodeSnapshotData::Decorator(decorator) => {
                let child = Self::try_reconstruct_tree(
                    decorator.into(),
                    node_plugins,
                    channel_map,
                    ext_receivers_registry,
                    builder,
                )?;

                let factory = node_plugins
                    .decorator
                    .get(&node_name)
                    .with_context(|| {
                        anyhow!("decorator factory for node: {node_name} does not exist")
                    })?
                    .factory();
                let data = DecoratorReconstructionData::builder()
                    .inner(DecoratorMetadata::new(child))
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
                debug!("created node reconstruction data {data:?} for node {node_name}");

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

struct NodePluginRegistry {
    action: ActionToPluginMap,
    condition: ConditionToPluginMap,
    control: ControlToPluginMap,
    decorator: DecoratorToPluginMap,
}

impl NodePluginRegistry {
    fn new() -> Result<Self> {
        Ok(Self {
            action: ActionToPluginMap::new(ActionPluginConstructor::plugins()?),
            condition: ConditionToPluginMap::new(ConditionPluginConstructor::plugins()?),
            control: ControlToPluginMap::new(ControlPluginConstructor::plugins()?),
            decorator: DecoratorToPluginMap::new(DecoratorPluginConstructor::plugins()?),
        })
    }
}

type ConditionToPluginMap = NodeNameToPluginMap<BoxConditionPlugin>;
type ActionToPluginMap = NodeNameToPluginMap<BoxActionPlugin>;
type ControlToPluginMap = NodeNameToPluginMap<BoxControlPlugin>;
type DecoratorToPluginMap = NodeNameToPluginMap<BoxDecoratorPlugin>;

struct ChannelHashToPluginMap {
    map: HashMap<MessageHash, BoxChannelPlugin>,
}

impl ChannelHashToPluginMap {
    fn new(plugins: Vec<BoxChannelPlugin>) -> Self {
        Self {
            map: plugins
                .into_iter()
                .map(|p| (p.spec().msg_hash(), p))
                .collect(),
        }
    }

    fn get(&self, hash: MessageHash) -> Option<&BoxChannelPlugin> {
        self.map.get(&hash)
    }
}

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

pub struct ParamsReconstructor;

impl ParamsReconstructor {
    pub fn reconstruct<T>(params: Parameters) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let deserializer = serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(
            serde_value::Value::Map(
                params
                    .into_iter()
                    .map(|(name, value)| {
                        let value = match value {
                            ParameterValue::Bool(b) => serde_value::Value::Bool(b),
                            ParameterValue::U64(u) => serde_value::Value::U64(u),
                            ParameterValue::I64(i) => serde_value::Value::I64(i),
                            ParameterValue::F64(f) => serde_value::Value::F64(f),
                            ParameterValue::String(s) => serde_value::Value::String(s),
                        };
                        (serde_value::Value::String(name), value)
                    })
                    .collect::<BTreeMap<_, _>>(),
            ),
        );
        Ok(T::deserialize(deserializer)?)
    }
}
