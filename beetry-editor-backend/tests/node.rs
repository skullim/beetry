mod common;

use anyhow::Result;
use beetry_editor_backend::api;
use beetry_editor_backend::api::{NodeTrackerQuery, SpecByNodeIdQuery, SpecBySpecIdQuery};
use beetry_editor_backend::ui::NodeUiQuery;
use beetry_editor_types::id::{NodeId, NodeSpecId};
use common::{NodeSpecCase, TestEditorService, TestSpecs, create_node, service, specs};
use rstest::rstest;

#[rstest]
fn creating_nodes_succeeds(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let cases = [
        NodeSpecCase::Root,
        NodeSpecCase::Control,
        NodeSpecCase::Decorator,
        NodeSpecCase::SenderA,
        NodeSpecCase::ReceiverA,
        NodeSpecCase::DuplexA,
        NodeSpecCase::SenderB,
    ];

    let mut node_ids = vec![];
    for case in cases {
        let id = create_node(&mut service, &specs, case)?;
        node_ids.push(id);
    }

    let tracker = api::node::tracker::query(&service);
    assert_eq!(tracker.nodes().count(), cases.len());

    let ui_query = api::ui::node::query(&service);
    for id in node_ids {
        ui_query.data(id)?;
    }

    Ok(())
}

#[rstest]
fn creating_second_root_fails(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let _root = create_node(&mut service, &specs, NodeSpecCase::Root)?;

    create_node(&mut service, &specs, NodeSpecCase::Root).unwrap_err();
    assert_eq!(api::node::tracker::query(&service).nodes().count(), 1);

    Ok(())
}

#[rstest]
fn removing_existing_node_succeeds(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let node_id = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;

    api::node::remove(&mut service, node_id)?;
    assert_eq!(api::node::tracker::query(&service).nodes().count(), 0);
    api::ui::node::query(&service).data(node_id).unwrap_err();

    Ok(())
}

#[rstest]
fn removing_missing_node_fails(mut service: TestEditorService) {
    api::node::remove(&mut service, NodeId::new(999)).unwrap_err();
}

#[rstest]
fn node_spec_query_by_node_id(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let sender = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;

    let by_node_id = api::node::spec::by_node_id(&service);
    assert_eq!(by_node_id.name(root)?.0, "Root");
    assert_eq!(by_node_id.name(sender)?.0, "SenderA");
    by_node_id.ports(root).unwrap_err();
    assert_eq!(by_node_id.ports(sender)?.sender_ids().count(), 1);
    by_node_id.spec(NodeId::new(999)).unwrap_err();

    Ok(())
}

#[rstest]
fn node_spec_query_by_spec_id(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let _root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let _sender = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;

    let by_spec_id = api::node::spec::by_spec_id(&service);
    assert_eq!(by_spec_id.name(NodeSpecId::new(0))?.0, "Root");
    assert_eq!(by_spec_id.name(NodeSpecId::new(1))?.0, "SenderA");
    by_spec_id.spec(NodeSpecId::new(999)).unwrap_err();

    Ok(())
}
