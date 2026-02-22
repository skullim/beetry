use anyhow::{Result, anyhow};
use beetry_core::MessageHash;
use beetry_editor_backend::{EditorService, NodeSpecMap};
use beetry_editor_types::output::channel::{ChannelConfigInput, ChannelKind, TokioChannelKind};
use beetry_editor_types::output::ui::Point;
use beetry_editor_types::spec::channel::ChannelSpec;
use beetry_editor_types::spec::message::{MessageHashProvider, MessageSpec, MessageTypeProvider};
use beetry_editor_types::spec::node::{
    NodeKind, NodeName, NodePortKind, NodePortSpec, NodeSpec, NodeSpecKey, PortsSpec,
};
use mitsein::iter1::FromIterator1;
use rstest::fixture;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeSpecCase {
    Root,
    Control,
    Decorator,
    ActionStub,
    SenderA,
    ReceiverA,
    DuplexA,
    SenderB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelSpecCase {
    MessageA,
    MessageB,
}

#[derive(Debug, Clone)]
pub struct TestSpecs {
    node_specs: HashMap<NodeSpecCase, NodeSpec>,
    channel_specs: HashMap<ChannelSpecCase, ChannelSpec>,
}

impl Default for TestSpecs {
    fn default() -> Self {
        let node_specs = HashMap::from([
            (
                NodeSpecCase::Root,
                NodeSpec::builder()
                    .key(NodeSpecKey::new(NodeName::new("Root"), NodeKind::Root))
                    .build(),
            ),
            (
                NodeSpecCase::Control,
                NodeSpec::builder()
                    .key(NodeSpecKey::new(
                        NodeName::new("ControlA"),
                        NodeKind::Control,
                    ))
                    .build(),
            ),
            (
                NodeSpecCase::Decorator,
                NodeSpec::builder()
                    .key(NodeSpecKey::new(
                        NodeName::new("DecoratorA"),
                        NodeKind::Decorator,
                    ))
                    .build(),
            ),
            (
                NodeSpecCase::ActionStub,
                NodeSpec::builder()
                    .key(NodeSpecKey::new(
                        NodeName::new("ActionStub"),
                        NodeKind::action(),
                    ))
                    .build(),
            ),
            (
                NodeSpecCase::SenderA,
                NodeSpec::builder()
                    .key(NodeSpecKey::new(
                        NodeName::new("SenderA"),
                        NodeKind::action(),
                    ))
                    .ports(PortsSpec::from_iter1([NodePortSpec {
                        kind: NodePortKind::Sender,
                        msg_spec: MessageSpec::new::<TestMessageA>("sender-a"),
                    }]))
                    .build(),
            ),
            (
                NodeSpecCase::ReceiverA,
                NodeSpec::builder()
                    .key(NodeSpecKey::new(
                        NodeName::new("ReceiverA"),
                        NodeKind::action(),
                    ))
                    .ports(PortsSpec::from_iter1([NodePortSpec {
                        kind: NodePortKind::Receiver,
                        msg_spec: MessageSpec::new::<TestMessageA>("receiver-a"),
                    }]))
                    .build(),
            ),
            (
                NodeSpecCase::DuplexA,
                NodeSpec::builder()
                    .key(NodeSpecKey::new(
                        NodeName::new("DuplexA"),
                        NodeKind::action(),
                    ))
                    .ports(PortsSpec::from_iter1([
                        NodePortSpec {
                            kind: NodePortKind::Sender,
                            msg_spec: MessageSpec::new::<TestMessageA>("duplex-a-sender"),
                        },
                        NodePortSpec {
                            kind: NodePortKind::Receiver,
                            msg_spec: MessageSpec::new::<TestMessageA>("duplex-a-receiver"),
                        },
                    ]))
                    .build(),
            ),
            (
                NodeSpecCase::SenderB,
                NodeSpec::builder()
                    .key(NodeSpecKey::new(
                        NodeName::new("SenderB"),
                        NodeKind::action(),
                    ))
                    .ports(PortsSpec::from_iter1([NodePortSpec {
                        kind: NodePortKind::Sender,
                        msg_spec: MessageSpec::new::<TestMessageB>("sender-b"),
                    }]))
                    .build(),
            ),
        ]);

        let channel_specs = HashMap::from([
            (
                ChannelSpecCase::MessageA,
                ChannelSpec::new::<TestMessageA>(),
            ),
            (
                ChannelSpecCase::MessageB,
                ChannelSpec::new::<TestMessageB>(),
            ),
        ]);

        Self {
            node_specs,
            channel_specs,
        }
    }
}

impl TestSpecs {
    pub fn node_spec(&self, fixture: NodeSpecCase) -> Result<&NodeSpec> {
        self.node_specs
            .get(&fixture)
            .ok_or_else(|| anyhow!("missing node spec fixture: {fixture:?}"))
    }

    #[allow(dead_code)]
    pub fn channel_spec(&self, fixture: ChannelSpecCase) -> Result<&ChannelSpec> {
        self.channel_specs
            .get(&fixture)
            .ok_or_else(|| anyhow!("missing channel spec fixture: {fixture:?}"))
    }

    pub fn node_spec_map(&self) -> NodeSpecMap {
        self.node_specs
            .values()
            .cloned()
            .map(|spec| (spec.key().clone(), spec))
            .collect()
    }

    #[allow(dead_code)]
    pub fn default_mpsc_config(&self) -> ChannelConfigInput {
        ChannelConfigInput::new(8, ChannelKind::Tokio(TokioChannelKind::Mpsc))
    }

    #[allow(dead_code)]
    pub fn default_broadcast_config(&self) -> ChannelConfigInput {
        ChannelConfigInput::new(8, ChannelKind::Tokio(TokioChannelKind::Broadcast))
    }
}

pub type TestEditorService = EditorService;

#[fixture]
pub fn specs() -> TestSpecs {
    TestSpecs::default()
}

#[fixture]
pub fn service() -> TestEditorService {
    TestEditorService::new(specs().node_spec_map())
}

#[fixture]
pub fn node_position() -> Point {
    Point { x: 0.0, y: 0.0 }
}

#[fixture]
pub fn channel_position() -> Point {
    Point { x: 0.0, y: 0.0 }
}

#[derive(Debug)]
struct TestMessageA;

impl MessageHashProvider for TestMessageA {
    fn hash() -> MessageHash {
        MessageHash::new(0xBEE7)
    }
}

impl MessageTypeProvider for TestMessageA {
    fn as_str() -> &'static str {
        "TestMessageA"
    }
}

#[derive(Debug)]
struct TestMessageB;

impl MessageHashProvider for TestMessageB {
    fn hash() -> MessageHash {
        MessageHash::new(0xBEE8)
    }
}

impl MessageTypeProvider for TestMessageB {
    fn as_str() -> &'static str {
        "TestMessageB"
    }
}
