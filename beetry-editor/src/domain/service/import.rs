use crate::{
    EditorService,
    domain::{
        persistence::EditorData,
        repository::{
            ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept,
        },
    },
};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

pub struct ProjectImportServiceApi;

impl ProjectImportServiceApi {
    pub fn import<NRF, ER, CRF>(data: EditorData) -> Result<EditorService<NRF, ER, CRF>>
    where
        NRF: NodeRepositoryFacadeConcept,
        ER: EdgeRepositoryConcept,
        CRF: ChannelRepositoryFacadeConcept,
    {
        let mut editor_service = EditorService::<NRF, ER, CRF>::default();
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
            }
        }
        todo!()
    }
}

//@todo implement in next release
struct SubtreeImporter;
