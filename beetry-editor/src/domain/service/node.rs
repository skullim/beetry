use crate::domain::{
    models::{
        NodeChannelPortId, NodeId, NodeKind, NodePortConnection, NodePortKind, NodePortSpec,
        NodePosition, NodeSpec,
    },
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeKindRepositoryConcept, NodeNameRepositoryConcept, NodePortRepositoryConcept,
        NodePositionRepositoryConcept, NodeRepositoryConcept, NodeRepositoryFacadeConcept,
        NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut, ParamRepositoryConcept,
    },
    service::edge::EdgeService,
};
use anyhow::{Result, anyhow, bail};
use beetry_plugin::ActionSpec;
use beetry_serde::{de::parameter::Parameters, ser::node::NodeName};

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeServiceView<'r, 's, 'e, NRF, ER, CR, A> {
    repo: &'r mut EditorRepository<NRF, ER, CR>,
    node_service: &'s mut NodeService<A>,
    edge_service: &'e mut EdgeService,
}

impl<'r, 's, 'e, NRF, ER, CR, A> NodeServiceView<'r, 's, 'e, NRF, ER, CR, A>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
    A: AssignNodeId,
{
    pub(crate) fn new(
        repo: &'r mut EditorRepository<NRF, ER, CR>,
        node_service: &'s mut NodeService<A>,
        edge_service: &'e mut EdgeService,
    ) -> Self {
        Self {
            repo,
            node_service,
            edge_service,
        }
    }

    pub fn create_node(&mut self, spec: NodeSpec) -> Result<NodeId> {
        let view = self.repo.node_mut().view_mut();
        //@todo add name, kind, parameters, ports
        self.node_service.create_node(spec, view)
    }

    pub fn remove(&mut self, id: NodeId) -> Result<()> {
        let repo = &mut self.repo;
        NodeServiceStateless::on_node_removal(repo.node_mut(), id)?;
        self.edge_service.on_node_removal(repo.edge_mut(), id)
    }

    pub fn name(&self, id: NodeId) -> Result<&NodeName> {
        NodeServiceStateless::name(self.repo.node(), id)
    }

    pub fn kind(&self, id: NodeId) -> Result<NodeKind> {
        let NodeRepositoryFacadeView { kinds, .. } = self.repo.node().view();
        NodeKindService::kind(kinds, id)
    }

    pub fn nodes(&self, kind: NodeKind) -> impl Iterator<Item = NodeId> {
        NodeServiceStateless::nodes(self.repo.node(), kind)
    }

    pub fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        NodeServiceStateless::update_position(self.repo.node_mut(), id, position)
    }

    pub fn positions(&self, kind: NodeKind) -> impl Iterator<Item = &NodePosition> {
        NodeServiceStateless::positions(self.repo.node(), kind)
    }

    pub fn parameters(&self, id: NodeId) -> Result<&Parameters> {
        NodeServiceStateless::parameters(self.repo.node(), id)
    }

    pub fn port_ids(&self, node_id: NodeId) -> impl Iterator<Item = NodeChannelPortId> {
        let NodeRepositoryFacadeView { ports, .. } = self.repo.node().view();
        NodeServiceStateless::port_ids(ports, node_id)
    }

    pub fn port_spec(&self, node_id: NodeId, port_id: NodeChannelPortId) -> Result<&NodePortSpec> {
        let NodeRepositoryFacadeView { ports, .. } = self.repo.node().view();
        NodeServiceStateless::port_spec(ports, node_id, port_id)
    }
    pub fn port_connection(
        &self,
        node_id: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<&NodePortConnection> {
        let NodeRepositoryFacadeView { ports, .. } = self.repo.node().view();
        NodeServiceStateless::port_connection(ports, node_id, port_id)
    }
}

pub struct NodeServiceStateless;

impl NodeServiceStateless {
    pub(crate) fn ensure_exists(
        view: NodeRepositoryFacadeView<'_, impl NodeRepositoryFacadeConcept>,
        id: NodeId,
    ) -> Result<()> {
        let kind = NodeKindService::kind(view.kinds, id)?;

        let contains = match kind {
            NodeKind::Action => view.action.contains(id),
            NodeKind::Condition => view.condition.contains(id),
            NodeKind::Control => view.control.contains(id),
            NodeKind::Decorator => view.decorator.contains(id),
            NodeKind::Root => view.root.contains(id),
        };

        if !contains {
            bail!("node {id} does not exist");
        }
        Ok(())
    }

