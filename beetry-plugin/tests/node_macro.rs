use anyhow::Result;
use beetry_core::{ActionBehavior, ConditionBehavior, NodeTask};
use beetry_editor_types::spec::node::{
    FieldDefinition, FieldMetadata, FieldTypeSpec, NodeKind, NodeSpec, ParamsSpec,
};
use beetry_message::Message;
use beetry_plugin::{Plugin, action, condition};
use bon::Builder;
use mitsein::iter1::IntoIterator1;
use type_hash::TypeHash;

struct StubAction;

impl ActionBehavior for StubAction {
    fn task(&mut self) -> Result<NodeTask> {
        todo!()
    }
}

struct StubCondition;

impl ConditionBehavior for StubCondition {
    fn cond(&mut self) -> bool {
        todo!()
    }
}

#[derive(TypeHash)]
struct MsgA;
impl Message for MsgA {}

#[derive(TypeHash)]
struct MsgB;
impl Message for MsgB {}

fn test_params_spec() -> ParamsSpec {
    [(
        "level".into(),
        FieldDefinition {
            type_spec: FieldTypeSpec::F64(FieldMetadata::default()),
            description: None,
        },
    )]
    .into_iter1()
    .collect1()
}

#[derive(Builder, Clone, Copy)]
struct Expected<'a> {
    name: &'a str,
    kind: NodeKind,
    #[builder(default)]
    has_params: bool,
    #[builder(default)]
    receiver_count: usize,
    #[builder(default)]
    sender_count: usize,
}

fn assert_plugin_meta<F>(
    plugin: &impl Plugin<Spec = NodeSpec, Factory = F>,
    expected: Expected<'_>,
) {
    assert_eq!(plugin.spec().name().0, expected.name);
    assert_eq!(plugin.spec().kind(), expected.kind);
    assert_eq!(plugin.spec().has_params(), expected.has_params);

    if let Some(ports) = plugin.spec().ports() {
        assert_eq!(ports.receiver_ids().count(), expected.receiver_count);
        assert_eq!(ports.sender_ids().count(), expected.sender_count);
    } else {
        assert_eq!(expected.receiver_count, 0);
        assert_eq!(expected.sender_count, 0);
    }
}

mod action_fixture {
    use super::*;

    const ACTION_MINIMAL_NAME: &str = "Test Action Minimal";
    const ACTION_OPTIONAL_FIELDS_NAME: &str = "Test Action Optional Fields";
    const ACTION_SINGLE_PORTS_NAME: &str = "Test Action Single Ports";
    const ACTION_MULTI_PORTS_PARAMS_NAME: &str = "Test Action Multi Ports And Params";

    action! {
        TestActionMinimalPlugin: ACTION_MINIMAL_NAME;
        create: StubAction;
    }

    action! {
        TestActionWithOptionalFieldsPlugin: ACTION_OPTIONAL_FIELDS_NAME;
        params(parameters): test_params_spec();
        create: {
            let _ = &parameters;
            StubAction
        };
    }

    action! {
        TestActionSinglePortsPlugin: ACTION_SINGLE_PORTS_NAME;
        receivers: [rx: MsgA => "rx a"];
        senders: [tx: MsgB => "tx b"];
        create: {
            let _ = (&rx, &tx);
            StubAction
        };
    }

    action! {
        TestActionMultiPortsAndParamsPlugin: ACTION_MULTI_PORTS_PARAMS_NAME;
        receivers: [rx_a: MsgA => "rx a", rx_b: MsgB => "rx b"];
        senders: [tx_a: MsgA => "tx a", tx_b: MsgB => "tx b"];
        params: test_params_spec();
        create: {
            let _ = (&rx_a, &rx_b, &tx_a, &tx_b);
            StubAction
        };
    }

