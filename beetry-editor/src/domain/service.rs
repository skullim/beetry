use super::models::NodeId;
use crate::domain::{
    models::{NodeKind, NodePosition},
    ports::{
        ChannelRepository, EdgeRepository, ExternalPortRepository, NodeRepository, ParamRepository,
        SpecRepository,
    },
};
use anyhow::{Result, bail};
use beetry_plugin::{ActionSpec, ConditionSpec};
use beetry_serde::ser::node::{ControlSpec, DecoratorSpec, NodeName, NodeSpec, RootSpec};
use delegate::delegate;

// storage for all the repositories and services
// maybe move to different file
pub struct EditorRepository<NR, ER, SR, EPR, CR, PR> {
    node_repo: NR,
    edge_repo: ER,
    spec_repo: SR,
    external_port_repo: EPR,
    channel_repo: CR,
    parameter_repo: PR,
}

pub struct EditorService<NR, ER, SR, EPR, CR, PR> {
    node: NodeService,
    repo: EditorRepository<NR, ER, SR, EPR, CR, PR>,
}

impl<NR, ER, SR, EPR, CR, PR> EditorService<NR, ER, SR, EPR, CR, PR>
where
    NR: NodeRepository,
    ER: EdgeRepository,
    SR: SpecRepository,
    EPR: ExternalPortRepository,
    CR: ChannelRepository,
    PR: ParamRepository,
{
    pub fn new(repo: EditorRepository<NR, ER, SR, EPR, CR, PR>) -> Self {
        Self {
            node: NodeService::new(),
            repo,
        }
    }

    pub fn create_root(&mut self) -> Result<NodeId> {
        self.node.create_root(&mut self.repo.node_repo)
    }

    pub fn create_action(&mut self) -> Result<NodeId> {
        self.node.create_action(&mut self.repo.node_repo)
    }

    pub fn create_condition(&mut self) -> Result<NodeId> {
        self.node.create_condition(&mut self.repo.node_repo)
    }

    pub fn create_control(&mut self) -> Result<NodeId> {
        self.node.create_control(&mut self.repo.node_repo)
    }

    pub fn remove(&mut self, id: NodeId) -> Result<()> {
        self.node.remove(&mut self.repo.node_repo, id)
    }

    delegate! {
        to NodePositionUpdater {
            #[call(update)]
            fn update_position<N>(id: NodeId, repo: &mut N, position: NodePosition) -> Result<()> where N: NodeRepository;
        }
    }
}

struct NodeService {
    creator: NodeCreator,
}

impl NodeService {
    fn new() -> Self {
        Self {
            creator: NodeCreator::new(),
        }
    }

    delegate! {
        to self.creator {
            fn create_root<N>(&mut self, repo: &mut N) -> Result<NodeId> where N: NodeRepository;
            fn create_action<N>(&mut self, repo: &mut N) -> Result<NodeId> where N: NodeRepository;
            fn create_condition<N>(&mut self, repo: &mut N) -> Result<NodeId> where N: NodeRepository;
            fn create_control<N>(&mut self, repo: &mut N) -> Result<NodeId> where N: NodeRepository;
            fn create_decorator<N>(&mut self, repo: &mut N) -> Result<NodeId> where N: NodeRepository;
        }
    }

    fn remove<N>(&mut self, repo: &mut N, id: NodeId) -> Result<()>
    where
        N: NodeRepository,
    {
        repo.remove(id)
    }

    //@todo check if all accessors are really needed
    fn kind(&self, id: NodeId) -> Option<NodeKind> {
        todo!()
    }

    fn position(&self, id: NodeId) -> Option<&NodePosition> {
        todo!()
    }

    fn name(&self, id: NodeId) -> Option<&NodeName> {
        todo!()
    }

    fn spec<T>(&self, id: NodeId) -> Option<&NodeSpec<T>> {
        todo!()
    }

    fn nodes(&self) -> &[NodeId] {
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
    fn create_root<N>(&mut self, repo: &mut N) -> Result<NodeId>
    where
        N: NodeRepository,
    {
        let id = self.next_id();
        if id != 0 {
            bail!("root should always have id 0 but tried to assign {id}");
        }
        repo.create_root(id)?;
        Ok(id)
    }

    fn create_action<N>(&mut self, repo: &mut N) -> Result<NodeId>
    where
        N: NodeRepository,
    {
        let id = self.next_id();
        repo.create_action(id)?;
        Ok(id)
    }

    fn create_condition<N>(&mut self, repo: &mut N) -> Result<NodeId>
    where
        N: NodeRepository,
    {
        let id = self.next_id();
        repo.create_condition(id)?;
        Ok(id)
    }

    fn create_decorator<N>(&mut self, repo: &mut N) -> Result<NodeId>
    where
        N: NodeRepository,
    {
        let id = self.next_id();
        repo.create_decorator(id)?;
        Ok(id)
    }

    fn create_control<N>(&mut self, repo: &mut N) -> Result<NodeId>
    where
        N: NodeRepository,
    {
        let id = self.next_id();
        repo.create_control(id)?;
        Ok(id)
    }

    fn next_id(&mut self) -> NodeId {
        self.id_assigner.next_id()
    }
}

struct NodePositionUpdater;

impl NodePositionUpdater {
    fn update<N>(id: NodeId, repo: &mut N, position: NodePosition) -> Result<()>
    where
        N: NodeRepository,
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

struct EdgeService;
struct ChannelService;
struct ParameterService;
struct StorageService;
