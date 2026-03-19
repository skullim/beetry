mod common;

use anyhow::Result;
use beetry_editor_backend::{
    api,
    api::ChannelQueryView,
    node::{PortConnectionQuery, PortStateQuery},
};
use beetry_editor_types::{
    id::{NodePortId, PortConnectionId},
    output::{
        node::{PortSource, PortState},
        ui::{PortConnectionUiData, VisibilityKind},
    },
};
use common::{
    ChannelSpecCase, NodeSpecCase, TestEditorService, TestSpecs, create_channel, create_node,
    service, specs,
};
use rstest::rstest;

fn visible_conn_ui() -> PortConnectionUiData {
    PortConnectionUiData::new(VisibilityKind::Visible)
}

#[rstest]
fn valid_connection_mutates_channel_state(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let node_id = create_node(&mut service, &specs, NodeSpecCase::DuplexA)?;
    let channel_id = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;
    let sender_port_id = NodePortId::new(0);
    let sender_connection_id = PortConnectionId::new(node_id, sender_port_id, channel_id);
    api::node::ports::connect(&mut service, sender_connection_id, visible_conn_ui())?;

    {
        let query = api::node::ports::connections_query(&service);
        assert_eq!(query.all_connections().count(), 1);
        let mut iter = query.all_connections();
        assert_eq!(iter.next(), Some(sender_connection_id));
    }

    assert_eq!(
        api::channel::query(&service)
            .config(channel_id)?
            .count()
            .sender(),
        1
    );

    let receiver_port_id = NodePortId::new(1);
    let receiver_connection_id = PortConnectionId::new(node_id, receiver_port_id, channel_id);
    api::node::ports::connect(&mut service, receiver_connection_id, visible_conn_ui())?;

    {
        let query = api::node::ports::connections_query(&service);
        assert_eq!(query.all_connections().count(), 2);
        let mut conns: Vec<_> = query.all_connections().collect();
        conns.sort_by_key(|c| c.port_id);
        assert_eq!(conns, vec![sender_connection_id, receiver_connection_id]);
    }

    assert_eq!(
        api::channel::query(&service)
            .config(channel_id)?
            .count()
            .sender(),
        1
    );

    assert_eq!(
        api::channel::query(&service)
            .config(channel_id)?
            .count()
            .receiver(),
        1
    );

    Ok(())
}

#[rstest]
fn connecting_second_receiver_to_mpsc_channel_fails(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let receiver_a = create_node(&mut service, &specs, NodeSpecCase::ReceiverA)?;
    let receiver_b = create_node(&mut service, &specs, NodeSpecCase::ReceiverA)?;
    let channel_id = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;
    let port_id = NodePortId::new(0);

    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(receiver_a, port_id, channel_id),
        visible_conn_ui(),
    )?;
    let second_connect = api::node::ports::connect(
        &mut service,
        PortConnectionId::new(receiver_b, port_id, channel_id),
        visible_conn_ui(),
    );
    assert!(second_connect.is_err());

    Ok(())
}

#[rstest]
fn connecting_second_channel_to_same_port_fails(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let sender = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let first_channel = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;
    let second_channel = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;
    let port_id = NodePortId::new(0);

    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(sender, port_id, first_channel),
        visible_conn_ui(),
    )?;

    let second_connect = api::node::ports::connect(
        &mut service,
        PortConnectionId::new(sender, port_id, second_channel),
        visible_conn_ui(),
    );
    assert!(second_connect.is_err());

    Ok(())
}

#[rstest]
fn connecting_mismatched_channel_type_fails(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let sender = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let channel_b = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageB,
        TestSpecs::default_mpsc_config(),
    )?;

    let port_id = NodePortId::new(0);
    let conn = PortConnectionId::new(sender, port_id, channel_b);
    let result = api::node::ports::connect(&mut service, conn, visible_conn_ui());
    assert!(result.is_err());

    let query = api::node::ports::connections_query(&service);
    assert!(!query.is_port_connected(sender, port_id));
    assert!(!query.all_connections().any(|id| id == conn));
    assert_eq!(
        api::channel::query(&service)
            .config(channel_b)?
            .count()
            .sender(),
        0
    );
    assert_eq!(
        api::channel::query(&service)
            .config(channel_b)?
            .count()
            .receiver(),
        0
    );

    Ok(())
}

#[rstest]
fn connecting_external_port_to_channel_fails(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let node_id = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let channel_id = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;
    let port_id = NodePortId::new(0);

    api::node::ports::set_state(
        &mut service,
        node_id,
        port_id,
        PortState::new(PortSource::External),
    )?;
    let result = api::node::ports::connect(
        &mut service,
        PortConnectionId::new(node_id, port_id, channel_id),
        visible_conn_ui(),
    );
    assert!(result.is_err());
    Ok(())
}

#[rstest]
fn disconnecting_port_triggers_cleanup(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let node_id = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let channel_id = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;
    let port_id = NodePortId::new(0);

    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(node_id, port_id, channel_id),
        visible_conn_ui(),
    )?;

    {
        let query = api::node::ports::connections_query(&service);
        assert_eq!(query.all_connections().count(), 1);
    }

    api::node::ports::disconnect(
        &mut service,
        PortConnectionId::new(node_id, port_id, channel_id),
    )?;
    {
        let query = api::node::ports::connections_query(&service);
        assert_eq!(query.all_connections().count(), 0);
    }
    assert_eq!(
        api::channel::query(&service)
            .config(channel_id)?
            .count()
            .sender(),
        0
    );
    assert_eq!(
        api::node::ports::connections_query(&service)
            .all_connections()
            .count(),
        0
    );
    Ok(())
}

#[rstest]
fn setting_port_to_external_removes_connection(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let node_id = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let channel_id = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;
    let port_id = NodePortId::new(0);
    let connection_id = PortConnectionId::new(node_id, port_id, channel_id);

    api::node::ports::connect(&mut service, connection_id, visible_conn_ui())?;
    assert!(api::node::ports::connections_query(&service).is_port_connected(node_id, port_id));

    api::node::ports::set_state(
        &mut service,
        node_id,
        port_id,
        PortState::new(PortSource::External),
    )?;

    let state_query = api::node::ports::state_query(&service);
    let state = state_query.state(node_id, port_id)?;
    assert!(state.is_external());
    assert!(!api::node::ports::connections_query(&service).is_port_connected(node_id, port_id));

    Ok(())
}

#[rstest]
fn removing_node_disconnects_connections(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let node_id = create_node(&mut service, &specs, NodeSpecCase::DuplexA)?;
    let channel_id = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;

    let receiver_port_id = NodePortId::new(0);
    let sender_port_id = NodePortId::new(1);
    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(node_id, receiver_port_id, channel_id),
        visible_conn_ui(),
    )?;
    api::node::ports::connect(
        &mut service,
        PortConnectionId::new(node_id, sender_port_id, channel_id),
        visible_conn_ui(),
    )?;

    let conn_count = api::node::ports::connections_query(&service)
        .all_connections()
        .count();
    assert_eq!(conn_count, 2);

    api::node::remove(&mut service, node_id)?;

    assert!(
        api::node::ports::connections_query(&service)
            .all_connections()
            .count()
            == 0
    );

    assert_eq!(
        api::channel::query(&service)
            .config(channel_id)?
            .count()
            .sender(),
        0
    );

    assert_eq!(
        api::channel::query(&service)
            .config(channel_id)?
            .count()
            .receiver(),
        0
    );

    Ok(())
}
