use crate::domain::{
    models::{NodeId, NodeKind, NodePosition},
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeKindRepositoryConcept, NodePositionRepositoryConcept, NodeRepositoryConcept,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeView, NodeRepositoryFacadeViewMut,
        ParamRepositoryConcept,
    },
    service::channel::ChannelService,
};
use anyhow::{Result, anyhow, bail};
use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::ser::node::{ControlSpec, DecoratorSpec, NodeName, NodeSpec, RootSpec};

/// User-facing API, internally this layer maps the concrete repository to corresponding service
pub struct NodeServiceView<'r, 's, NRF, ER, CR, PR> {
    repo: &'r mut EditorRepository<NRF, ER, CR, PR>,
    node_service: &'s mut NodeService,
}

impl<'r, 's, NRF, ER, CR, PR> NodeServiceView<'r, 's, NRF, ER, CR, PR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
    PR: ParamRepositoryConcept,
{
    pub(crate) fn new(
        repo: &'r mut EditorRepository<NRF, ER, CR, PR>,
        node_service: &'s mut NodeService,
    ) -> Self {
        Self { repo, node_service }
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
        self.node_service.on_node_removal(repo.node_mut(), id)?;
        repo.edge_mut().on_node_removal(id)?;
        ChannelService::on_node_removal(repo.channel_mut(), id)?;
        repo.parameter_mut().remove(id)
    }

    pub fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        NodeService::update_position(self.repo.node_mut(), id, position)
    }

    pub fn positions(&self, kind: NodeKind) -> impl Iterator<Item = &NodePosition> {
        NodeService::positions(self.repo.node(), kind)
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
        node_facade: &impl NodeRepositoryFacadeConcept,
        id: NodeId,
    ) -> Result<()> {
        let view = node_facade.view();
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
        let id = self.creator.create_action(conditions_repo, spec)?;
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

    fn update_position(
        repo: &mut impl NodeRepositoryFacadeConcept,
        id: NodeId,
        position: NodePosition,
    ) -> Result<()> {
        Self::ensure_exists(repo, id)?;
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

    fn on_node_removal(
        &mut self,
        repo: &mut impl NodeRepositoryFacadeConcept,
        id: NodeId,
    ) -> Result<()> {
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
        view.positions.remove(id)
    }

    //@todo check if all accessors are really needed
    fn name(&self, id: NodeId) -> Option<&NodeName> {
        todo!()
    }

    fn spec<T>(&self, id: NodeId) -> Option<&NodeSpec<T>> {
        todo!()
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

struct NodeKindService;

impl NodeKindService {
    fn insert(repo: &mut impl NodeKindRepositoryConcept, id: NodeId, kind: NodeKind) -> Result<()> {
        repo.insert(id, kind)
    }

    fn kind(repo: &impl NodeKindRepositoryConcept, id: NodeId) -> Result<NodeKind> {
        repo.kind(id)
            .ok_or_else(|| anyhow!("cannot obtain node kind for node {id}"))
    }
}
