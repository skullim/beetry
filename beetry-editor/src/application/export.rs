use std::collections::HashSet;

use anyhow::{Result, anyhow};
use beetry_plugin::node;
use beetry_serde::de::node::{ControlSnapshot, NodeSnapshot, NodeSnapshotData, RootSnapshot};

use crate::{
    EditorService,
    domain::{
        models::{NodeId, NodeKind},
        ports::{ChannelRepositoryConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept},
    },
};

pub struct TreeExporter<'a, NRF, ER, CR> {
    //@todo long term split API into mut and shared. Export should be possible using only shared reference
    service: &'a mut EditorService<NRF, ER, CR>,
}

impl<'a, NRF, ER, CR> TreeExporter<'a, NRF, ER, CR>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CR: ChannelRepositoryConcept,
{
    fn export_root(&mut self) -> Result<RootSnapshot> {
        let root_id = self
            .service
            .node_view()
            .nodes(NodeKind::Root)
            .next()
            .ok_or_else(|| anyhow!("no root found in the tree"))?;

        let child_id = self
            .service
            .edge_view()
            .children_of(root_id)
            .next()
            .ok_or_else(|| anyhow!("root has no child"))?;
        let child = self.export_node(child_id)?;
        Ok(RootSnapshot::new(child))
    }

    fn export_node(&mut self, id: NodeId) -> Result<NodeSnapshot> {
        let kind: NodeKind = self.service.node_view().kind(id)?;
        match kind {
            NodeKind::Root => unreachable!(),
            NodeKind::Control => {
                let child_ids: Vec<_> = self.service.edge_view().children_of(id).collect();
                //@todo: child_ids have to be sorted based on the x coordinate (increasing) to determine the proper children order
                let mut children = Vec::with_capacity(child_ids.len());
                for child_id in child_ids {
                    children.push(self.export_node(child_id)?);
                }

                Ok(NodeSnapshot::builder()
                    .name(self.service.node_view().name(id)?.clone())
                    .data(NodeSnapshotData::Control(ControlSnapshot::new(children)?))
                    .build())
            }
            NodeKind::Action => {
                let (expected_receivers, expected_senders): (HashSet<_>, HashSet<_>) = {
                    let node_view = self.service.node_view();
                    let spec = node_view
                        .action_spec(id)
                        .ok_or_else(|| anyhow!("no spec found for action node {id}"))?;

                    (
                        spec.schema
                            .receivers
                            .iter()
                            .map(|item| item.hash())
                            .copied()
                            .collect(),
                        spec.schema
                            .senders
                            .iter()
                            .map(|item| item.hash())
                            .copied()
                            .collect(),
                    )
                };

                todo!()
            }

            NodeKind::Decorator => {
                unimplemented!()
            }
            _ => {
                todo!()
            }
        }
    }
}
