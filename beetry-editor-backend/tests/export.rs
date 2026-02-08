mod common;

use anyhow::Result;
use beetry_editor_backend::api;
use beetry_editor_backend::edge::EdgeQueryApi;
use beetry_editor_backend::node::NodeTrackerQueryApi;
use beetry_editor_types::id::NodePortId;
use beetry_editor_types::output::edge::NodeEdge;
use beetry_editor_types::output::ui::{ChannelUiData, NodeUiData, Point};
use common::{
    ChannelSpecCase, NodeSpecCase, TestEditorService, TestSpecs, channel_position, node_position,
    service, specs,
};
use rstest::rstest;

#[rstest]
fn project_export_import(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let root = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Root)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let child = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: child,
        },
    )?;

    let store = api::project::export(&service)?;

    let mut target = TestEditorService::new(specs.node_spec_map());
    api::project::import(&mut target, store)?;

    assert_eq!(api::node::tracker::borrow(&target).nodes().count(), 2);
    assert_eq!(api::edge::borrow(&target).edges().count(), 1);
    Ok(())
}

#[rstest]
fn valid_tree_export_rejects_unconnected_node(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let root = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Root)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let connected = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::ActionStub)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let _unconnected = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderB)?,
        NodeUiData {
            position: node_position,
        },
    )?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: connected,
        },
    )?;

    let result = api::project::export_valid_tree(&service);
    assert!(result.is_err());
    Ok(())
}

#[rstest]
fn valid_tree_export_rejects_unconnected_port(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let root = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Root)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let leaf = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: node_position,
        },
    )?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: leaf,
        },
    )?;

    let result = api::project::export_valid_tree(&service);
    assert!(result.is_err());
    Ok(())
}

#[rstest]
fn valid_tree_export_succeeds_when_fully_connected(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
    channel_position: Point,
) -> Result<()> {
    let root = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Root)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let sequence_like = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Control)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let sender = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let receiver = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::ReceiverA)?,
        NodeUiData {
            position: node_position,
        },
    )?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: sequence_like,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: sequence_like,
            to: sender,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: sequence_like,
            to: receiver,
        },
    )?;

    let channel = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: channel_position,
        },
    )?;
    api::node::ports::connect(&mut service, sender, NodePortId::new(0), channel)?;
    api::node::ports::connect(&mut service, receiver, NodePortId::new(0), channel)?;

    let valid = api::project::export_valid_tree(&service);
    assert!(valid.is_ok());

    Ok(())
}

#[rstest]
fn valid_tree_export_rejects_when_connected_channel_is_removed(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
    channel_position: Point,
) -> Result<()> {
    let root = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Root)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let sequence_like = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Control)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let sender = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let receiver = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::ReceiverA)?,
        NodeUiData {
            position: node_position,
        },
    )?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: sequence_like,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: sequence_like,
            to: sender,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: sequence_like,
            to: receiver,
        },
    )?;

    let channel = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: channel_position,
        },
    )?;
    api::node::ports::connect(&mut service, sender, NodePortId::new(0), channel)?;
    api::node::ports::connect(&mut service, receiver, NodePortId::new(0), channel)?;

    api::channel::remove(&mut service, channel)?;
    let invalid_after_channel_remove = api::project::export_valid_tree(&service);
    assert!(invalid_after_channel_remove.is_err());

    Ok(())
}

#[rstest]
#[case(NodeSpecCase::SenderA)]
#[case(NodeSpecCase::ReceiverA)]
fn valid_tree_unconnected_channel(
    mut service: TestEditorService,
    specs: TestSpecs,
    #[case] leaf_case: NodeSpecCase,
    node_position: Point,
    channel_position: Point,
) -> Result<()> {
    let root = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Root)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let leaf = api::node::create(
        &mut service,
        specs.node_spec(leaf_case)?,
        NodeUiData {
            position: node_position,
        },
    )?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: leaf,
        },
    )?;

    let channel = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: channel_position,
        },
    )?;
    api::node::ports::connect(&mut service, leaf, NodePortId::new(0), channel)?;

    let result = api::project::export_valid_tree(&service);
    assert!(result.is_err());

    Ok(())
}
