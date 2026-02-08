mod common;

use anyhow::Result;
use beetry_editor_backend::api;
use beetry_editor_backend::api::ChannelQueryView;
use beetry_editor_types::id::{ChannelId, NodePortId};
use beetry_editor_types::output::ui::{ChannelUiData, NodeUiData, Point};
use common::{
    ChannelSpecCase, NodeSpecCase, TestEditorService, TestSpecs, channel_position, node_position,
    service, specs,
};
use rstest::rstest;

#[rstest]
fn create_channel(
    mut service: TestEditorService,
    specs: TestSpecs,
    channel_position: Point,
) -> Result<()> {
    let channel_id = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: channel_position,
        },
    )?;

    assert!(api::channel::borrow(&service).config(channel_id).is_ok());
    Ok(())
}

#[rstest]
fn removing_existing_channel_succeeds(
    mut service: TestEditorService,
    specs: TestSpecs,
    channel_position: Point,
) -> Result<()> {
    let channel_id = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: channel_position,
        },
    )?;

    api::channel::remove(&mut service, channel_id)?;
    assert!(api::channel::borrow(&service).config(channel_id).is_err());
    Ok(())
}

#[rstest]
fn removing_missing_channel_fails(mut service: TestEditorService) {
    let result = api::channel::remove(&mut service, ChannelId::new(999));
    assert!(result.is_err());
}

#[rstest]
fn sender_port_connect_sets_internal_state_and_sender_count(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
    channel_position: Point,
) -> Result<()> {
    let node_id = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let channel_id = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: channel_position,
        },
    )?;
    let port_id = NodePortId::new(0);

    api::node::ports::connect(&mut service, node_id, port_id, channel_id)?;
    let internal_connections: Vec<_> = api::node::ports::internal_connections(&service)
        .filter(|(n, p, _)| **n == node_id && **p == port_id)
        .map(|(_, _, c)| *c)
        .collect();
    assert_eq!(internal_connections, vec![channel_id]);

    let views = api::node::ports::connection_views(&service).collect::<Result<Vec<_>>>()?;
    assert_eq!(views.len(), 1);
    let conn = views[0];
    assert_eq!(conn.node_id, node_id);
    assert_eq!(conn.port_id, port_id);
    assert_eq!(conn.channel_id, channel_id);

    assert_eq!(
        api::channel::borrow(&service)
            .config(channel_id)?
            .count()
            .sender(),
        1
    );

    Ok(())
}

#[rstest]
fn connecting_second_receiver_to_mpsc_channel_fails(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
    channel_position: Point,
) -> Result<()> {
    let receiver_a = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::ReceiverA)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let receiver_b = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::ReceiverA)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let channel_id = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: channel_position,
        },
    )?;
    let port_id = NodePortId::new(0);

    api::node::ports::connect(&mut service, receiver_a, port_id, channel_id)?;
    let second_connect = api::node::ports::connect(&mut service, receiver_b, port_id, channel_id);
    assert!(second_connect.is_err());

    Ok(())
}

#[rstest]
fn sender_port_disconnect_clears_internal_state_and_sender_count(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
    channel_position: Point,
) -> Result<()> {
    let node_id = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let channel_id = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: channel_position,
        },
    )?;
    let port_id = NodePortId::new(0);

    api::node::ports::connect(&mut service, node_id, port_id, channel_id)?;
    api::node::ports::disconnect(&mut service, node_id, port_id, channel_id)?;
    assert!(
        api::node::ports::internal_connections(&service)
            .all(|(n, p, c)| !(*n == node_id && *p == port_id && *c == channel_id))
    );
    assert_eq!(
        api::channel::borrow(&service)
            .config(channel_id)?
            .count()
            .sender(),
        0
    );
    assert!(
        api::node::ports::connection_views(&service)
            .collect::<Result<Vec<_>>>()?
            .is_empty()
    );
    Ok(())
}

#[rstest]
fn connecting_external_port_to_channel_fails(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
    channel_position: Point,
) -> Result<()> {
    let node_id = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let channel_id = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: channel_position,
        },
    )?;
    let port_id = NodePortId::new(0);

    api::node::ports::set_external(&mut service, node_id, port_id)?;
    let result = api::node::ports::connect(&mut service, node_id, port_id, channel_id);
    assert!(result.is_err());
    Ok(())
}
