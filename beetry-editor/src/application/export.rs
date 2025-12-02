use std::collections::HashSet;

use anyhow::{Result, anyhow, bail};
use beetry_serde::{
    de::node::{ControlSnapshot, LeafSnapshot, NodeSnapshot, NodeSnapshotData, RootSnapshot},
    ser::node::LeafKind,
};

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
            NodeKind::Action => self.export_leaf(id, LeafKind::Action),
            NodeKind::Condition => self.export_leaf(id, LeafKind::Condition),

            NodeKind::Decorator => {
                unimplemented!()
            }
            _ => {
                todo!()
            }
        }
    }

    fn export_leaf(&mut self, id: NodeId, kind: LeafKind) -> Result<NodeSnapshot> {
        let (mut expected_receivers, mut expected_senders): (HashSet<_>, HashSet<_>) = {
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

        let channel_view = self.service.channel_view();
        let connected_senders: Vec<_> = channel_view
            .senders(id)
            .map(|channel_id| channel_view.spec(channel_id))
            .collect::<Result<Vec<_>>>()?;

        for sender in connected_senders {
            if !expected_senders.remove(sender.msg_hash()) {
                bail!(
                    "found unexpected sender {} for leaf node {id}",
                    sender.as_str()
                );
            }
        }
        if !expected_senders.is_empty() {
            bail!("found unconnected senders: {expected_senders:?} for leaf node {id}");
        }

        let connected_receivers: Vec<_> = channel_view
            .receivers(id)
            .map(|channel_id| channel_view.spec(channel_id))
            .collect::<Result<Vec<_>>>()?;

        for receiver in connected_receivers {
            if !expected_receivers.remove(receiver.msg_hash()) {
                bail!(
                    "found unexpected receiver {} for leaf node {id}",
                    receiver.as_str()
                );
            }
        }
        if !expected_receivers.is_empty() {
            bail!("found unconnected receivers: {expected_receivers:?} for leaf node {id}");
        }

        //@todo add external senders/receivers
        let leaf_snapshot = LeafSnapshot::builder()
            .kind(kind)
            .receivers(channel_view.receivers(id))
            .senders(channel_view.senders(id))
            .build();

        let node_view = self.service.node_view();
        let name = node_view.name(id)?;
        let params = node_view.parameters(id)?;

        Ok(NodeSnapshot::builder()
            .name(name.clone())
            .data(leaf_snapshot)
            .parameters(params.clone())
            .build())
    }
}
