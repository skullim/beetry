use std::default;

use anyhow::{Result, anyhow};

use beetry_plugin_types::channel::ChannelSpec;
use beetry_reconstruction_types::{channel::ChannelId, parameter};
use indexmap::IndexSet;

use crate::domain::{
    models::{
        ChannelData, ChannelSpecId, ChannelUiData, EdgeId, NodeEdge, NodeId, NodePortConnection,
        NodePortId, NodeSpec, NodeSpecId, NodeUiData,
    },
    repository::{
        EdgeRepositoryConcept, NodeRepositoryConcept, NodeRepositoryFacadeConcept,
        NodeRepositoryFacadeView, ParamValueRepositoryConcept, PortStateRepositoryConcept,
        SpecRepositoryConcept,
    },
};

pub struct EditorStore {
    pub tree: TreeStore,
    pub ui_elements: UiElementStore,
}

pub struct TreeStore {
    pub graph: GraphStore,
    pub channels: ChannelStore,
}

pub struct GraphStore {
    pub specs: Vec<NodeSpecRecord>,
    pub nodes: Vec<NodeRecord>,
}

impl GraphStore {
    pub fn export(facade: &impl NodeRepositoryFacadeConcept) -> Result<Self> {
        let NodeRepositoryFacadeView {
            nodes,
            specs,
            parameters,
            ports,
        } = facade.view();
        let specs: Vec<_> = specs
            .iter()
            .map(|(id, spec)| NodeSpecRecord {
                id: *id,
                spec: spec.clone(),
            })
            .collect();

        let node_ids = nodes.ids().copied();
        let nodes = node_ids
            .map(|id| {
                Ok(NodeRecord {
                    id,
                    spec_id: *nodes
                        .spec_id(&id)
                        .ok_or_else(|| anyhow!("expected spec id for node {id}"))?,
                    parameters: parameters.value(id).cloned(),
                    port_records: ports
                        .port_iter(id)
                        .map(|(port_id, conn)| {
                            Ok(NodePortRecord {
                                id: *port_id,
                                conn: conn.clone(),
                            })
                        })
                        .collect::<Result<Vec<NodePortRecord>>>()?,
                    //@todo this has to be obtained from edge service
                    children: <_>::default(),
                })
            })
            .collect::<Result<Vec<NodeRecord>>>()?;
        Ok(Self { specs, nodes })
    }
}

pub struct NodeSpecRecord {
    pub id: NodeSpecId,
    pub spec: NodeSpec,
}

pub struct NodeRecord {
    pub id: NodeId,
    pub spec_id: NodeSpecId,
    pub port_records: Vec<NodePortRecord>,
    pub parameters: Option<parameter::Parameters>,
    pub children: IndexSet<NodeId>,
}

pub struct NodePortRecord {
    pub id: NodePortId,
    pub conn: NodePortConnection,
}

//@todo should no longer be needed
pub struct EdgeStore {
    edges: Vec<EdgeRecord>,
}

impl EdgeStore {
    pub fn export(repo: &impl EdgeRepositoryConcept) -> Result<Self> {
        let edges = repo
            .iter()
            .map(|(id, edge)| EdgeRecord {
                id: *id,
                node_edge: edge.clone(),
            })
            .collect();
        Ok(Self { edges })
    }
}

pub struct EdgeRecord {
    pub id: EdgeId,
    pub node_edge: NodeEdge,
}

pub struct ChannelStore {
    specs: Vec<ChannelSpecRecord>,
    channels: Vec<ChannelRecord>,
}

pub struct ChannelSpecRecord {
    pub id: ChannelSpecId,
    pub spec: ChannelSpec,
}

pub struct ChannelRecord {
    pub id: ChannelId,
    pub data: ChannelData,
}

pub struct UiElementStore {
    pub nodes: Vec<NodeUiRecord>,
    pub channels: Vec<ChannelUiRecord>,
}

pub struct NodeUiRecord {
    pub id: NodeId,
    pub data: NodeUiData,
}

pub struct ChannelUiRecord {
    pub id: ChannelId,
    pub data: ChannelUiData,
}