    fn initialize_ports(
        ports_repo: &mut impl NodePortRepositoryConcept,
        id: NodeId,
        spec: &ActionSpec,
    ) -> Result<()> {
        let mut port_specs = vec![];
        for msg_spec in &spec.schema.senders {
            port_specs.push(NodePortSpec {
                kind: NodePortKind::Sender,
                msg_spec: msg_spec.clone(),
            });
        }
        for msg_spec in &spec.schema.receivers {
            port_specs.push(NodePortSpec {
                kind: NodePortKind::Receiver,
                msg_spec: msg_spec.clone(),
            });
        }

        Self::create_node_ports(ports_repo, id, port_specs.into_iter())?;
        // collect to avoid borrowing mutably in the for loop
        let port_ids: Vec<_> = Self::port_ids(ports_repo, id).collect();
        for port_id in port_ids {
            Self::connect_port(ports_repo, id, port_id, NodePortConnection::default())?;
        }
        Ok(())
    }

    fn nodes(
        repo: &impl NodeRepositoryFacadeConcept,
        kind: NodeKind,
    ) -> impl Iterator<Item = NodeId> {
        let view = repo.view();

        match kind {
            NodeKind::Root => view.root.nodes(),
            NodeKind::Action => view.action.nodes(),
            NodeKind::Condition => view.condition.nodes(),
            NodeKind::Control => view.control.nodes(),
            NodeKind::Decorator => view.decorator.nodes(),
        }
    }

    fn update_position(
        repo: &mut impl NodeRepositoryFacadeConcept,
        id: NodeId,
        position: NodePosition,
    ) -> Result<()> {
        Self::ensure_exists(repo.view(), id)?;
        let NodeRepositoryFacadeViewMut { positions, .. } = repo.view_mut();
        positions.update(id, position)
    }

    fn positions(
        repo: &impl NodeRepositoryFacadeConcept,
        kind: NodeKind,
    ) -> impl Iterator<Item = &NodePosition> {
        let view = repo.view();
        let nodes = match kind {
            NodeKind::Action => view.action.nodes(),
            NodeKind::Condition => view.condition.nodes(),
            NodeKind::Control => view.control.nodes(),
            NodeKind::Decorator => view.decorator.nodes(),
            NodeKind::Root => view.root.nodes(),
        };
        nodes.flat_map(|id| view.positions.position(id))
    }

    fn insert_parameter() {
        todo!()
    }

    fn parameters(repo: &impl NodeRepositoryFacadeConcept, id: NodeId) -> Result<&Parameters> {
        let view = repo.view();
        view.parameters
            .params(id)
            .ok_or_else(|| anyhow!("failed to obtain parameters for node {id}"))
    }

    fn on_node_removal(repo: &mut impl NodeRepositoryFacadeConcept, id: NodeId) -> Result<()> {
        let view = repo.view_mut();
        let kind = NodeKindService::kind(view.kinds, id)?;
        match kind {
            NodeKind::Action => view.action.remove(id)?,
            NodeKind::Condition => view.condition.remove(id)?,
            NodeKind::Control => view.control.remove(id)?,
            NodeKind::Decorator => view.decorator.remove(id)?,
            NodeKind::Root => view.root.remove(id)?,
        };
        view.kinds.remove(id)?;
        view.positions.remove(id)?;
        view.parameters.remove(id)
    }

    fn name(repo: &impl NodeRepositoryFacadeConcept, id: NodeId) -> Result<&NodeName> {
        let NodeRepositoryFacadeView { names, .. } = repo.view();
        names
            .name(id)
            .ok_or_else(|| anyhow!("no name found for node {id}"))
    }

    fn create_node_ports(
        repo: &mut impl NodePortRepositoryConcept,
        node_id: NodeId,
        port_specs: impl Iterator<Item = NodePortSpec>,
    ) -> Result<()> {
        repo.create(node_id, port_specs)
    }

