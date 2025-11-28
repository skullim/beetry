use crate::domain::{
    models::{NodeId, NodeKind, NodePosition},
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository, NodeRepositoryConcept,
        NodeRepositoryFacadeConcept, ParamRepositoryConcept,
    },
    service::channel::ChannelService,
};
use anyhow::{Result, anyhow, bail};
use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::ser::node::{ControlSpec, DecoratorSpec, NodeName, NodeSpec, RootSpec};
use delegate::delegate;

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
        self.node_service
            .create_root(self.repo.node_mut().root_mut(), spec)
    }

    pub fn create_action(&mut self, spec: &ActionSpec) -> Result<NodeId> {
        self.node_service
            .create_action(self.repo.node_mut().action_mut(), spec)
    }

    pub fn create_condition(&mut self, spec: &ConditionSpec) -> Result<NodeId> {
        self.node_service
            .create_condition(self.repo.node_mut().condition_mut(), spec)
    }

    pub fn create_control(&mut self, spec: &ControlSpec) -> Result<NodeId> {
        self.node_service
            .create_control(self.repo.node_mut().control_mut(), spec)
    }

    pub fn create_decorator(&mut self, spec: &DecoratorSpec) -> Result<NodeId> {
        self.node_service
            .create_decorator(self.repo.node_mut().decorator_mut(), spec)
    }

    pub fn remove(&mut self, id: NodeId) -> Result<()> {
        let repo = &mut self.repo;

        self.node_service.on_node_removal(repo.node_mut(), id)?;
        repo.edge_mut().on_node_removal(id)?;
        ChannelService::on_node_removal(repo.channel_mut(), id)?;
        repo.parameter_mut().remove(id)
    }

    pub fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        let kind = NodeKindService::kind(id, self.repo.node())?;
        let node_repo = self.repo.node_mut();

        match kind {
            NodeKind::Action => NodePositionUpdater::update(id, node_repo.action_mut(), position)?,
            NodeKind::Condition => {
                NodePositionUpdater::update(id, node_repo.condition_mut(), position)?
            }
            NodeKind::Control => {
                NodePositionUpdater::update(id, node_repo.control_mut(), position)?
            }
            NodeKind::Decorator => {
                NodePositionUpdater::update(id, node_repo.decorator_mut(), position)?
            }
            NodeKind::Root => NodePositionUpdater::update(id, node_repo.root_mut(), position)?,
        };
        Ok(())
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

    pub(crate) fn ensure_exists<N>(&self, id: NodeId, node_facade: &N) -> Result<()>
    where
        N: NodeRepositoryFacadeConcept,
    {
        let kind = Self::kind(id, node_facade)?;

        let contains = match kind {
            NodeKind::Action => node_facade.action().contains(id),
            NodeKind::Condition => node_facade.condition().contains(id),
            NodeKind::Control => node_facade.control().contains(id),
            NodeKind::Decorator => node_facade.decorator().contains(id),
            NodeKind::Root => node_facade.root().contains(id),
        };

        if !contains {
            bail!("node {id} does not exist");
        }
        Ok(())
    }

    delegate! {
        to self.creator {
            fn create_root<N>(&mut self, repo: &mut N, spec: &RootSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = RootSpec>;
            fn create_action<N>(&mut self, repo: &mut N, spec: &ActionSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = ActionSpec>;
            fn create_condition<N>(&mut self, repo: &mut N, spec: &ConditionSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = ConditionSpec>;
            fn create_control<N>(&mut self, repo: &mut N, spec: &ControlSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = ControlSpec>;
            fn create_decorator<N>(&mut self, repo: &mut N, spec: &DecoratorSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = DecoratorSpec>;
        }
    }

    fn on_node_removal<N>(&mut self, repo: &mut N, id: NodeId) -> Result<()>
    where
        N: NodeRepositoryFacadeConcept,
    {
        let kind = Self::kind(id, repo)?;
        match kind {
            NodeKind::Action => repo.action_mut().remove(id)?,
            NodeKind::Condition => repo.condition_mut().remove(id)?,
            NodeKind::Control => repo.control_mut().remove(id)?,
            NodeKind::Decorator => repo.decorator_mut().remove(id)?,
            NodeKind::Root => repo.root_mut().remove(id)?,
        };
        repo.remove_kind(id)
    }

    pub(crate) fn kind<N>(id: NodeId, repo: &N) -> Result<NodeKind>
    where
        N: NodeRepositoryFacadeConcept,
    {
        repo.kind(id)
            .ok_or_else(|| anyhow!("cannot obtain node kind for node {id}"))
    }

    //@todo check if all accessors are really needed
    fn position(&self, id: NodeId) -> Option<&NodePosition> {
        todo!()
    }

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
    fn create_root<N>(&mut self, repo: &mut N, spec: &RootSpec) -> Result<NodeId>
    where
        N: NodeRepositoryConcept<Spec = RootSpec>,
    {
        let id = self.next_id();
        if id != 0 {
            bail!("root should always have id 0 but tried to assign {id}");
        }
        repo.create(id, spec)?;
        Ok(id)
    }

    fn create_action<N>(&mut self, repo: &mut N, spec: &ActionSpec) -> Result<NodeId>
    where
        N: NodeRepositoryConcept<Spec = ActionSpec>,
    {
        let id = self.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn create_condition<N>(&mut self, repo: &mut N, spec: &ConditionSpec) -> Result<NodeId>
    where
        N: NodeRepositoryConcept<Spec = ConditionSpec>,
    {
        let id = self.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn create_decorator<N>(&mut self, repo: &mut N, spec: &DecoratorSpec) -> Result<NodeId>
    where
        N: NodeRepositoryConcept<Spec = DecoratorSpec>,
    {
        let id = self.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn create_control<N>(&mut self, repo: &mut N, spec: &ControlSpec) -> Result<NodeId>
    where
        N: NodeRepositoryConcept<Spec = ControlSpec>,
    {
        let id = self.next_id();
        repo.create(id, spec)?;
        Ok(id)
    }

    fn next_id(&mut self) -> NodeId {
        self.id_assigner.next_id()
    }
}

struct NodePositionUpdater;

impl NodePositionUpdater {
    fn update<N, S>(id: NodeId, repo: &mut N, position: NodePosition) -> Result<()>
    where
        N: NodeRepositoryConcept<Spec = S>,
    {
        repo.update_position(id, position)
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
    fn kind<N>(id: NodeId, repo: &N) -> Result<NodeKind>
    where
        N: NodeRepositoryFacadeConcept,
    {
        repo.kind(id)
            .ok_or_else(|| anyhow!("cannot obtain node kind for node {id}"))
    }
}
