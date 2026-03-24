use anyhow::Result;
use beetry_core::{ActionTask, BoxNode, Node, NonEmptyNodes, TickStatus};
use beetry_editor_types::spec::node::{
    FieldDefinition, FieldMetadata, FieldTypeSpec, NodeKind, NodeSpec, ParamsSpec, PortKey,
};
use beetry_macros::Message;
use beetry_message::Message;
use beetry_plugin::{Plugin, ProvideParamSpec, action, condition, control, decorator};
use bon::Builder;
use mitsein::iter1::IntoIterator1;
use serde::Deserialize;
use type_hash::TypeHash;

#[derive(TypeHash, Message)]
struct MsgA;

#[derive(TypeHash, Message)]
struct MsgB;

#[derive(Deserialize)]
struct TypedParams {
    _level: f64,
}

impl ProvideParamSpec for TypedParams {
    fn provide() -> ParamsSpec {
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
    use beetry_core::ActionBehavior;

    use super::*;
    macro_rules! stub_action {
        ($name:ident) => {
            struct $name;

            impl ActionBehavior for $name {
                fn task(&mut self) -> Result<ActionTask> {
                    todo!()
                }
            }
        };
        ($name:ident { $($field:ident : $ty:ty),+ $(,)? }) => {
            struct $name {
                $( $field: $ty, )+
            }

            impl $name {
                fn new($($field: $ty),+) -> Self {
                    Self { $($field),+ }
                }
            }

            impl ActionBehavior for $name {
                fn task(&mut self) -> Result<ActionTask> {
                    todo!()
                }
            }
        };
        ($name:ident < $($generic:ident),+ > { $($field:ident : $ty:ty),+ $(,)? }) => {
            struct $name<$($generic),+> {
                $( $field: $ty, )+
            }

            impl<$($generic),+> $name<$($generic),+> {
                fn new($($field: $ty),+) -> Self {
                    Self { $($field),+ }
                }
            }

            impl<$($generic),+> ActionBehavior for $name<$($generic),+> {
                fn task(&mut self) -> Result<ActionTask> {
                    todo!()
                }
            }
        };
    }
    stub_action!(Stub);
    stub_action!(StubWithParam {
        _params: TypedParams
    });
    stub_action!(StubWithPorts<R, S> { _rx: R, _tx: S });
    stub_action!(StubWithReceivers<R1, R2> { _rx_a: R1, _rx_b: R2 });
    stub_action!(
        StubWithPortsAndParam<RX1, RX2, TX1, TX2> {
            _rx_a: RX1,
            _rx_b: RX2,
            _tx_a: TX1,
            _tx_b: TX2,
            _params: TypedParams
        }
    );

    const ACTION_MINIMAL_NAME: &str = "Test Action Minimal";
    const ACTION_SINGLE_PORTS_NAME: &str = "Test Action Single Ports";
    const ACTION_MULTI_PORTS_PARAMS_NAME: &str = "Test Action Multi Ports And Params";
    const ACTION_TYPED_PARAMS_NAME: &str = "Test Action Typed Params";
    const ACTION_DUPLICATE_TYPED_RECEIVERS_NAME: &str = "Test Action Duplicate Typed Receivers";

    action! {
        TestActionMinimalPlugin: ACTION_MINIMAL_NAME;
        create: Stub;
    }

    action! {
        TestActionSinglePortsPlugin: ACTION_SINGLE_PORTS_NAME;
        receivers: [rx: MsgA => "rx a"];
        senders: [tx: MsgB => "tx b"];
        create: StubWithPorts::new(rx, tx);
    }

    action! {
        TestActionMultiPortsAndParamsPlugin: ACTION_MULTI_PORTS_PARAMS_NAME;
        receivers: [rx_a: MsgA => "rx a", rx_b: MsgB => "rx b"];
        senders: [tx_a: MsgA => "tx a", tx_b: MsgB => "tx b"];
        params(parameters): TypedParams;
        create: StubWithPortsAndParam::new(rx_a, rx_b, tx_a, tx_b, parameters);
    }

    action! {
        TestActionSameTypeReceiversPlugin: ACTION_DUPLICATE_TYPED_RECEIVERS_NAME;
        receivers: [rx_a: MsgA => "rx a", rx_b: MsgA => "rx b"];
        create: StubWithReceivers::new(rx_a, rx_b);
    }

    action! {
        TestActionTypedParamsPlugin: ACTION_TYPED_PARAMS_NAME;
        params(parameters): TypedParams;
        create: StubWithParam::new(parameters);
    }

    #[test]
    fn registers_minimal_plugin() {
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
    fn registers_single_receiver_sender_plugin() {
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
    fn registers_multiple_ports_and_params_plugin() {
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

    #[test]
    fn uses_identifier_as_port_key() {
        let plugin = TestActionSameTypeReceiversPlugin::new();
        let ports = plugin.spec().ports().as_ref().unwrap();

        assert_eq!(ports.specs().next().unwrap().key, PortKey::new("rx_a"));
        assert_eq!(ports.specs().nth(1).unwrap().key, PortKey::new("rx_b"));
    }

    #[test]
    fn accepts_typed_params_shorthand() {
        let plugin = TestActionTypedParamsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(ACTION_TYPED_PARAMS_NAME)
                .kind(NodeKind::action())
                .has_params(true)
                .build(),
        );
    }
}

mod condition_fixture {
    use beetry_core::ConditionBehavior;

    use super::*;
    macro_rules! stub_condition {
        ($name:ident) => {
            struct $name;

            impl ConditionBehavior for $name {
                fn cond(&mut self) -> bool {
                    todo!()
                }
            }
        };
        ($name:ident { $($field:ident : $ty:ty),+ $(,)? }) => {
            struct $name {
                $( $field: $ty, )+
            }

            impl $name {
                fn new($($field: $ty),+) -> Self {
                    Self { $($field),+ }
                }
            }

            impl ConditionBehavior for $name {
                fn cond(&mut self) -> bool {
                    todo!()
                }
            }
        };
        ($name:ident < $($generic:ident),+ > { $($field:ident : $ty:ty),+ $(,)? }) => {
            struct $name<$($generic),+> {
                $( $field: $ty, )+
            }

            impl<$($generic),+> $name<$($generic),+> {
                fn new($($field: $ty),+) -> Self {
                    Self { $($field),+ }
                }
            }

            impl<$($generic),+> ConditionBehavior for $name<$($generic),+> {
                fn cond(&mut self) -> bool {
                    todo!()
                }
            }
        };
    }

    stub_condition!(Stub);
    stub_condition!(StubWithParam {
        _params: TypedParams
    });
    stub_condition!(StubWithPorts<R, S> { _rx: R, _tx: S });
    stub_condition!(StubWithReceivers<R1, R2> { _rx_a: R1, _rx_b: R2 });
    stub_condition!(
        StubWithPortsAndParam<RX1, RX2, TX1, TX2> {
            _rx_a: RX1,
            _rx_b: RX2,
            _tx_a: TX1,
            _tx_b: TX2,
            _params: TypedParams
        }
    );

    const CONDITION_MINIMAL_NAME: &str = "Test Condition Minimal";
    const CONDITION_SINGLE_PORTS_NAME: &str = "Test Condition Single Ports";
    const CONDITION_MULTI_PORTS_PARAMS_NAME: &str = "Test Condition Multi Ports And Params";
    const CONDITION_DUPLICATE_TYPED_RECEIVERS_NAME: &str =
        "Test Condition Duplicate Typed Receivers";
    const CONDITION_TYPED_PARAMS_NAME: &str = "Test Condition Typed Params";

    condition! {
        TestConditionMinimalPlugin: CONDITION_MINIMAL_NAME;
        create: Stub;
    }

    condition! {
        TestConditionSinglePortsPlugin: CONDITION_SINGLE_PORTS_NAME;
        receivers: [rx: MsgA => "rx a"];
        senders: [tx: MsgB => "tx b"];
        create: StubWithPorts::new(rx, tx);
    }

    condition! {
        TestConditionMultiPortsAndParamsPlugin: CONDITION_MULTI_PORTS_PARAMS_NAME;
        receivers: [rx_a: MsgA => "rx a", rx_b: MsgB => "rx b"];
        senders: [tx_a: MsgA => "tx a", tx_b: MsgB => "tx b"];
        params(parameters): TypedParams;
        create: StubWithPortsAndParam::new(rx_a, rx_b, tx_a, tx_b, parameters);
    }

    condition! {
        TestConditionSameTypeReceiversPlugin: CONDITION_DUPLICATE_TYPED_RECEIVERS_NAME;
        receivers: [rx_a: MsgA => "rx a", rx_b: MsgA => "rx b"];
        create: StubWithReceivers::new(rx_a, rx_b);
    }

    condition! {
        TestConditionTypedParamsPlugin: CONDITION_TYPED_PARAMS_NAME;
        params(parameters): TypedParams;
        create: StubWithParam::new(parameters);
    }

    #[test]
    fn builds_minimal_plugin_spec() {
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
    fn builds_single_receiver_sender_ports() {
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
    fn builds_multiple_ports_and_params() {
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
    fn uses_identifier_as_port_key() {
        let plugin = TestConditionSameTypeReceiversPlugin::new();
        let ports = plugin.spec().ports().as_ref().unwrap();

        assert_eq!(ports.specs().next().unwrap().key, PortKey::new("rx_a"));
        assert_eq!(ports.specs().nth(1).unwrap().key, PortKey::new("rx_b"));
    }

    #[test]
    fn builds_receiver_ports() {
        let plugin = TestConditionSameTypeReceiversPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(CONDITION_DUPLICATE_TYPED_RECEIVERS_NAME)
                .kind(NodeKind::condition())
                .receiver_count(2)
                .build(),
        );
    }

    #[test]
    fn accepts_typed_params_shorthand() {
        let plugin = TestConditionTypedParamsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(CONDITION_TYPED_PARAMS_NAME)
                .kind(NodeKind::condition())
                .has_params(true)
                .build(),
        );
    }
}

mod control_fixture {
    use beetry_editor_types::spec::node::{NodeName, NodeSpecKey};
    use beetry_plugin::node::{ControlFactory, ControlPluginConstructor, ControlReconstructionData};

    use super::*;

    macro_rules! stub_control {
        ($name:ident) => {
            struct $name;

            impl Node for $name {
                fn tick(&mut self) -> TickStatus {
                    todo!()
                }
            }
        };
        ($name:ident { $($field:ident : $ty:ty),+ $(,)? }) => {
            struct $name {
                $( $field: $ty, )+
            }

            impl $name {
                fn new($($field: $ty),+) -> Self {
                    Self { $($field),+ }
                }
            }

            impl Node for $name {
                fn tick(&mut self) -> TickStatus {
                    todo!()
                }
            }
        };
    }

    stub_control!(StubWithChildren { _children: NonEmptyNodes });
    stub_control!(StubWithChildrenAndParam {
        _children: NonEmptyNodes,
        _params: TypedParams
    });

    const CONTROL_MINIMAL_NAME: &str = "Test Control Minimal";
    const CONTROL_TYPED_PARAMS_NAME: &str = "Test Control Typed Params";

    control!(
        TestControlMinimalPlugin: CONTROL_MINIMAL_NAME;
        children(children),
        create: StubWithChildren::new(children),
    );

    control!(
        TestControlTypedParamsPlugin: CONTROL_TYPED_PARAMS_NAME;
        children(children),
        params(parameters): TypedParams,
        create: StubWithChildrenAndParam::new(children, parameters),
    );

    #[test]
    fn registers_minimal_plugin() {
        let plugin = TestControlMinimalPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(CONTROL_MINIMAL_NAME)
                .kind(NodeKind::Control)
                .build(),
        );
    }

    #[test]
    fn accepts_typed_params_shorthand() {
        let plugin = TestControlTypedParamsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(CONTROL_TYPED_PARAMS_NAME)
                .kind(NodeKind::Control)
                .has_params(true)
                .build(),
        );
    }
}

mod decorator_fixture {
    use beetry_editor_types::spec::node::{NodeName, NodeSpecKey};
    use beetry_plugin::node::{
        DecoratorFactory, DecoratorPluginConstructor, DecoratorReconstructionData,
    };

    use super::*;

    macro_rules! stub_decorator {
        ($name:ident) => {
            struct $name;

            impl Node for $name {
                fn tick(&mut self) -> TickStatus {
                    todo!()
                }
            }
        };
        ($name:ident { $($field:ident : $ty:ty),+ $(,)? }) => {
            struct $name {
                $( $field: $ty, )+
            }

            impl $name {
                fn new($($field: $ty),+) -> Self {
                    Self { $($field),+ }
                }
            }

            impl Node for $name {
                fn tick(&mut self) -> TickStatus {
                    todo!()
                }
            }
        };
    }

    stub_decorator!(StubWithChild { _child: BoxNode });
    stub_decorator!(StubWithChildAndParam {
        _child: BoxNode,
        _params: TypedParams
    });

    const DECORATOR_MINIMAL_NAME: &str = "Test Decorator Minimal";
    const DECORATOR_TYPED_PARAMS_NAME: &str = "Test Decorator Typed Params";

    decorator!(
        TestDecoratorMinimalPlugin: DECORATOR_MINIMAL_NAME;
        child(child),
        create: StubWithChild::new(child),
    );

    decorator!(
        TestDecoratorTypedParamsPlugin: DECORATOR_TYPED_PARAMS_NAME;
        child(child),
        params(parameters): TypedParams,
        create: StubWithChildAndParam::new(child, parameters),
    );

    #[test]
    fn registers_minimal_plugin() {
        let plugin = TestDecoratorMinimalPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(DECORATOR_MINIMAL_NAME)
                .kind(NodeKind::Decorator)
                .build(),
        );
    }

    #[test]
    fn accepts_typed_params_shorthand() {
        let plugin = TestDecoratorTypedParamsPlugin::new();
        assert_plugin_meta(
            &plugin,
            Expected::builder()
                .name(DECORATOR_TYPED_PARAMS_NAME)
                .kind(NodeKind::Decorator)
                .has_params(true)
                .build(),
        );
    }
}