    pub(crate) fn connect_port(
        repo: &mut impl NodePortRepositoryConcept,
        node_id: NodeId,
        port_id: NodeChannelPortId,
        kind: NodePortConnection,
    ) -> Result<()> {
        repo.set_conn(node_id, port_id, kind)
    }

    pub(crate) fn port_ids(
        repo: &impl NodePortRepositoryConcept,
        node_id: NodeId,
    ) -> impl Iterator<Item = NodeChannelPortId> {
        repo.ports(node_id)
    }

    pub(crate) fn port_spec(
        repo: &impl NodePortRepositoryConcept,
        node_id: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<&NodePortSpec> {
        repo.spec(node_id, port_id)
            .ok_or_else(|| anyhow!("unable to retrieve port spec"))
    }

    pub(crate) fn port_connection(
        repo: &impl NodePortRepositoryConcept,
        node_id: NodeId,
        port_id: NodeChannelPortId,
    ) -> Result<&NodePortConnection> {
        repo.connection(node_id, port_id).ok_or_else(|| {
            anyhow!("unable to retrieve node's (id: {node_id}) port (id: {port_id}) connection")
        })
    }
}

mod sealed {
    use beetry_serde::ser::node::{ActionSpec, ControlSpec, DecoratorSpec};
    // negative trait would be nicer, but is not available yet on stable
    pub trait NonRootSpec {}
    impl NonRootSpec for ActionSpec {}
    impl NonRootSpec for DecoratorSpec {}
    impl NonRootSpec for ControlSpec {}
}

pub struct NodeService<A> {
    id_assigner: A,
}

impl NodeService<IncrementalNodeIdAssigner> {
    pub(crate) fn new() -> Self {
        Self {
            id_assigner: IncrementalNodeIdAssigner::default(),
        }
    }
}

impl<A> NodeService<A>
where
    A: AssignNodeId,
{
    pub(crate) fn with_assigner(id_assigner: A) -> Self {
        Self { id_assigner }
    }

    pub fn create_node(
        &mut self,
        spec: NodeSpec,
        view: NodeRepositoryFacadeViewMut<'_, impl NodeRepositoryFacadeConcept>,
    ) -> Result<NodeId> {
        match spec.kind {
            NodeKind::Root => NodeCreator::create_root(view.root, 0),
            NodeKind::Action => NodeCreator::create(view.action, self.id_assigner.next_id()),
            NodeKind::Condition => NodeCreator::create(view.condition, self.id_assigner.next_id()),
            NodeKind::Control => NodeCreator::create(view.control, self.id_assigner.next_id()),
            NodeKind::Decorator => NodeCreator::create(view.decorator, self.id_assigner.next_id()),
        }
    }
}

pub trait AssignNodeId {
    fn next_id(&mut self) -> NodeId;
}

#[derive(Default)]
pub(crate) struct IncrementalNodeIdAssigner {
    id: NodeId,
}

impl AssignNodeId for IncrementalNodeIdAssigner {
    fn next_id(&mut self) -> NodeId {
        let id = self.id;
        self.id += 1;
        id
    }
}

#[derive(Default)]
struct NodeCreator;

impl NodeCreator {
    // can be called only once, unfortunately implementation via state pattern would be difficult to integrate
    fn create_root(repo: &mut impl NodeRepositoryConcept, id: NodeId) -> Result<NodeId> {
        if repo.nodes().count() > 0 {
            bail!("attempted to create multiple roots");
        }
        if id != 0 {
            bail!("root should always have id 0 but tried to assign {id}");
        }
        repo.create(id)?;
        Ok(id)
    }

    fn create(repo: &mut impl NodeRepositoryConcept, id: NodeId) -> Result<NodeId> {
        repo.create(id)?;
        Ok(id)
    }
}

pub(crate) struct NodeKindService;

impl NodeKindService {
    fn insert(repo: &mut impl NodeKindRepositoryConcept, id: NodeId, kind: NodeKind) -> Result<()> {
        repo.insert(id, kind)
    }

    pub(crate) fn kind(repo: &impl NodeKindRepositoryConcept, id: NodeId) -> Result<NodeKind> {
        repo.kind(id)
            .ok_or_else(|| anyhow!("cannot obtain node kind for node {id}"))
    }
}
