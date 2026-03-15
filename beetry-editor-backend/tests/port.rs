mod common;

use anyhow::Result;
use beetry_editor_backend::{api, ui::PortConnectionUiQuery};
use beetry_editor_types::{
    id::{ChannelId, NodePortId, PortConnectionId},
    output::ui::{PortConnectionUiData, VisibilityKind},
    spec::node::NodePortKind,
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
fn port_order_returns_correct_position(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let duplex = create_node(&mut service, &specs, NodeSpecCase::DuplexA)?;

    let sender_row =
        api::node::ports::order(&service, NodePortKind::Sender, duplex, NodePortId::new(0))?;
    let receiver_row =
        api::node::ports::order(&service, NodePortKind::Receiver, duplex, NodePortId::new(1))?;
    assert_eq!(sender_row, 0);
    assert_eq!(receiver_row, 0);

    api::node::ports::order(&service, NodePortKind::Sender, duplex, NodePortId::new(1))
        .unwrap_err();

    Ok(())
}

#[rstest]
fn port_ui_query_and_update(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let sender = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let channel = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;
    let conn_id = PortConnectionId::new(sender, NodePortId::new(0), channel);
    api::node::ports::connect(&mut service, conn_id, visible_conn_ui())?;

    {
        let ui_query = api::ui::port::query(&service);
        assert!(matches!(
            ui_query.data(conn_id)?.visibility(),
            VisibilityKind::Visible
        ));
    }

    api::ui::port::update_data(
        &mut service,
        conn_id,
        PortConnectionUiData::new(VisibilityKind::Hidden),
    )?;

    {
        let updated = api::ui::port::query(&service);
        assert!(matches!(
            updated.data(conn_id)?.visibility(),
            VisibilityKind::Hidden
        ));
    }

    {
        let query = api::ui::port::query(&service);
        query
            .data(PortConnectionId::new(
                sender,
                NodePortId::new(0),
                ChannelId::new(999),
            ))
            .unwrap_err();
    }

    api::ui::port::update_data(
        &mut service,
        PortConnectionId::new(sender, NodePortId::new(0), ChannelId::new(999)),
        PortConnectionUiData::new(VisibilityKind::Visible),
    )
    .unwrap_err();

    Ok(())
}
