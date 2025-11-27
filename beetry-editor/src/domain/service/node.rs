use crate::domain::{
    models::{NodeId, NodeKind, NodePosition},
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeKindRepositoryConcept, NodeRepositoryConcept, ParamRepositoryConcept,
    },
};
use anyhow::{Result, anyhow, bail};
use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::ser::node::{ControlSpec, DecoratorSpec, NodeName, NodeSpec, RootSpec};
use delegate::delegate;

pub struct NodeServiceApi<'r, 's, ER, KR, CR, PR> {
    repo: &'r mut EditorRepository<ER, KR, CR, PR>,
    service: &'s mut NodeService,
}

impl<'r, 's, ER, KR, CR, PR> NodeServiceApi<'r, 's, ER, KR, CR, PR>
where
    ER: EdgeRepositoryConcept,
    KR: NodeKindRepositoryConcept,
    CR: ChannelRepositoryConcept,
    PR: ParamRepositoryConcept,
{
    pub fn new(
        repo: &'r mut EditorRepository<ER, KR, CR, PR>,
        service: &'s mut NodeService,
    ) -> Self {
        Self { repo, service }
    }

    pub fn create_root(&mut self, spec: &RootSpec) -> Result<NodeId> {
        self.service
            .create_root(self.repo.node_mut().root_mut(), spec)
    }

    pub fn create_action(&mut self, spec: &ActionSpec) -> Result<NodeId> {
        self.service
            .create_action(self.repo.node_mut().action_mut(), spec)
    }

    pub fn create_condition(&mut self, spec: &ConditionSpec) -> Result<NodeId> {
        self.service
            .create_condition(self.repo.node_mut().condition_mut(), spec)
    }

    pub fn create_control(&mut self, spec: &ControlSpec) -> Result<NodeId> {
        self.service
            .create_control(self.repo.node_mut().control_mut(), spec)
    }

    pub fn create_decorator(&mut self, spec: &DecoratorSpec) -> Result<NodeId> {
        self.service
            .create_decorator(self.repo.node_mut().decorator_mut(), spec)
    }

    pub fn remove(&mut self, id: NodeId) -> Result<()> {
        let kind = NodeKindService::kind(id, self.repo.kind())?;
        let service = &mut self.service;
        let node_repo = self.repo.node_mut();

        match kind {
            NodeKind::Action => service.remove(node_repo.action_mut(), id)?,
            NodeKind::Condition => service.remove(node_repo.condition_mut(), id)?,
            NodeKind::Control => service.remove(node_repo.control_mut(), id)?,
            NodeKind::Decorator => service.remove(node_repo.decorator_mut(), id)?,
            NodeKind::Root => service.remove(node_repo.root_mut(), id)?,
        };

        let repo = &mut self.repo;
        repo.edge_mut().on_node_removal(id)?;
        repo.kind_mut().on_node_removal(id)?;
        repo.channel_mut().on_node_removal(id)?;
        repo.parameter_mut().on_node_removal(id)
    }

    pub fn update_position(&mut self, id: NodeId, position: NodePosition) -> Result<()> {
        let kind = NodeKindService::kind(id, self.repo.kind())?;
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

    delegate! {
        to self.creator {
            fn create_root<N>(&mut self, repo: &mut N, spec: &RootSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = RootSpec>;
            fn create_action<N>(&mut self, repo: &mut N, spec: &ActionSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = ActionSpec>;
            fn create_condition<N>(&mut self, repo: &mut N, spec: &ConditionSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = ConditionSpec>;
            fn create_control<N>(&mut self, repo: &mut N, spec: &ControlSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = ControlSpec>;
            fn create_decorator<N>(&mut self, repo: &mut N, spec: &DecoratorSpec) -> Result<NodeId> where N: NodeRepositoryConcept<Spec = DecoratorSpec>;
        }
    }

    fn remove<N, S>(&mut self, repo: &mut N, id: NodeId) -> Result<()>
    where
        N: NodeRepositoryConcept<Spec = S>,
    {
        repo.remove(id)
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

struct NodeKindService;

impl NodeKindService {
    fn kind<K>(id: NodeId, repo: &K) -> Result<NodeKind>
    where
        K: NodeKindRepositoryConcept,
    {
        repo.kind(id)
            .ok_or_else(|| anyhow!("cannot obtain node kind for node {id}"))
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
