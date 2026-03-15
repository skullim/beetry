mod common;

use anyhow::Result;
use beetry_editor_backend::{
    api,
    api::{ChannelUiQuery, NodeUiQuery},
};
use beetry_editor_types::output::ui::{ChannelUiData, NodeUiData, Point};
use common::{ChannelSpecCase, NodeSpecCase, TestEditorService, TestSpecs, service, specs};
use rstest::rstest;

#[rstest]
fn update_node_position(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let start_pose = Point { x: 1.0, y: 2.0 };
    let final_pose = Point { x: 7.0, y: 8.0 };

    let node_id = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: start_pose,
        },
    )?;

    {
        let q = api::ui::node::query(&service);
        assert_eq!(*q.position(node_id)?, start_pose);
    }

    api::ui::node::update_position(&mut service, node_id, final_pose)?;

    let q = api::ui::node::query(&service);
    assert_eq!(*q.position(node_id)?, final_pose);
    Ok(())
}

#[rstest]
fn update_channel_position(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let start_pose = Point { x: 3.0, y: 4.0 };
    let final_pose = Point { x: 9.0, y: 10.0 };

    let channel_id = api::channel::create(
        &mut service,
        specs.channel_spec(ChannelSpecCase::MessageA)?,
        TestSpecs::default_mpsc_config(),
        ChannelUiData {
            position: start_pose,
        },
    )?;

    {
        let q = api::ui::channel::query(&service);
        assert_eq!(*q.position(channel_id)?, start_pose);
    }

    api::ui::channel::update_position(&mut service, channel_id, final_pose)?;

    let q = api::ui::channel::query(&service);
    assert_eq!(*q.position(channel_id)?, final_pose);
    Ok(())
}
