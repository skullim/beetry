use std::{collections::HashSet, fmt};

use mitsein::vec1::{Vec1, vec1};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    id::{ChannelId, NodeId, NodePortId, NodeSpecId, PortConnectionId},
    persistence::{channel, node, parameter, port},
    spec::node::NodeKind,
};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MaybeValid(pub Store);

impl From<ValidTreeStore> for MaybeValid {
    fn from(value: ValidTreeStore) -> Self {
        Self(value.into_inner())
    }
}

// Proxy object to store valid tree.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(try_from = "Store")]
pub struct ValidTreeStore(#[schemars(with = "Store")] Store);

impl ValidTreeStore {
    pub fn into_inner(self) -> Store {
        self.0
    }
}

impl TryFrom<Store> for ValidTreeStore {
    type Error = ValidationErrors;

    fn try_from(tree: Store) -> Result<Self, Self::Error> {
        validate_tree(&tree)?;
        Ok(Self(tree))
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Store {
    pub node: node::Store,
    pub port: port::Store,
    pub parameter: parameter::Store,
    pub channel: channel::Store,
}

impl Store {
    pub fn new(
        node: node::Store,
        port: port::Store,
        parameter: parameter::Store,
        channel: channel::Store,
    ) -> Self {
        Self {
            node,
            port,
            parameter,
            channel,
        }
    }
}

#[derive(Debug)]
pub struct ValidationErrors(Vec1<ValidationError>);

impl ValidationErrors {
    pub fn new(errors: Vec1<ValidationError>) -> Self {
        Self(errors)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValidationError> {
        self.0.iter()
    }

    pub fn into_inner(self) -> Vec1<ValidationError> {
        self.0
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tree validation failed with {} error(s):", self.0.len())?;
        for error in &self.0 {
            writeln!(f, "{error}")?;
        }

        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("tree is missing a root node")]
    MissingRoot,
    #[error("tree references missing node {0}")]
    MissingNode(NodeId),
    #[error("node {node_id} references missing spec {spec_id}")]
    MissingSpec {
        node_id: NodeId,
        spec_id: NodeSpecId,
    },
    #[error("node {parent_id} references missing child node {child_id}")]
    MissingChildNode { parent_id: NodeId, child_id: NodeId },
    #[error("non-leaf node {0} has no children")]
    ChildFreeNonLeafNode(NodeId),
    #[error("port {port_id} on node {node_id} is unconnected")]
    UnconnectedPort {
        node_id: NodeId,
        port_id: NodePortId,
    },
    #[error("port connection {0:?} references unknown port")]
    UnknownPortReference(PortConnectionId),
    #[error("external port {port_id} on node {node_id} has an illegal connection")]
    IllegallyConnectedPort {
        node_id: NodeId,
        port_id: NodePortId,
    },
    #[error("port connection {0:?} references unknown channel")]
    UnknownChannelReference(PortConnectionId),
    #[error("node {0} is not connected to root")]
    UnconnectedNode(NodeId),
    #[error("channel {0} is not fully connected")]
    UnconnectedChannel(ChannelId),
}

fn validate_tree(tree: &Store) -> Result<(), ValidationErrors> {
    let root_id = match validate_root(tree.node.nodes.iter(), &tree.node.specs) {
        Ok(root_id) => root_id,
        Err(e) => {
            return Err(ValidationErrors::new(vec1![e]));
        }
    };

    let mut errors = Vec::new();
    let visited_nodes = validate_nodes(tree, root_id, &mut errors);
    validate_all_nodes_connected(tree.node.nodes.iter(), &visited_nodes, &mut errors);

    let connected_ports = validate_connections(tree, &mut errors);
    validate_port_connections(tree, &connected_ports, &mut errors);
    validate_channels(tree, &mut errors);

    let Ok(errors) = Vec1::try_from(errors) else {
        return Ok(());
    };
    Err(ValidationErrors(errors))
}

fn validate_root<'a, I>(mut nodes: I, specs: &node::SpecStore) -> Result<NodeId, ValidationError>
where
    I: Iterator<Item = node::RecordView<'a>>,
{
    nodes
        .find_map(|record| {
            specs
                .get(&record.value().spec_id())
                .filter(|spec| spec.kind() == NodeKind::Root)
                .map(|_| *record.id())
        })
        .ok_or(ValidationError::MissingRoot)
}

fn validate_nodes(
    tree: &Store,
    root_id: NodeId,
    errors: &mut Vec<ValidationError>,
) -> HashSet<NodeId> {
    let mut visited_nodes = HashSet::from([root_id]);
    let mut to_visit = vec![root_id];

    while let Some(node_id) = to_visit.pop() {
        let Some(node_record) = tree.node.nodes.get(&node_id) else {
            errors.push(ValidationError::MissingNode(node_id));
            continue;
        };

        let Some(node_spec) = tree.node.specs.get(&node_record.spec_id()) else {
            errors.push(ValidationError::MissingSpec {
                node_id,
                spec_id: node_record.spec_id(),
            });
            continue;
        };

        let mut children = node_record.children().copied().peekable();
        if node_spec.kind().leaf().is_none() && children.peek().is_none() {
            errors.push(ValidationError::ChildFreeNonLeafNode(node_id));
        }

        for child_id in children {
            if tree.node.nodes.get(&child_id).is_none() {
                errors.push(ValidationError::MissingChildNode {
                    parent_id: node_id,
                    child_id,
                });
                continue;
            }

            if visited_nodes.insert(child_id) {
                to_visit.push(child_id);
            }
        }
    }

    visited_nodes
}

fn validate_all_nodes_connected<'a, I>(
    nodes: I,
    visited_nodes: &HashSet<NodeId>,
    errors: &mut Vec<ValidationError>,
) where
    I: Iterator<Item = node::RecordView<'a>>,
{
    for node_id in nodes
        .map(|record| *record.id())
        .filter(|node_id| !visited_nodes.contains(node_id))
    {
        errors.push(ValidationError::UnconnectedNode(node_id));
    }
}

fn validate_connections(
    tree: &Store,
    errors: &mut Vec<ValidationError>,
) -> HashSet<(NodeId, NodePortId)> {
    let mut connected_ports = HashSet::new();

    for connection in tree.port.connections_iter().copied() {
        if tree
            .port
            .state(&connection.node_id, &connection.port_id)
            .is_none()
        {
            errors.push(ValidationError::UnknownPortReference(connection));
            continue;
        }

        if tree.channel.channels.get(&connection.channel_id).is_none() {
            errors.push(ValidationError::UnknownChannelReference(connection));
            continue;
        }
        connected_ports.insert((connection.node_id, connection.port_id));
    }

    connected_ports
}

fn validate_port_connections(
    tree: &Store,
    connected_ports: &HashSet<(NodeId, NodePortId)>,
    errors: &mut Vec<ValidationError>,
) {
    for (node_id, ports_state) in tree.port.states_iter() {
        for (port_id, state) in ports_state {
            let has_connection = connected_ports.contains(&(*node_id, *port_id));

            match (state.is_external(), has_connection) {
                (true, true) => {
                    errors.push(ValidationError::IllegallyConnectedPort {
                        node_id: *node_id,
                        port_id: *port_id,
                    });
                }
                (false, false) => {
                    errors.push(ValidationError::UnconnectedPort {
                        node_id: *node_id,
                        port_id: *port_id,
                    });
                }
                _ => {}
            }
        }
    }
}

fn validate_channels(tree: &Store, errors: &mut Vec<ValidationError>) {
    for channel_id in tree.channel.channels.iter().filter_map(|(id, data)| {
        let count = data.config.count();
        ((count.sender() == 0) || (count.receiver() == 0)).then_some(*id)
    }) {
        errors.push(ValidationError::UnconnectedChannel(channel_id));
    }
}
