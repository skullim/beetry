use anyhow::Result;
use std::collections::HashMap;

use beetry_plugin_types::channel::ChannelSpec;
use beetry_reconstruction_types::{
    channel::{ChannelConfig, ChannelId},
    parameter,
};

use crate::domain::{
    models::{
        ChannelPosition, ChannelSpecId, EdgeId, NodeEdge, NodeId, NodePortConnection, NodePortId,
        NodePosition, NodeSpec, NodeSpecId,
    },
    repository::{
        EdgeRepositoryConcept, NodeRepositoryConcept, NodeRepositoryFacadeConcept,
        NodeRepositoryFacadeView, ParamValuesRepositoryConcept, PortStateRepositoryConcept,
        SpecRepositoryConcept,
    },
};

pub struct EditorStorage {
    pub tree: TreeStorage,
    pub positions: UiElementPositions,
}

pub struct TreeStorage {
    pub node: NodeStorage,
    pub edges: EdgeStorage,
    pub channels: ChannelStorage,
}

pub struct NodeStorage {
    specs: Vec<NodeSpecRecord>,
    nodes: Vec<NodeRecord>,
}

impl NodeStorage {
    pub fn load(facade: &impl NodeRepositoryFacadeConcept) -> Result<Self> {
        let NodeRepositoryFacadeView {
            nodes,
            specs,
            parameters,
            ports,
            ..
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
            .map(|id| NodeRecord {
                id,
                spec_id: *nodes
                    .spec_id(&id)
                    .ok_or_else(|| format!("expected spec id for node {id}"))
                    .unwrap(),
                parameters: parameters.params(id).cloned(),
                port_records: ports
                    .port_iter(id)
                    .map(|(port_id, conn)| NodePortRecord {
                        id: *port_id,
                        conn: conn.clone(),
                    })
                    .collect(),
            })
            .collect();
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
}

pub struct NodePortRecord {
    pub id: NodePortId,
    pub conn: NodePortConnection,
}

pub struct EdgeStorage {
    edges: Vec<EdgeRecord>,
}

impl EdgeStorage {
    pub fn load(repo: &impl EdgeRepositoryConcept) -> Result<Self> {
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

pub struct ChannelStorage {
    specs: Vec<ChannelSpecRecord>,
    channels: Vec<ChannelRecord>,
}

pub struct ChannelSpecRecord {
    pub id: ChannelSpecId,
    pub spec: ChannelSpec,
}

pub struct ChannelRecord {
    pub id: ChannelId,
    pub spec_id: ChannelSpecId,
    pub config: ChannelConfig,
}

pub struct UiElementPositions {
    pub nodes: HashMap<NodeId, NodePosition>,
    pub channels: HashMap<ChannelId, ChannelPosition>,
}