    #[test]
    fn action_macro_builds_minimal_plugin_spec() {
        let plugin = TestActionMinimalPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(ACTION_MINIMAL_NAME)
                .kind(NodeKind::action())
                .build(),
        );
    }

    #[test]
    fn action_macro_accepts_optional_fields() {
        let plugin = TestActionWithOptionalFieldsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(ACTION_OPTIONAL_FIELDS_NAME)
                .kind(NodeKind::action())
                .has_params(true)
                .build(),
        );
    }

    #[test]
    fn action_macro_builds_single_receiver_sender_ports() {
        let plugin = TestActionSinglePortsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(ACTION_SINGLE_PORTS_NAME)
                .kind(NodeKind::action())
                .receiver_count(1)
                .sender_count(1)
                .build(),
        );
    }

    #[test]
    fn action_macro_builds_multiple_ports_and_params() {
        let plugin = TestActionMultiPortsAndParamsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(ACTION_MULTI_PORTS_PARAMS_NAME)
                .kind(NodeKind::action())
                .has_params(true)
                .receiver_count(2)
                .sender_count(2)
                .build(),
        );
    }
}

mod condition_fixture {
    use super::*;

    const CONDITION_MINIMAL_NAME: &str = "Test Condition Minimal";
    const CONDITION_OPTIONAL_FIELDS_NAME: &str = "Test Condition Optional Fields";
    const CONDITION_SINGLE_PORTS_NAME: &str = "Test Condition Single Ports";
    const CONDITION_MULTI_PORTS_PARAMS_NAME: &str = "Test Condition Multi Ports And Params";
    const CONDITION_SINGLE_RECEIVER_NAME: &str = "Test Condition Single Receiver";

    condition! {
        TestConditionMinimalPlugin: CONDITION_MINIMAL_NAME;
        create: StubCondition;
    }

    condition! {
        TestConditionWithOptionalFieldsPlugin: CONDITION_OPTIONAL_FIELDS_NAME;
        params(parameters): test_params_spec();
        create: {
            let _ = &parameters;
            StubCondition
        };
    }

    condition! {
        TestConditionSinglePortsPlugin: CONDITION_SINGLE_PORTS_NAME;
        receivers: [rx: MsgA => "rx a"];
        senders: [tx: MsgB => "tx b"];
        create: {
            let _ = (&rx, &tx);
            StubCondition
        };
    }

    condition! {
        TestConditionMultiPortsAndParamsPlugin: CONDITION_MULTI_PORTS_PARAMS_NAME;
        receivers: [rx_a: MsgA => "rx a", rx_b: MsgB => "rx b"];
        senders: [tx_a: MsgA => "tx a", tx_b: MsgB => "tx b"];
        params: test_params_spec();
        create: {
            let _ = (&rx_a, &rx_b, &tx_a, &tx_b);
            StubCondition
        };
    }

    condition! {
        TestConditionSingleReceiverPlugin: CONDITION_SINGLE_RECEIVER_NAME;
        receivers: [rx: MsgA => "rx a"];
        create: {
            let _ = &rx;
            StubCondition
        };
    }

    #[test]
    fn condition_macro_builds_minimal_plugin_spec() {
        let plugin = TestConditionMinimalPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(CONDITION_MINIMAL_NAME)
                .kind(NodeKind::condition())
                .build(),
        );
    }

    #[test]
    fn condition_macro_accepts_optional_fields() {
        let plugin = TestConditionWithOptionalFieldsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(CONDITION_OPTIONAL_FIELDS_NAME)
                .kind(NodeKind::condition())
                .has_params(true)
                .build(),
        );
    }

    #[test]
    fn condition_macro_builds_single_receiver_sender_ports() {
        let plugin = TestConditionSinglePortsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(CONDITION_SINGLE_PORTS_NAME)
                .kind(NodeKind::condition())
                .receiver_count(1)
                .sender_count(1)
                .build(),
        );
    }

    #[test]
    fn condition_macro_builds_multiple_ports_and_params() {
        let plugin = TestConditionMultiPortsAndParamsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(CONDITION_MULTI_PORTS_PARAMS_NAME)
                .kind(NodeKind::condition())
                .has_params(true)
                .receiver_count(2)
                .sender_count(2)
                .build(),
        );
    }

    #[test]
    fn condition_macro_builds_receiver_ports() {
        let plugin = TestConditionSingleReceiverPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(CONDITION_SINGLE_RECEIVER_NAME)
                .kind(NodeKind::condition())
                .receiver_count(1)
                .build(),
        );
    }
}
