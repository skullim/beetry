mod common;

use anyhow::Result;
use beetry_editor_backend::api;
use beetry_editor_backend::api::EdgeQueryView;
use beetry_editor_types::output::edge::NodeEdge;
use common::{NodeSpecCase, TestEditorService, TestSpecs, create_node, service, specs};
use rstest::rstest;

#[rstest]
fn creating_edge_succeeds(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let parent = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let child = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;

    let edge_id = api::edge::create(
        &mut service,
        NodeEdge {
            from: parent,
            to: child,
        },
    )?;

    let edge_query = api::edge::query(&service);
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
fn removing_existing_edge_succeeds(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let parent = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let child = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;

    let edge_id = api::edge::create(
        &mut service,
        NodeEdge {
            from: parent,
            to: child,
        },
    )?;

    api::edge::remove(&mut service, edge_id)?;
    assert!(
        api::edge::query(&service)
            .children_of(parent)
            .next()
            .is_none()
    );

    Ok(())
}

#[rstest]
fn removing_same_edge_twice_fails(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let parent = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let child = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;

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
fn ensure_leaves_cannot_connect(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let leaf_a = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let leaf_b = create_node(&mut service, &specs, NodeSpecCase::SenderB)?;

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
fn ensure_root_has_no_parent(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let control = create_node(&mut service, &specs, NodeSpecCase::Control)?;
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;

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
fn ensure_no_edge_cycles(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let a = create_node(&mut service, &specs, NodeSpecCase::Control)?;
    let b = create_node(&mut service, &specs, NodeSpecCase::Decorator)?;

    api::edge::create(&mut service, NodeEdge { from: a, to: b })?;
    let result = api::edge::create(&mut service, NodeEdge { from: b, to: a });
    assert!(result.is_err());
    Ok(())
}

#[rstest]
fn reparenting_preserves_old_parent_children(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let old_parent = create_node(&mut service, &specs, NodeSpecCase::Control)?;
    let new_parent = create_node(&mut service, &specs, NodeSpecCase::Control)?;
    let child_to_move = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let child_to_keep = create_node(&mut service, &specs, NodeSpecCase::ReceiverA)?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: old_parent,
            to: child_to_move,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: old_parent,
            to: child_to_keep,
        },
    )?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: new_parent,
            to: child_to_move,
        },
    )?;

    let query = api::edge::query(&service);
    assert_eq!(query.parent_of(child_to_move), Some(&new_parent));
    assert_eq!(query.parent_of(child_to_keep), Some(&old_parent));
    assert!(query.children_of(old_parent).any(|id| *id == child_to_keep));
    assert!(!query.children_of(old_parent).any(|id| *id == child_to_move));

    Ok(())
}

#[rstest]
fn root_replacing_child_disconnects_previous_child(
    mut service: TestEditorService,
    specs: TestSpecs,
) -> Result<()> {
    let root = create_node(&mut service, &specs, NodeSpecCase::Root)?;
    let first_child = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    let second_child = create_node(&mut service, &specs, NodeSpecCase::ReceiverA)?;

    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: first_child,
        },
    )?;
    api::edge::create(
        &mut service,
        NodeEdge {
            from: root,
            to: second_child,
        },
    )?;

    let query = api::edge::query(&service);
    assert_eq!(query.parent_of(first_child), None);
    assert_eq!(query.parent_of(second_child), Some(&root));

    let children: Vec<_> = query.children_of(root).copied().collect();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], second_child);

    Ok(())
}
