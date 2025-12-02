use crate::domain::{
    models::{NodeId, NodeKind, NodePosition},
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeKindRepositoryConcept, NodePositionRepositoryConcept, NodeRepositoryConcept,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut,
        ParamRepositoryConcept,
    },
    service::{channel::ChannelService, edge::EdgeService},
};
use anyhow::{Result, anyhow, bail};
use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::{
    de::parameter::Parameters,
    ser::node::{ControlSpec, DecoratorSpec, NodeName, NodeSpec, RootSpec},
};

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeServiceView<'r, 's, 'e, NRF, ER, CR> {
    repo: &'r mut EditorRepository<NRF, ER, CR>,
    node_service: &'s mut NodeService,
    edge_service: &'e mut EdgeService,
}

impl<'r, 's, 'e, NRF, ER, CR> NodeServiceView<'r, 's, 'e, NRF, ER, CR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
{
    pub(crate) fn new(
        repo: &'r mut EditorRepository<NRF, ER, CR>,
        node_service: &'s mut NodeService,
        edge_service: &'e mut EdgeService,
    ) -> Self {
        Self {
            repo,
            node_service,
            edge_service,
        }
    }

    pub fn create_root(&mut self, spec: &RootSpec) -> Result<NodeId> {
        let NodeRepositoryFacadeViewMut { root, kinds, .. } = self.repo.node_mut().view_mut();
        self.node_service.create_root(root, kinds, spec)
    }

    pub fn create_action(&mut self, spec: &ActionSpec) -> Result<NodeId> {
        let NodeRepositoryFacadeViewMut { action, kinds, .. } = self.repo.node_mut().view_mut();
        self.node_service.create_action(action, kinds, spec)
    }

    pub fn create_condition(&mut self, spec: &ConditionSpec) -> Result<NodeId> {
        let NodeRepositoryFacadeViewMut {
            condition, kinds, ..
        } = self.repo.node_mut().view_mut();
        self.node_service.create_condition(condition, kinds, spec)
    }

    pub fn create_control(&mut self, spec: &ControlSpec) -> Result<NodeId> {
        let NodeRepositoryFacadeViewMut { control, kinds, .. } = self.repo.node_mut().view_mut();
        self.node_service.create_control(control, kinds, spec)
    }

    pub fn create_decorator(&mut self, spec: &DecoratorSpec) -> Result<NodeId> {
        let NodeRepositoryFacadeViewMut {
            decorator, kinds, ..
        } = self.repo.node_mut().view_mut();
        self.node_service.create_decorator(decorator, kinds, spec)
    }

    pub fn remove(&mut self, id: NodeId) -> Result<()> {
        let repo = &mut self.repo;
        NodeService::on_node_removal(repo.node_mut(), id)?;
        self.edge_service.on_node_removal(repo.edge_mut(), id)?;
        ChannelService::on_node_removal(repo.channel_mut(), id)
    }

    pub fn root_spec(&self, id: NodeId) -> Option<&RootSpec> {
        let NodeRepositoryFacadeView { root, .. } = self.repo.node().view();
        NodeService::root_spec(root, id)
    }

    pub fn action_spec(&self, id: NodeId) -> Option<&ActionSpec> {
        let NodeRepositoryFacadeView { action, .. } = self.repo.node().view();
        NodeService::action_spec(action, id)
    }

    pub fn condition_spec(&self, id: NodeId) -> Option<&ConditionSpec> {
        let NodeRepositoryFacadeView { condition, .. } = self.repo.node().view();
        NodeService::condition_spec(condition, id)
    }

    pub fn control_spec(&self, id: NodeId) -> Option<&ControlSpec> {
        let NodeRepositoryFacadeView { control, .. } = self.repo.node().view();
        NodeService::control_spec(control, id)
    }

    pub fn decorator_spec(&self, id: NodeId) -> Option<&DecoratorSpec> {
        let NodeRepositoryFacadeView { decorator, .. } = self.repo.node().view();
        NodeService::decorator_spec(decorator, id)
    }

    pub fn name(&self, id: NodeId) -> Result<&NodeName> {
        NodeService::name(self.repo.node(), id)
    }

    pub fn kind(&self, id: NodeId) -> Result<NodeKind> {
        let NodeRepositoryFacadeView { kinds, .. } = self.repo.node().view();
        NodeKindService::kind(kinds, id)
    }

    pub fn nodes(&self, kind: NodeKind) -> impl Iterator<Item = NodeId> {
        NodeService::nodes(self.repo.node(), kind)
    }

    pub fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        NodeService::update_position(self.repo.node_mut(), id, position)
    }

    pub fn positions(&self, kind: NodeKind) -> impl Iterator<Item = &NodePosition> {
        NodeService::positions(self.repo.node(), kind)
    }

    pub fn parameters(&self, id: NodeId) -> Result<&Parameters> {
        NodeService::parameters(self.repo.node(), id)
    }
}

pub struct NodeService {
    creator: NodeCreator,
}

impl NodeService {
    pub fn new() -> Self {
        Self {
            creator: NodeCreator::new(),
        }
    }

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

    fn create_root(
        &mut self,
        root_repo: &mut impl NodeRepositoryConcept<Spec = RootSpec>,
        kinds_repo: &mut impl NodeKindRepositoryConcept,
        spec: &RootSpec,
    ) -> Result<NodeId> {
        let id = self.creator.create_root(root_repo, spec)?;
        NodeKindService::insert(kinds_repo, id, NodeKind::Root)?;
        Ok(id)
    }

