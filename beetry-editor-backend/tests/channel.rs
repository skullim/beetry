mod common;

use anyhow::Result;
use beetry_editor_backend::{api, api::ChannelQueryView};
use beetry_editor_types::{id::ChannelId, output::channel::ChannelConfigUpdate};
use common::{ChannelSpecCase, TestEditorService, TestSpecs, create_channel, service, specs};
use rstest::rstest;

#[rstest]
fn creating_channel_succeeds(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let channel_id = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;

    api::channel::query(&service).config(channel_id).unwrap();
    Ok(())
}

#[rstest]
fn channel_update_config_updates_capacity(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let channel_id = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;
    let new_capacity = 32_usize;

    api::channel::update_config(
        &mut service,
        channel_id,
        ChannelConfigUpdate {
            capacity: new_capacity,
        },
    )?;

    {
        let query = api::channel::query(&service);
        let cfg = query.config(channel_id)?;
        assert_eq!(cfg.capacity(), new_capacity);
        assert_eq!(cfg.count().sender(), 0);
        assert_eq!(cfg.count().receiver(), 0);
    }

    api::channel::update_config(
        &mut service,
        ChannelId::new(999),
        ChannelConfigUpdate { capacity: 1 },
    )
    .unwrap_err();

    Ok(())
}

#[rstest]
fn removing_existing_channel_succeeds(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let channel_id = create_channel(
        &mut service,
        &specs,
        ChannelSpecCase::MessageA,
        TestSpecs::default_mpsc_config(),
    )?;

    api::channel::remove(&mut service, channel_id)?;
    api::channel::query(&service)
        .config(channel_id)
        .unwrap_err();
    Ok(())
}

#[rstest]
fn removing_missing_channel_fails(mut service: TestEditorService) {
    api::channel::remove(&mut service, ChannelId::new(999)).unwrap_err();
}
