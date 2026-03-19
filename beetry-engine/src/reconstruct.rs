//! Rebuilds a runtime tree from persisted editor data in two steps.
//! First, this module builds a snapshot tree: an AST-like representation that
//! preserves the hierarchy together with the data needed to create node
//! instances. Then [`TreeReconstructor`] recreates the serialized channels,
//! consumes the snapshot tree, wires the resolved channel endpoints into
//! nodes, and constructs the runtime tree.

mod snapshot;

use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result, anyhow};
use beetry_core::{BoxNode, NonEmptyNodes, RegisterTask, Root, TaskHandle, Tree, leaf};
use beetry_editor_types::{
    id::ChannelId,
    output::node::Parameters,
    persistence,
    spec::node::{LeafKind, NodeName},
};
use beetry_message::MessageHash;
use beetry_plugin::{
    BoxPlugin, Named, Plugin,
    channel::{BoxChannelPlugin, ChannelPluginConstructor, TypeErasedChannel},
    node::{
        ActionPluginConstructor, BoxActionPlugin, BoxConditionPlugin, BoxControlPlugin,
        BoxDecoratorPlugin, ConditionPluginConstructor, ControlContext, ControlPluginConstructor,
        ControlReconstructionData, DecoratorContext, DecoratorPluginConstructor,
        DecoratorReconstructionData, LeafContext, LeafReconstructionData,
    },
};
use tracing::debug;

use self::snapshot::{Control, Decorator, Leaf, Node, NodeData, TreeSnapshotBuilder};

pub struct TreeReconstructor {
    node_plugins: NodePluginRegistry,
}

impl TreeReconstructor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            node_plugins: NodePluginRegistry::new()?,
        })
    }

    pub fn try_reconstruct<RT, TH>(
        &self,
        tree: persistence::tree::ValidTreeStore,
        builder: &leaf::Builder<RT, TH>,
    ) -> Result<Tree<BoxNode>>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let tree = tree.into_inner();
        let channel_factory_map = ChannelHashToPluginMap::new(ChannelPluginConstructor::plugins()?);
        let mut channels = Self::try_reconstruct_channels(tree.channel, &channel_factory_map)?;
        let (node, port, parameter) = (tree.node, tree.port, tree.parameter);
        let mut tree_snapshot_builder =
            TreeSnapshotBuilder::new(node, parameter, port, &self.node_plugins);
        let root = tree_snapshot_builder.build_root()?;

        let child =
            Self::try_reconstruct_tree(root.child, &self.node_plugins, &mut channels, builder)?;
        Ok(Tree::new(Root::new(child)))
    }

    fn try_reconstruct_channels(
        store: persistence::channel::Store,
        channel_plugin_map: &ChannelHashToPluginMap,
    ) -> Result<ChannelIdToChannelMap> {
        store
            .channels
            .into_records()
            .map(|record| {
                let id = record.id;
                let data = record.data;
                let msg_hash = store
                    .specs
                    .get(&data.spec_id)
                    .ok_or_else(|| anyhow!("failed to get channel spec with id {}", data.spec_id))?
                    .msg_hash();
                let factory = channel_plugin_map
                    .get(msg_hash)
                    .ok_or_else(|| {
                        anyhow!(
                            "cannot create channel, failed to find channel constructor with required hash {msg_hash:?}"
                        )
                    })?
                    .factory();
                Ok((id, factory.create(data.config)))
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    fn try_reconstruct_tree<RT, TH>(
        mut node: Node,
        node_plugins: &NodePluginRegistry,
        channel_map: &mut ChannelIdToChannelMap,
        builder: &leaf::Builder<RT, TH>,
    ) -> Result<BoxNode>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let parameters = node.take_parameters();
        match node.data {
            NodeData::Control(control) => Self::try_reconstruct_control(
                &node.name,
                parameters,
                control,
                node_plugins,
                channel_map,
                builder,
            ),
            NodeData::Decorator(decorator) => Self::try_reconstruct_decorator(
                &node.name,
                parameters,
                decorator,
                node_plugins,
                channel_map,
                builder,
            ),
            NodeData::Leaf(leaf) => Self::try_reconstruct_leaf(
                &node.name,
                parameters,
                leaf,
                node_plugins,
                channel_map,
                builder,
            ),
        }
    }

    fn try_reconstruct_control<RT, TH>(
        node_name: &NodeName,
        parameters: Parameters,
        control: Control,
        node_plugins: &NodePluginRegistry,
        channel_map: &mut ChannelIdToChannelMap,
        builder: &leaf::Builder<RT, TH>,
    ) -> Result<BoxNode>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let children: Vec<_> = control
            .into_children()
            .into_iter()
            .map(|child| Self::try_reconstruct_tree(child, node_plugins, channel_map, builder))
            .collect::<Result<_>>()?;
        let children = NonEmptyNodes::try_from(children)
            .map_err(|_| anyhow!("wrong export, no children found for control node"))?;

        let factory = node_plugins
            .control
            .get(node_name)
            .with_context(|| anyhow!("control factory for node: {node_name} does not exist"))?
            .factory();
        let data = ControlReconstructionData::builder()
            .context(ControlContext::new(children))
            .parameters(parameters)
            .build();
        factory.try_create(data)
    }

    fn try_reconstruct_decorator<RT, TH>(
        node_name: &NodeName,
        parameters: Parameters,
        decorator: Decorator,
        node_plugins: &NodePluginRegistry,
        channel_map: &mut ChannelIdToChannelMap,
        builder: &leaf::Builder<RT, TH>,
    ) -> Result<BoxNode>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let child =
            Self::try_reconstruct_tree(decorator.into_child(), node_plugins, channel_map, builder)?;

        let factory = node_plugins
            .decorator
            .get(node_name)
            .with_context(|| anyhow!("decorator factory for node: {node_name} does not exist"))?
            .factory();
        let data = DecoratorReconstructionData::builder()
            .context(DecoratorContext::new(child))
            .parameters(parameters)
            .build();
        factory.try_create(data)
    }

    fn try_reconstruct_leaf<RT, TH>(
        node_name: &NodeName,
        parameters: Parameters,
        mut leaf: Leaf,
        node_plugins: &NodePluginRegistry,
        channel_map: &mut ChannelIdToChannelMap,
        builder: &leaf::Builder<RT, TH>,
    ) -> Result<BoxNode>
    where
        RT: RegisterTask<TH> + 'static,
        TH: TaskHandle + 'static,
    {
        let receivers: BTreeMap<_, _> = leaf
            .take_receivers()
            .into_iter()
            .map(|(key, id)| {
                Ok((
                    key,
                    Self::try_get_channel_mut(channel_map, id)?.try_take_receiver()?,
                ))
            })
            .collect::<Result<_>>()?;

        let senders: BTreeMap<_, _> = leaf
            .take_senders()
            .into_iter()
            .map(|(key, id)| {
                Ok((
                    key,
                    Self::try_get_channel_mut(channel_map, id)?.try_take_sender()?,
                ))
            })
            .collect::<Result<_>>()?;

        let data = LeafReconstructionData::builder()
            .context(
                LeafContext::builder()
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
                Ok(builder.condition(factory.try_create(data)?))
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

pub struct NodePluginRegistry {
    pub action: ActionToPluginMap,
    pub condition: ConditionToPluginMap,
    pub control: ControlToPluginMap,
    pub decorator: DecoratorToPluginMap,
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

pub struct NodeNameToPluginMap<P> {
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