    fn create_action(
        &mut self,
        actions_repo: &mut impl NodeRepositoryConcept<Spec = ActionSpec>,
        kinds_repo: &mut impl NodeKindRepositoryConcept,
        spec: &ActionSpec,
    ) -> Result<NodeId> {
        let id = self.creator.create_action(actions_repo, spec)?;
        NodeKindService::insert(kinds_repo, id, NodeKind::Action)?;
        Ok(id)
    }

    fn create_condition(
        &mut self,
        conditions_repo: &mut impl NodeRepositoryConcept<Spec = ConditionSpec>,
        kinds_repo: &mut impl NodeKindRepositoryConcept,
        spec: &ConditionSpec,
    ) -> Result<NodeId> {
        let id = self.creator.create_condition(conditions_repo, spec)?;
        NodeKindService::insert(kinds_repo, id, NodeKind::Condition)?;
        Ok(id)
    }

    fn create_control(
        &mut self,
        controls_repo: &mut impl NodeRepositoryConcept<Spec = ControlSpec>,
        kinds_repo: &mut impl NodeKindRepositoryConcept,
        spec: &ControlSpec,
    ) -> Result<NodeId> {
        let id = self.creator.create_control(controls_repo, spec)?;
        NodeKindService::insert(kinds_repo, id, NodeKind::Control)?;
        Ok(id)
    }

    fn create_decorator(
        &mut self,
        decorators_repo: &mut impl NodeRepositoryConcept<Spec = DecoratorSpec>,
        kinds_repo: &mut impl NodeKindRepositoryConcept,
        spec: &DecoratorSpec,
    ) -> Result<NodeId> {
        let id = self.creator.create_decorator(decorators_repo, spec)?;
        NodeKindService::insert(kinds_repo, id, NodeKind::Decorator)?;
        Ok(id)
    }

    fn root_spec(
        repo: &impl NodeRepositoryConcept<Spec = RootSpec>,
        id: NodeId,
    ) -> Option<&RootSpec> {
        repo.spec(id)
    }
    fn action_spec(
        repo: &impl NodeRepositoryConcept<Spec = ActionSpec>,
        id: NodeId,
    ) -> Option<&ActionSpec> {
        repo.spec(id)
    }
    fn condition_spec(
        repo: &impl NodeRepositoryConcept<Spec = ConditionSpec>,
        id: NodeId,
    ) -> Option<&ConditionSpec> {
        repo.spec(id)
    }
    fn control_spec(
        repo: &impl NodeRepositoryConcept<Spec = ControlSpec>,
        id: NodeId,
    ) -> Option<&ControlSpec> {
        repo.spec(id)
    }
    fn decorator_spec(
        repo: &impl NodeRepositoryConcept<Spec = DecoratorSpec>,
        id: NodeId,
    ) -> Option<&DecoratorSpec> {
        repo.spec(id)
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
        let view = repo.view();
        let kind = NodeKindService::kind(view.kinds, id)?;
        let on_error = || anyhow!("no name found for node {id}");

        Ok(match kind {
            NodeKind::Action => view.action.spec(id).ok_or_else(on_error)?.name(),
            NodeKind::Condition => view.condition.spec(id).ok_or_else(on_error)?.name(),
            NodeKind::Control => view.control.spec(id).ok_or_else(on_error)?.name(),
            NodeKind::Decorator => view.decorator.spec(id).ok_or_else(on_error)?.name(),
            NodeKind::Root => view.root.spec(id).ok_or_else(on_error)?.name(),
        })
    }
}

#[derive(Default)]
struct NodeCreator {
    id_assigner: NodeIdAssigner,
}

impl NodeCreator {
    fn new() -> Self {
        Self::default()
    }

    // can be called only once, unfortunately implementation via state pattern would be difficult to integrate
    fn create_root(
        &mut self,
        repo: &mut impl NodeRepositoryConcept<Spec = RootSpec>,
        spec: &RootSpec,
    ) -> Result<NodeId> {
        if repo.nodes().count() > 0 {
            bail!("attempted to create multiple roots");
        }
        let id = self.next_id();
        if id != 0 {
            bail!("root should always have id 0 but tried to assign {id}");
        }
        repo.create(id, spec)?;
        Ok(id)
    }

    fn create_action(
        &mut self,
        repo: &mut impl NodeRepositoryConcept<Spec = ActionSpec>,
        spec: &ActionSpec,
    ) -> Result<NodeId> {
        let id = self.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn create_condition(
        &mut self,
        repo: &mut impl NodeRepositoryConcept<Spec = ConditionSpec>,
        spec: &ConditionSpec,
    ) -> Result<NodeId> {
        let id = self.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn create_decorator(
        &mut self,
        repo: &mut impl NodeRepositoryConcept<Spec = DecoratorSpec>,
        spec: &DecoratorSpec,
    ) -> Result<NodeId> {
        let id = self.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn create_control(
        &mut self,
        repo: &mut impl NodeRepositoryConcept<Spec = ControlSpec>,
        spec: &ControlSpec,
    ) -> Result<NodeId> {
        let id = self.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn next_id(&mut self) -> NodeId {
        self.id_assigner.next_id()
    }
}

#[derive(Default)]
struct NodeIdAssigner {
    id: NodeId,
}

impl NodeIdAssigner {
    fn next_id(&mut self) -> NodeId {
        let id = self.id;
        self.id += 1;
        id
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
