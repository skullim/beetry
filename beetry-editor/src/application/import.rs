use std::collections::{HashMap, VecDeque};

use crate::{
    EditorService,
    domain::{
        models::{NodeId, NodeKind},
        persistence::EditorData,
        ports::{ChannelRepositoryConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept},
        service::node::{AssignNodeId, NodeService},
    },
};
use anyhow::{Result, anyhow};

pub struct ProjectImporter;

impl ProjectImporter {
    pub fn import<NRF, ER, CR>(
        data: EditorData,
    ) -> Result<EditorService<NRF, ER, CR, ImportNodeIdAssigner>>
    where
        NRF: NodeRepositoryFacadeConcept,
        ER: EdgeRepositoryConcept,
        CR: ChannelRepositoryConcept,
    {
        // nodes are ordered when stored and this fact is used to assign the ids that will be used during import
        let node_id_assigner = {
            let imported_node_ids: VecDeque<NodeId> =
                data.tree.nodes.iter().map(|node| node.id).collect();
            let next = imported_node_ids.iter().max().copied().unwrap_or_default();
            ImportNodeIdAssigner::new(imported_node_ids, next)
        };

        let mut editor_service =
            EditorService::<NRF, ER, CR, ImportNodeIdAssigner>::with_node_service(
                NodeService::with_assigner(node_id_assigner),
            );
        // first import nodes
        {
            let spec_lookup: HashMap<_, _> = data
                .tree
                .node_metadata
                .into_iter()
                .map(|m| (m.id, (m.spec, m.kind, m.port_ids)))
                .collect();
            let subsequent_node_ids = data.tree.nodes.iter().map(|node| node.id);
            for node in data.tree.nodes {
                let meta = spec_lookup
                    .get(&node.metadata_id)
                    .ok_or_else(|| anyhow!("failed to obtain metadata for node {}", node.id))?;
                let (spec, kind, ports) = meta;

                let mut node_view = editor_service.node_view();
            }
        }
        todo!()
    }
}

pub(crate) struct ImportNodeIdAssigner {
    imported_node_ids: VecDeque<NodeId>,
    next: NodeId,
}

impl ImportNodeIdAssigner {
    fn new(imported_node_ids: VecDeque<NodeId>, next: NodeId) -> Self {
        Self {
            imported_node_ids,
            next,
        }
    }
}

impl AssignNodeId for ImportNodeIdAssigner {
    fn next_id(&mut self) -> NodeId {
        if let Some(next) = self.imported_node_ids.pop_front() {
            return next;
        }
        let id = self.next;
        self.next += 1;
        id
    }
}

//@todo implement in next release
struct SubtreeImporter;
