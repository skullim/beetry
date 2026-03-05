mod common;

use anyhow::Result;
use beetry_editor_backend::api;
use beetry_editor_backend::api::{EdgeQueryView, NodeTrackerQuery};
use beetry_editor_types::id::{NodePortId, PortConnectionId};
use beetry_editor_types::output::edge::NodeEdge;
use beetry_editor_types::output::ui::{PortConnectionUiData, VisibilityKind};
use common::{
    ChannelSpecCase, NodeSpecCase, TestEditorService, TestSpecs, create_channel, create_node,
    service, specs,
};
use rstest::rstest;

fn visible_conn_ui() -> PortConnectionUiData {
    PortConnectionUiData::new(VisibilityKind::Visible)
}

#[rstest]
fn two_nodes_project_export_import(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let child = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: child,
        },
    )?;

    let store = api::project::export(&service)?;

    api::project::import(&mut service, store)?;
    assert_eq!(api::node::tracker::query(&service).nodes().count(), 2);
    assert_eq!(api::edge::query(&service).edges().count(), 1);
    Ok(())
}

#[rstest]
fn simple_valid_tree_export_import(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let control = create_node(&mut service, &specs, NodeSpecCase::Control)?;
    let action = create_node(&mut service, &specs, NodeSpecCase::ActionStub)?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: control,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: control,
            to: action,
        },
    )?;

    let store = api::project::export(&service)?;
    api::project::import(&mut service, store)?;

    assert_eq!(api::node::tracker::query(&service).nodes().count(), 3);
    assert_eq!(api::edge::query(&service).edges().count(), 2);
    assert!(api::project::export_valid_tree(&service).is_ok());

    Ok(())
}

#[rstest]
fn valid_tree_export_rejects_unconnected_node(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let connected = create_node(&mut service, &specs, NodeSpecCase::ActionStub)?;
    let _unconnected = create_node(&mut service, &specs, NodeSpecCase::SenderB)?;

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
) -> Result<()> {
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let leaf = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;

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
) -> Result<()> {
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let sequence_like = create_node(&mut service, &specs, NodeSpecCase::Control)?;
    let sender = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let receiver = create_node(&mut service, &specs, NodeSpecCase::ReceiverA)?;

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

    let channel = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        specs.default_mpsc_config(),
    )?;
    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(sender, NodePortId::new(0), channel),
        visible_conn_ui(),
    )?;
    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(receiver, NodePortId::new(0), channel),
        visible_conn_ui(),
    )?;

    let valid = api::project::export_valid_tree(&service);
    assert!(valid.is_ok());

    Ok(())
}

#[rstest]
fn valid_tree_export_fails_when_connected_channel_is_removed(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let control = create_node(&mut service, &specs, NodeSpecCase::Control)?;
    let sender = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let receiver = create_node(&mut service, &specs, NodeSpecCase::ReceiverA)?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: control,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: control,
            to: sender,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: control,
            to: receiver,
        },
    )?;

    let channel = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        specs.default_mpsc_config(),
    )?;
    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(sender, NodePortId::new(0), channel),
        visible_conn_ui(),
    )?;
    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(receiver, NodePortId::new(0), channel),
        visible_conn_ui(),
    )?;

    api::channel::remove(&mut service, channel)?;
    let valid_after_channel_remove = api::project::export_valid_tree(&service);
    assert!(valid_after_channel_remove.is_err());

    Ok(())
}

#[rstest]
#[case(NodeSpecCase::SenderA)]
#[case(NodeSpecCase::ReceiverA)]
fn valid_tree_export_fails_on_unconnected_channel(
    mut service: TestEditorService,
    specs: TestSpecs,
    #[case] leaf_case: NodeSpecCase,
) -> Result<()> {
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let leaf = create_node(&mut service, &specs, leaf_case)?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: leaf,
        },
    )?;

    let channel = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        specs.default_mpsc_config(),
    )?;
    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(leaf, NodePortId::new(0), channel),
        visible_conn_ui(),
    )?;

    let result = api::project::export_valid_tree(&service);
    assert!(result.is_err());

    Ok(())
}
