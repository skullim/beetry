use anyhow::{Result, anyhow};

use beetry_plugin_types::channel::ChannelSpec;
use beetry_reconstruction_types::{channel::ChannelId, parameter};

use crate::domain::{
    models::{
        ChannelData, ChannelSpecId, EdgeId, NodeEdge, NodeId, NodePortConnection, NodePortId,
        NodePosition, NodeSpec, NodeSpecId,
    },
    repository::{
        EdgeRepositoryConcept, NodePositionRepositoryConcept, NodeRepositoryConcept,
        NodeRepositoryFacadeConcept, NodeRepositoryFacadeView, ParamValuesRepositoryConcept,
        PortStateRepositoryConcept, SpecRepositoryConcept,
    },
};

pub struct EditorStorage {
    pub tree: TreeStorage,
}

pub struct TreeStorage {
    pub node: NodeStorage,
    pub edges: EdgeStorage,
    pub channels: ChannelStorage,
}

pub struct NodeStorage {
    pub specs: Vec<NodeSpecRecord>,
    pub nodes: Vec<NodeRecord>,
}

impl NodeStorage {
    pub fn load(facade: &impl NodeRepositoryFacadeConcept) -> Result<Self> {
        let NodeRepositoryFacadeView {
            nodes,
            specs,
            parameters,
            positions,
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
                    parameters: parameters.params(id).cloned(),
                    position: *positions
                        .position(id)
                        .ok_or_else(|| anyhow!("expected set position for node {id}"))?,
                    port_records: ports
                        .port_iter(id)
                        .map(|(port_id, conn)| {
                            Ok(NodePortRecord {
                                id: *port_id,
                                conn: conn.clone(),
                            })
                        })
                        .collect::<Result<Vec<NodePortRecord>>>()?,
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

//@todo still to consider, but it might be more beneficial here to have children indexset https://docs.rs/indexmap/latest/indexmap/set/struct.IndexSet.html
// Then it would be possible to decouple the UiPosition from the node data.
// This in turn would allow to split the project into strictly tree data and UI elements data.
// Only tree data would be needed for backend to execute the tree, whereas the project would be sum of tree and UI elements data.
// This also mean that channel position data should probably be split into another repository, or merged together with node position.
pub struct NodeRecord {
    pub id: NodeId,
    pub spec_id: NodeSpecId,
    pub port_records: Vec<NodePortRecord>,
    pub parameters: Option<parameter::Parameters>,
    pub position: NodePosition,
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
    pub data: ChannelData,
}
