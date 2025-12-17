use anyhow::{Result, anyhow, bail};
use beetry_plugin_types::node::LeafKind;
use beetry_reconstruction_types::{
    channel::{ChannelSnapshot2, ChannelSnapshotMap},
    node::{ControlSnapshot, LeafSnapshot, NodeSnapshot, NodeSnapshotData, RootSnapshot},
    tree::TreeSnapshot,
};

use itertools::izip;
use std::collections::HashMap;

use crate::domain::{
    models::{NodeId, NodeKind, NodePortConnection, NodePortKind},
    repository::{
        ChannelRepositoryFacadeConcept, EdgeRepositoryConcept, NodeRepositoryFacadeConcept,
    },
    service::{channel::ChannelServiceApi, edge::EdgeServiceApi, node::NodeServiceApi},
};

pub struct TreeExportServiceApi<'a, NRF, ER, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    channel_api: ChannelServiceApi<'a, CRF>,
    node_api: NodeServiceApi<'a, NRF, ER, CRF>,
    edge_api: EdgeServiceApi<'a, ER, NRF>,
}

impl<'a, NRF, ER, CRF> TreeExportServiceApi<'a, NRF, ER, CRF>
where
    NRF: NodeRepositoryFacadeConcept,
    ER: EdgeRepositoryConcept,
    CRF: ChannelRepositoryFacadeConcept,
{
    pub fn export(&mut self) -> Result<TreeSnapshot> {
        let root = self.export_root()?;
        Ok(TreeSnapshot::builder()
            .root(root)
            .channels(self.export_channels()?)
            .build()?)
    }

    fn export_channels(&mut self) -> Result<ChannelSnapshotMap> {
        let ids: Vec<_> = self.channel_api.channels().copied().collect();
        let spec = ids
            .iter()
            .map(|id| self.channel_api.spec(*id))
            .collect::<Result<Vec<_>>>()?;
        let config = ids
            .iter()
            .map(|id| self.channel_api.config(*id))
            .collect::<Result<Vec<_>>>()?;

        let map = izip!(ids, spec, config)
            .map(|(id, spec, meta)| (id, ChannelSnapshot2::new(spec.clone(), meta.clone())))
            .collect::<HashMap<_, _>>();

        todo!()
    }

    fn export_root(&mut self) -> Result<RootSnapshot> {
        let root_id = self
            .node_api
            .tracker()
            .nodes_by_kind(NodeKind::Root)
            .next()
            .copied()
            .ok_or_else(|| anyhow!("no root found in the tree"))?;

        let child_id = self
            .edge_api
            .children_of(root_id)
            .next()
            .copied()
            .ok_or_else(|| anyhow!("root has no child"))?;
        let child = self.export_node(child_id)?;
        Ok(RootSnapshot::new(child))
    }

    fn export_node(&mut self, id: NodeId) -> Result<NodeSnapshot> {
        let kind: NodeKind = self.node_api.spec().kind(id)?;
        match kind {
            NodeKind::Root => unreachable!(),
            NodeKind::Control => {
                let child_ids: Vec<_> = self.edge_api.children_of(id).copied().collect();
                //@todo: child_ids have to be sorted based on the x coordinate (increasing) to determine the proper children order
                let mut children = Vec::with_capacity(child_ids.len());
                for child_id in child_ids {
                    children.push(self.export_node(child_id)?);
                }

                Ok(NodeSnapshot::builder()
                    .name(self.node_api.spec().name(id)?.clone())
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
        let mut senders = vec![];
        let mut receivers = vec![];

        let ports_spec = {
            let spec_api = self.node_api.spec();
            spec_api.ports(id)?.clone()
        };
        let port_state_api = self.node_api.port_state();

        for port_id in ports_spec.ids() {
            let spec = ports_spec.spec(*port_id)?;
            let conn = port_state_api.state(id, *port_id)?;
            if let NodePortConnection::Internal(connected) = conn {
                if connected.is_empty() {
                    bail!("unconnected internal port ({port_id}) of node {id}");
                }
                match spec.kind {
                    NodePortKind::Sender => {
                        senders.extend(connected);
                    }
                    NodePortKind::Receiver => {
                        receivers.extend(connected);
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

        let name = self.node_api.spec().name(id)?.clone();
        let params = self.node_api.parameters().parameters(id)?.clone();
        Ok(NodeSnapshot::builder()
            .name(name)
            .data(leaf_snapshot)
            .parameters(params)
            .build())
    }
}
