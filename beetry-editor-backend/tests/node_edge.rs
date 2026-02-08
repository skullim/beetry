mod common;

use anyhow::Result;
use beetry_editor_backend::api;
use beetry_editor_backend::api::{EdgeQueryView, NodeTrackerQueryView};
use beetry_editor_types::id::NodeId;
use beetry_editor_types::output::edge::NodeEdge;
use beetry_editor_types::output::ui::{NodeUiData, Point};
use common::{NodeSpecCase, TestEditorService, TestSpecs, node_position, service, specs};
use rstest::rstest;

#[rstest]
fn create_nodes(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let cases = [
        NodeSpecCase::Root,
        NodeSpecCase::Control,
        NodeSpecCase::Decorator,
        NodeSpecCase::SenderA,
        NodeSpecCase::ReceiverA,
        NodeSpecCase::DuplexA,
        NodeSpecCase::SenderB,
    ];

    for case in cases {
        let _ = api::node::create(
            &mut service,
            specs.node_spec(case)?,
            NodeUiData {
                position: node_position,
            },
        )?;
    }

    let tracker = api::node::tracker::query_view(&service);
    assert_eq!(tracker.nodes().count(), cases.len());

    Ok(())
}

#[rstest]
fn removing_existing_node_succeeds(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let node_id = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: node_position,
        },
    )?;

    api::node::remove(&mut service, node_id)?;

    assert!(
        !api::node::tracker::query_view(&service)
            .nodes()
            .any(|id| *id == node_id)
    );

    Ok(())
}

#[rstest]
fn removing_missing_node_fails(mut service: TestEditorService) {
    let result = api::node::remove(&mut service, NodeId::new(999));
    assert!(result.is_err());
}

#[rstest]
fn create_edge(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let parent = api::node::create(
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

    let edge_id = api::edge::create(
        &mut service,
        NodeEdge {
            from: parent,
            to: child,
        },
    )?;

    let edge_query = api::edge::borrow(&service);
    assert_eq!(edge_query.edges().count(), 1);
    assert!(
        edge_query
            .edges()
            .any(|(id, e)| *id == edge_id && e.from == parent && e.to == child)
    );
    assert_eq!(edge_query.parent_of(child), Some(&parent));
    assert!(edge_query.children_of(parent).any(|id| *id == child));

    Ok(())
}

#[rstest]
fn removing_existing_edge_succeeds(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let parent = api::node::create(
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

    let edge_id = api::edge::create(
        &mut service,
        NodeEdge {
            from: parent,
            to: child,
        },
    )?;

    api::edge::remove(&mut service, edge_id)?;
    assert!(api::edge::borrow(&service).edges().next().is_none());

    Ok(())
}

#[rstest]
fn removing_edge_twice_fails(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let parent = api::node::create(
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

    let edge_id = api::edge::create(
        &mut service,
        NodeEdge {
            from: parent,
            to: child,
        },
    )?;

    api::edge::remove(&mut service, edge_id)?;

    let second_remove = api::edge::remove(&mut service, edge_id);
    assert!(second_remove.is_err());

    Ok(())
}

#[rstest]
fn ensure_leaves_cannot_connect(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let leaf_a = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderA)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let leaf_b = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::SenderB)?,
        NodeUiData {
            position: node_position,
        },
    )?;

    assert!(
        api::edge::create(
            &mut service,
            NodeEdge {
                from: leaf_a,
                to: leaf_b,
            },
        )
        .is_err()
    );
    Ok(())
}

#[rstest]
fn ensure_root_has_no_parent(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let control = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Control)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let root = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Root)?,
        NodeUiData {
            position: node_position,
        },
    )?;

    let result = api::edge::create(
        &mut service,
        NodeEdge {
            from: control,
            to: root,
        },
    );
    assert!(result.is_err());
    Ok(())
}

#[rstest]
fn ensure_no_edge_cycles(
    mut service: TestEditorService,
    specs: TestSpecs,
    node_position: Point,
) -> Result<()> {
    let a = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Control)?,
        NodeUiData {
            position: node_position,
        },
    )?;
    let b = api::node::create(
        &mut service,
        specs.node_spec(NodeSpecCase::Decorator)?,
        NodeUiData {
            position: node_position,
        },
    )?;

    api::edge::create(&mut service, NodeEdge { from: a, to: b })?;
    let result = api::edge::create(&mut service, NodeEdge { from: b, to: a });
    assert!(result.is_err());
    Ok(())
}
