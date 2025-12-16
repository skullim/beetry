use crate::{
    EditorService,
    domain::{
        persistence::EditorStorage,
        repository::{
            ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept,
        },
    },
};
use anyhow::{Result, anyhow};
use beetry_serde::de::tree::TreeSnapshot;
use std::collections::HashMap;

pub struct ImportServiceApi;

impl ImportServiceApi {
    pub fn import_project<NRF, ER, CRF>(data: EditorStorage) -> Result<EditorService<NRF, ER, CRF>>
    where
        NRF: NodeRepositoryFacadeConcept,
        ER: EdgeRepositoryConcept,
        CRF: ChannelRepositoryFacadeConcept,
    {
        let mut editor_service = EditorService::<NRF, ER, CRF>::default();
        // first import nodes
        {
            // let spec_lookup: HashMap<_, _> = data
            //     .tree
            //     .node_specs
            //     .into_iter()
            //     .map(|m| (m.id, (m.spec, m.kind, m.port_ids)))
            //     .collect();
            // let subsequent_node_ids = data.tree.nodes.iter().map(|node| node.id);
            // for node in data.tree.nodes {
            //     let meta = spec_lookup
            //         .get(&node.spec_id)
            //         .ok_or_else(|| anyhow!("failed to obtain metadata for node {}", node.id))?;
            //     let (spec, kind, ports) = meta;
            // }
        }
        todo!()
    }

    pub fn import_tree() -> Result<TreeSnapshot> {
        todo!()
    }
}

//@todo implement in next release
struct SubtreeImporter;
