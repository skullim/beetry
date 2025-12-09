use std::collections::{HashMap, VecDeque};

use crate::{
    EditorService,
    domain::{
        models::NodeId,
        persistence::EditorData,
        ports::{ChannelDataRepositoryConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept},
        service::node::NodeService,
    },
};
use anyhow::{Result, anyhow};

pub struct ProjectImporter;

impl ProjectImporter {
    pub fn import<NRF, ER, CR>(data: EditorData) -> Result<EditorService<NRF, ER, CR>>
    where
        NRF: NodeRepositoryFacadeConcept,
        ER: EdgeRepositoryConcept,
        CR: ChannelDataRepositoryConcept,
    {
        let mut editor_service = EditorService::<NRF, ER, CR>::default();
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

//@todo implement in next release
struct SubtreeImporter;
