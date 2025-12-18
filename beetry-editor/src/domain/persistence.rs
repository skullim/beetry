use std::collections::HashMap;

use crate::domain::{
    models::{
        ChannelData, ChannelSpecId, ChannelUiData, NodeId, NodePortConnection, NodePortId,
        NodeSpecId, NodeSpecKey, NodeUiData,
    },
    repository::{
        NodeRepositoryConcept, NodeRepositoryFacadeConcept, NodeRepositoryFacadeView,
        ParamValueRepositoryConcept, PortStateRepositoryConcept, SpecRepositoryConcept,
    },
};
use anyhow::{Result, anyhow};
use beetry_plugin_types::channel::ChannelSpec;
use beetry_reconstruction_types::{channel::ChannelId, parameter};
use indexmap::IndexSet;

pub struct EditorStateStore {
    pub tree: MaybeValidTree,
    pub ui_elements: UiElementStore,
}

// Editor might export/import either valid or (still) invalid tree
pub struct MaybeValidTree(pub TreeStore);

// Proxy object to store valid tree
// @todo: hide constructor and let service layer validate and construct the instance
pub struct ValidTree(pub TreeStore);

impl ValidTree {
    pub fn into_inner(self) -> TreeStore {
        self.0
    }
}

pub struct TreeStore {
    pub node: NodeStore,
    pub ports: NodePortStore,
    pub parameter: ParameterValueStore,
    pub channel: ChannelStore,
}

pub struct NodeStore {
    pub specs: Vec<NodeSpecRecord>,
    pub nodes: Vec<NodeRecord>,
}

impl NodeStore {
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
                key: NodeSpecKey::new(spec.name().clone(), spec.kind()),
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
                    // parameters: parameters.value(id).cloned(),
                    // port_records: ports
                    //     .port_iter(id)
                    //     .map(|(port_id, conn)| {
                    //         Ok(NodePortRecord {
                    //             id: *port_id,
                    //             conn: conn.clone(),
                    //         })
                    //     })
                    //     .collect::<Result<Vec<NodePortRecord>>>()?,

                    //@todo this has to be obtained from edge service
                    children: <_>::default(),
                })
            })
            .collect::<Result<Vec<NodeRecord>>>()?;
        Ok(Self { specs, nodes })
    }
}

//The remaining parts of spec are to be loaded by the appropriate plugin
pub struct NodeSpecRecord {
    pub id: NodeSpecId,
    pub key: NodeSpecKey,
}

pub struct NodeRecord {
    pub id: NodeId,
    pub spec_id: NodeSpecId,
    pub children: IndexSet<NodeId>,
}

pub struct ParameterValueStore {
    parameters: HashMap<NodeId, ParameterValue>,
}

impl ParameterValueStore {
    pub fn take(&mut self, id: &NodeId) -> Option<ParameterValue> {
        self.parameters.remove(id)
    }
}

pub struct ParameterValue {
    pub params: parameter::Parameters,
}

pub struct NodePortStore {
    ports: HashMap<NodeId, NodePortState>,
}

impl NodePortStore {
    pub fn take(&mut self, id: &NodeId) -> Option<NodePortState> {
        self.ports.remove(id)
    }
}

pub struct NodePortState {
    pub conns: Vec<(NodePortId, NodePortConnection)>,
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
