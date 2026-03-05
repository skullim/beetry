mod common;

use anyhow::Result;
use beetry_editor_backend::api;
use beetry_editor_backend::api::ParameterValueQuery;
use beetry_editor_types::id::NodeId;
use beetry_editor_types::output::node::{ParameterValue, Parameters};
use common::{NodeSpecCase, TestEditorService, TestSpecs, create_node, service, specs};
use rstest::rstest;

#[rstest]
fn create_and_query_params(mut service: TestEditorService, specs: TestSpecs) -> Result<()> {
    let node_id = create_node(&mut service, &specs, NodeSpecCase::SenderA)?;
    {
        let query = api::node::parameters::query(&service);
        assert!(query.parameters(node_id).is_err());
    }

    let mut params = Parameters::default();
    params.insert("attempts".to_string(), ParameterValue::U64(3));
    params.insert("enabled".to_string(), ParameterValue::Bool(true));
    api::node::parameters::create(&mut service, node_id, params);

    let query = api::node::parameters::query(&service);
    let stored = query.parameters(node_id)?;
    let attempts = stored.get(&"attempts".to_string());
    assert!(matches!(attempts, Some(ParameterValue::U64(3))));
    let enabled = stored.get(&"enabled".to_string());
    assert!(matches!(enabled, Some(ParameterValue::Bool(true))));
    assert!(query.parameters(NodeId::new(999)).is_err());

    Ok(())
}
