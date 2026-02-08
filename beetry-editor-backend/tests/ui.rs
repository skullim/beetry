mod common;

use anyhow::Result;
use beetry_editor_backend::api;
use beetry_editor_backend::api::{
    ChannelQueryView, ChannelUiQueryApi, NodeTrackerQueryView, NodeUiQueryApi,
};
use beetry_editor_types::output::ui::{ChannelUiData, NodeUiData, Point};
use common::{ChannelSpecCase, NodeSpecCase, TestEditorService, TestSpecs, service, specs};
use rstest::rstest;

#[rstest]
fn ui_update_pos(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let node_id = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: Point { x: 1.0, y: 2.0 },
        },
    )?;
    let channel_id = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: Point { x: 3.0, y: 4.0 },
        },
    )?;

    api::ui::node::update_position(&mut service, node_id, Point { x: 7.0, y: 8.0 })?;
    api::ui::channel::update_position(&mut service, channel_id, Point { x: 9.0, y: 10.0 })?;

    {
        let q = api::ui::node::borrow(&service);
        assert_eq!(*q.position(node_id)?, Point { x: 7.0, y: 8.0 });
    }
    {
        let q = api::ui::channel::borrow(&service);
        assert_eq!(*q.position(channel_id)?, Point { x: 9.0, y: 10.0 });
    }
    Ok(())
}

#[rstest]
fn node_and_channel_in_sync(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let node_id = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: Point { x: 10.0, y: 20.0 },
        },
    )?;
    let channel_id = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        specs.default_mpsc_config(),
        ChannelUiData {
            position: Point { x: 30.0, y: 40.0 },
        },
    )?;

    {
        let tracker = api::node::tracker::query_view(&service);
        assert_eq!(tracker.nodes().count(), 1);
        assert!(tracker.nodes().any(|id| *id == node_id));
    }

    let node_pos = {
        let query = api::ui::node::borrow(&service);
        *query.position(node_id)?
    };
    assert_eq!(node_pos, Point { x: 10.0, y: 20.0 });

    let channel_pos = {
        let query = api::ui::channel::borrow(&service);
        *query.position(channel_id)?
    };
    assert_eq!(channel_pos, Point { x: 30.0, y: 40.0 });

    api::node::remove(&mut service, node_id)?;
    api::channel::remove(&mut service, channel_id)?;

    assert!(
        !api::node::tracker::query_view(&service)
            .nodes()
            .any(|id| *id == node_id)
    );
    assert!(api::ui::node::borrow(&service).position(node_id).is_err());

    assert!(api::channel::borrow(&service).config(channel_id).is_err());
    assert!(
        api::ui::channel::borrow(&service)
            .position(channel_id)
            .is_err()
    );

    Ok(())
}
