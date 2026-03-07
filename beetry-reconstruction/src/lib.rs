use anyhow::{Context, Result, anyhow, bail};
use beetry_builder::Builder as BehaviorTreeBuilder;
use beetry_channel::external;
use beetry_core::{BoxNode, MessageHash, NonEmptyNodes, RegisterTask, Root, TaskHandle, Tree};
use beetry_editor_types::id::ChannelId;
use beetry_editor_types::output::node::Parameters;
use beetry_editor_types::spec::node::{LeafKind, NodeKind, NodeName, NodePortKind, NodeSpecKey};
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
pub use beetry_reconstruction_types::params::ParamsReconstructor;
use itertools::Itertools;
use std::collections::{BTreeSet, HashMap};
use tracing::debug;

pub struct TreeReconstructor {
    ext_receivers: external::ReceiverRegistry,
    node_plugins: NodePluginRegistry,
}

struct ReconstructionContext<'a> {
    node_store: &'a NodeStore,
    param_store: &'a mut ParameterStore,
    port_store: &'a mut PortStore,
    node_plugins: &'a NodePluginRegistry,
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

    #[expect(
        clippy::needless_pass_by_value,
        reason = "Store contains mostly Copy types which cannot be consumed"
    )]
    fn try_create_root_snapshot(
        node_store: NodeStore,
        mut param_store: ParameterStore,
        mut port_store: PortStore,
        node_plugins: &NodePluginRegistry,
    ) -> Result<RootSnapshot> {
        let root_id = node_store
            .nodes
            .iter()
            .find_map(|record| {
                if node_store
                    .specs
                    .get(&record.value.spec_id())
                    .map(NodeSpecKey::kind)
                    == Some(NodeKind::Root)
                {
                    Some(record.id)
                } else {
                    None
                }
            })
            .with_context(|| anyhow!("failed to find root id"))?;

        let root_node = node_store.nodes.get(root_id).with_context(|| {
            anyhow!("root node with id {root_id:?} does not exist in node store")
        })?;

        let root_child = *root_node
            .children()
            .next()
            .with_context(|| anyhow!("root node {root_id:?} does not have a child node"))?;

        let mut context = ReconstructionContext {
            node_store: &node_store,
            param_store: &mut param_store,
            port_store: &mut port_store,
            node_plugins,
        };

        let node_snapshot = Self::try_create_node_snapshot(root_child, &mut context)?;
        Ok(RootSnapshot::new(node_snapshot))
    }

    fn try_create_node_snapshot(
        node_id: NodeId,
        context: &mut ReconstructionContext<'_>,
    ) -> Result<NodeSnapshot> {
        let node_record =
            context.node_store.nodes.get(&node_id).with_context(|| {
                anyhow!("node with id {node_id:?} does not exist in node store")
            })?;
        let spec_id = node_record.spec_id();
        let spec_key = context
            .node_store
            .specs
            .get(&spec_id)
            .with_context(|| anyhow!("node spec with id {spec_id} does not exist"))?;
        let kind = spec_key.kind();
        let name = spec_key.name().clone();
        match kind {
            NodeKind::Control => {
                Self::try_create_control_snapshot(node_id, name, node_record.children(), context)
            }
            NodeKind::Decorator => {
                let child_id = node_record
                    .children()
                    .exactly_one()
                    .map_err(|children_iter| {
                        anyhow!(
                            "expected exactly one child for decorator node {name}, got {}",
                            children_iter.count()
                        )
                    })?;
                Self::try_create_decorator_snapshot(node_id, name, *child_id, context)
            }

            NodeKind::Leaf(leaf_kind) => {
                Self::try_create_leaf_snapshot(node_id, name, leaf_kind, context)
            }
            NodeKind::Root => bail!("unexpected Root node found during tree traversal"),
        }
    }

    fn try_create_control_snapshot<'a>(
        node_id: NodeId,
        name: NodeName,
        children_iter: impl Iterator<Item = &'a NodeId>,
        context: &mut ReconstructionContext<'_>,
    ) -> Result<NodeSnapshot> {
        let children: Vec<_> = children_iter
            .map(|child_id| Self::try_create_node_snapshot(*child_id, context))
            .collect::<Result<_>>()?;

        let params = context
            .param_store
            .take(&node_id)
            .map(|value| value.params)
            .unwrap_or_default();

        Ok(NodeSnapshot::builder()
            .name(name)
            .data(NodeSnapshotData::Control(ControlSnapshot::new(children)?))
            .parameters(params)
            .build())
    }

    fn try_create_decorator_snapshot(
        node_id: NodeId,
        name: NodeName,
        child_id: NodeId,
        context: &mut ReconstructionContext<'_>,
    ) -> Result<NodeSnapshot> {
        let child = Self::try_create_node_snapshot(child_id, context)?;

        let params = context
            .param_store
            .take(&node_id)
            .map(|value| value.params)
            .unwrap_or_default();

        Ok(NodeSnapshot::builder()
            .name(name)
            .data(NodeSnapshotData::Decorator(DecoratorSnapshot::new(child)))
            .parameters(params)
            .build())
    }

    fn try_create_leaf_snapshot(
        node_id: NodeId,
        name: NodeName,
        leaf_kind: LeafKind,
        context: &mut ReconstructionContext<'_>,
    ) -> Result<NodeSnapshot> {
        let spec = match leaf_kind {
            LeafKind::Action => {
                let plugin = context.node_plugins.action.get(&name)?;
                plugin.spec()
            }
            LeafKind::Condition => {
                let plugin = context.node_plugins.condition.get(&name)?;
                plugin.spec()
            }
        };

        let mut receivers = BTreeSet::new();
        let mut senders = BTreeSet::new();
        let mut ext_receivers = Vec::new();
        let mut ext_senders = Vec::new();

        for (port_id, port_state) in context
            .port_store
            .take_state(&node_id)
            .into_iter()
            .flatten()
        {
            let port_spec = spec
                .ports()
                .as_ref()
                .ok_or_else(|| anyhow!("expected port specification for node {name}"))?
                .spec(port_id)?;
            if port_state.is_external() {
                match port_spec.kind {
                    NodePortKind::Receiver => ext_receivers.push(port_spec.msg_spec.hash()),
                    NodePortKind::Sender => ext_senders.push(port_spec.msg_spec.hash()),
                }
                continue;
            }

            let port_connections = context.port_store.connections_iter().filter_map(|conn| {
                (conn.node_id == node_id && conn.port_id == port_id).then_some(&conn.channel_id)
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

        let params = context
            .param_store
            .take(&node_id)
            .map(|value| value.params)
            .unwrap_or_default();

        Ok(NodeSnapshot::builder()
            .name(name)
            .data(leaf_snapshot)
            .parameters(params)
            .build())
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
        let parameters = node.take_parameters();
        match node.data {
            NodeSnapshotData::Control(control) => Self::try_reconstruct_control(
                &node.name,
                parameters,
                control,
                node_plugins,
                channel_map,
                ext_receivers_registry,
                builder,
            ),
            NodeSnapshotData::Decorator(decorator) => Self::try_reconstruct_decorator(
                &node.name,
                parameters,
                decorator,
                node_plugins,
                channel_map,
                ext_receivers_registry,
                builder,
            ),
            NodeSnapshotData::Leaf(leaf) => Self::try_reconstruct_leaf(
                &node.name,
                parameters,
                leaf,
                node_plugins,
                channel_map,
                ext_receivers_registry,
                builder,
            ),
        }
    }

    fn try_reconstruct_control<RT, TH>(
        node_name: &NodeName,
        parameters: Parameters,
        control: ControlSnapshot,
        node_plugins: &NodePluginRegistry,
        channel_map: &mut ChannelIdToChannelMap,
        ext_receivers_registry: &mut external::ReceiverRegistry,
        builder: &BehaviorTreeBuilder<RT, TH>,
    ) -> Result<BoxNode>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
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
            .get(node_name)
            .with_context(|| anyhow!("control factory for node: {node_name} does not exist"))?
            .factory();
        let data = ControlReconstructionData::builder()
            .inner(ControlMetadata::new(children))
            .parameters(parameters)
            .build();
        factory.try_create(data)
    }

    fn try_reconstruct_decorator<RT, TH>(
        node_name: &NodeName,
        parameters: Parameters,
        decorator: DecoratorSnapshot,
        node_plugins: &NodePluginRegistry,
        channel_map: &mut ChannelIdToChannelMap,
        ext_receivers_registry: &mut external::ReceiverRegistry,
        builder: &BehaviorTreeBuilder<RT, TH>,
    ) -> Result<BoxNode>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let child = Self::try_reconstruct_tree(
            decorator.into(),
            node_plugins,
            channel_map,
            ext_receivers_registry,
            builder,
        )?;

        let factory = node_plugins
            .decorator
            .get(node_name)
            .with_context(|| anyhow!("decorator factory for node: {node_name} does not exist"))?
            .factory();
        let data = DecoratorReconstructionData::builder()
            .inner(DecoratorMetadata::new(child))
            .parameters(parameters)
            .build();
        factory.try_create(data)
    }

    fn try_reconstruct_leaf<RT, TH>(
        node_name: &NodeName,
        parameters: Parameters,
        mut leaf: LeafSnapshot,
        node_plugins: &NodePluginRegistry,
        channel_map: &mut ChannelIdToChannelMap,
        ext_receivers_registry: &mut external::ReceiverRegistry,
        builder: &BehaviorTreeBuilder<RT, TH>,
    ) -> Result<BoxNode>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let mut receivers: Vec<_> = leaf
            .take_receivers()
            .into_iter()
            .map(|id| Self::try_get_channel_mut(channel_map, id)?.try_take_receiver())
            .collect::<Result<_>>()?;

        for hash in leaf.take_ext_receivers() {
            if let Some(external_receiver) = ext_receivers_registry.take(hash) {
                receivers.push(external_receiver);
            }
        }

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
                    .get(node_name)
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
                    .get(node_name)
                    .with_context(|| {
                        anyhow!("condition factory for node: {node_name} does not exist")
                    })?
                    .factory();
                let condition = factory.try_create(data)?;
                Ok(builder.condition(condition))
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
