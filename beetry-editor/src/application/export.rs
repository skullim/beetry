use anyhow::{Result, anyhow, bail};
use beetry_serde::{
    de::{
        channel::{ChannelIdToSnapshotMap, ChannelSnapshot2},
        node::{ControlSnapshot, LeafSnapshot, NodeSnapshot, NodeSnapshotData, RootSnapshot},
        tree::TreeSnapshot,
    },
    ser::node::LeafKind,
};
use itertools::izip;
use std::collections::HashMap;

use crate::{
    EditorService,
    domain::{
        models::{NodeId, NodeKind, NodePortConnection, NodePortKind},
        ports::{ChannelRepositoryConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept},
    },
};

//@todo move to service layer, there should be no application layer
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
    pub fn export(&mut self) -> Result<TreeSnapshot> {
        let root = self.export_root()?;
        Ok(TreeSnapshot::builder()
            .root(root)
            .channels(self.export_channels()?)
            .build()?)
    }

    fn export_channels(&mut self) -> Result<ChannelIdToSnapshotMap> {
        let view = self.service.channel_view();
        let ids: Vec<_> = view.channels().collect();
        let spec = ids
            .iter()
            .map(|id| view.spec(*id))
            .collect::<Result<Vec<_>>>()?;
        let metadata = ids
            .iter()
            .map(|id| view.parameters(*id))
            .collect::<Result<Vec<_>>>()?;

        let map = izip!(ids, spec, metadata)
            .map(|(id, spec, meta)| (id, ChannelSnapshot2::new(spec.clone(), meta.clone())))
            .collect::<HashMap<_, _>>();

        todo!()
    }

    fn export_root(&mut self) -> Result<RootSnapshot> {
        let root_id = self
            .service
            .node_view()
            .nodes_by_kind(NodeKind::Root)
            .next()
            .copied()
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
        }
    }

    fn export_leaf(&mut self, id: NodeId, kind: LeafKind) -> Result<NodeSnapshot> {
        let node_view = self.service.node_view();
        let port_ids = node_view.port_ids(id);

        for port_id in port_ids {
            let conn = node_view.port_connection(id, port_id)?;
            if let NodePortConnection::Internal(connected) = conn
                && connected.is_empty()
            {
                bail!("unconnected internal port ({port_id}) of node {id}");
            }
        }

        let mut senders = vec![];
        let mut receivers = vec![];
        for port_id in node_view.port_ids(id) {
            let spec = node_view.port_spec(id, port_id)?;
            let conn = node_view.port_connection(id, port_id)?;
            if let NodePortConnection::Internal(channels) = conn {
                match spec.kind {
                    NodePortKind::Sender => {
                        senders.extend(channels);
                    }
                    NodePortKind::Receiver => {
                        receivers.extend(channels);
                    }
                }
            }
        }

        //@todo add external senders/receivers
        let leaf_snapshot = LeafSnapshot::builder()
            .kind(kind)
            .receivers(receivers)
            .senders(senders)
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
