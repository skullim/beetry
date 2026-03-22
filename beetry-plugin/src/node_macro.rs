/// Creates an action plugin from Beetry's leaf-node DSL.
///
/// The generated plugin:
///
/// - defines a `NodeSpec`
/// - defines optional port metadata and parameter metadata
/// - defines a factory that reconstructs the action from reconstruction data
/// - registers itself automatically
///
///
/// Minimal action without params or channels:
///
/// ```rust, no_run
/// # use anyhow::Result;
/// # use beetry_core::{ActionBehavior, NodeTask};
/// # use beetry_plugin::action;
///
/// struct WaitForSignal;
///
/// impl ActionBehavior for WaitForSignal {
///     fn task(&mut self) -> Result<NodeTask> {
///         # todo!()
///     }
/// }
///
/// action! {
///     WaitForSignalPlugin: "Wait For Signal";
///     create: WaitForSignal;
/// }
/// ```
///
/// Action with parameters:
///
/// ```rust, no_run
/// # use anyhow::Result;
/// # use beetry_core::{ActionBehavior, NodeTask};
/// # use beetry_editor_types::spec::node::{
/// #     FieldDefinition, FieldMetadata, FieldTypeSpec, ParamsSpec,
/// # };
/// # use beetry_plugin::{ParamsDeserializer, ProvideParamSpec, action};
/// # use mitsein::iter1::IntoIterator1;
/// # use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct RetryParams {
///     retries: u64,
/// }
///
/// impl ProvideParamSpec for RetryParams {
///     fn provide() -> ParamsSpec {
///         [(
///             "retries".into(),
///             FieldDefinition {
///                 type_spec: FieldTypeSpec::U64(FieldMetadata::default()),
///                 description: Some("Maximum retry count".into()),
///             },
///         )]
///         .into_iter1()
///         .collect1()
///     }
/// }
///
/// struct RetryAction {
///     params: RetryParams,
/// }
///
/// impl RetryAction {
///     fn new(params: RetryParams) -> Self {
///         Self { params }
///     }
/// }
///
/// impl ActionBehavior for RetryAction {
///     fn task(&mut self) -> Result<NodeTask> {
///         # let _ = self.params.retries;
///         # todo!()
///     }
/// }
///
/// action! {
///     RetryActionPlugin: "Retry Action";
///     params(parameters): RetryParams::provide();
///     create: RetryAction::new(ParamsDeserializer::deserialize(parameters)?);
/// }
/// ```
///
/// Action with one receiver and one sender:
///
/// ```rust, no_run
/// # use anyhow::Result;
/// # use beetry_core::{ActionBehavior, NodeTask, Receiver, Sender};
/// # use beetry_macros::Message;
/// # use beetry_message::Message;
/// # use beetry_plugin::action;
/// # use type_hash::TypeHash;
/// #[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
/// struct Pose;
///
/// struct RelayPose<R, S>
/// where
///     R: Receiver<Pose>,
///     S: Sender<Pose>,
/// {
///     input: R,
///     output: S,
/// }
///
/// impl<R, S> RelayPose<R, S>
/// where
///     R: Receiver<Pose>,
///     S: Sender<Pose>,
/// {
///     fn new(input: R, output: S) -> Self {
///         Self { input, output }
///     }
/// }
///
/// impl<R, S> ActionBehavior for RelayPose<R, S>
/// where
///     R: Receiver<Pose>,
///     S: Sender<Pose>,
/// {
///     fn task(&mut self) -> Result<NodeTask> {
///         # let _ = (&mut self.input, &mut self.output);
///         # todo!()
///     }
/// }
///
/// action! {
///     RelayPosePlugin: "Relay Pose";
///     receivers: [input: Pose => "Incoming pose"];
///     senders: [output: Pose => "Republished pose"];
///     create: RelayPose::new(input, output);
/// }
/// ```
/// The DSL supports both parameters and ports, so action nodes that depend on
/// channels and parameters can be defined by combining the examples above.
#[macro_export]
macro_rules! action {
    ($plugin_name:ident : $name:expr; $($tokens:tt)*) => {
        $crate::__leaf_plugin_parse! {
            ctx: {
                plugin_name: $plugin_name,
                node_name: $name,
                factory_type: $crate::node::ActionFactory,
                plugin_constructor: $crate::node::ActionPluginConstructor,
                reconstruction_data: $crate::__macro_support::ActionReconstructionData,
                behavior_box_type: $crate::__macro_support::BoxActionBehavior,
                node_kind: $crate::__macro_support::NodeKind::action(),
            },
            receivers: [],
            senders: [],
            params: none,
            params_binding: _parameters,
            tokens: $($tokens)*
        }
    };
}

/// Creates a condition plugin from Beetry's leaf-node DSL.
///
/// Use when the node evaluates to success or failure without
/// producing child nodes.
///
/// The DSL is the same as [`action!`]: `receivers`, `senders`, `params`, and
/// `create` work the same way, but the constructed behavior must implement
/// `ConditionBehavior` instead of `ActionBehavior`.
#[macro_export]
macro_rules! condition {
    ($plugin_name:ident : $name:expr; $($tokens:tt)*) => {
        $crate::__leaf_plugin_parse! {
            ctx: {
                plugin_name: $plugin_name,
                node_name: $name,
                factory_type: $crate::node::ConditionFactory,
                plugin_constructor: $crate::node::ConditionPluginConstructor,
                reconstruction_data: $crate::__macro_support::ConditionReconstructionData,
                behavior_box_type: $crate::__macro_support::BoxConditionBehavior,
                node_kind: $crate::__macro_support::NodeKind::condition(),
            },
            receivers: [],
            senders: [],
            params: none,
            params_binding: _parameters,
            tokens: $($tokens)*
        }
    };
}

/// Creates a decorator plugin for nodes that wrap a single child.
///
/// Minimal example with a bound child node:
///
/// ```rust, no_run
/// # use beetry_core::{Node, TickStatus};
/// # use beetry_core::BoxNode;
/// # use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};
/// # use beetry_plugin::{
/// #     Plugin,
/// #     decorator,
/// #     node::{DecoratorFactory, DecoratorPluginConstructor, DecoratorReconstructionData},
/// # };
///
/// struct Invert<N>
/// {
///     child: N,
/// }
///
/// impl<N> Invert<N>
/// where
///     N: Node,
/// {
///     fn new(child: N) -> Self {
///         Self { child }
///     }
/// }
///
/// impl<N> Node for Invert<N>
/// where
///     N: Node,
/// {
///     fn tick(&mut self) -> TickStatus {
///     #    match self.child.tick() {
///     #        TickStatus::Success => TickStatus::Failure,
///     #        TickStatus::Failure => TickStatus::Success,
///     #        TickStatus::Running => TickStatus::Running,
///     #    }
///     }
/// }
///
/// decorator!(
///     InvertPlugin: "Invert";
///     child(child),
///     create: Invert::new(child),
/// );
/// ```
///
/// Parameters can be attached the same way as in [`action!`].
#[macro_export]
macro_rules! decorator {
    ($plugin_name:ident : $name:expr; child($child_binding:ident),create: $create:expr,) => {
        $crate::__decorator_plugin_impl! {
            $plugin_name,
            $name,
            $child_binding,
            none,
            _parameters,
            $create
        }
    };
}

/// Creates a control plugin for nodes that manage multiple children.
///
///
/// Minimal example with bound child nodes:
///
/// ```rust, no_run
/// # use beetry_core::{BoxNode, Node, NonEmptyNodes, TickStatus};
/// # use beetry_editor_types::spec::node::{NodeKind, NodeName, NodeSpec, NodeSpecKey};
/// # use beetry_plugin::{
/// #     Plugin,
/// #     control,
/// #     node::{ControlFactory, ControlPluginConstructor, ControlReconstructionData},
/// # };
///
/// struct Sequence {
///     children: NonEmptyNodes,
/// }
///
/// impl Sequence {
///     fn new(children: NonEmptyNodes) -> Self {
///         Self { children }
///     }
/// }
///
/// impl Node for Sequence {
///     fn tick(&mut self) -> TickStatus {
///     #    for child in &mut self.children {
///     #        match child.tick() {
///     #            TickStatus::Success => continue,
///     #            TickStatus::Failure => return TickStatus::Failure,
///     #            TickStatus::Running => return TickStatus::Running,
///     #        }
///     #    }
///     #
///     #    TickStatus::Success
///     }
/// }
///
/// control!(
///     SequencePlugin: "Sequence";
///     children(children),
///     create: Sequence::new(children),
/// );
/// ```
///
/// Parameters are attached the same way as in [`action!`].
#[macro_export]
macro_rules! control {
    ($plugin_name:ident : $name:expr; children($children_binding:ident),create: $create:expr,) => {
        $crate::__control_plugin_impl! {
            $plugin_name,
            $name,
            $children_binding,
            none,
             _parameters,
            $create
        }
    };
    (
        $plugin_name:ident :
        $name:expr; children($children_binding:ident),params($params_binding:ident):
        $params:expr,create:
        $create:expr,
    ) => {
        $crate::__control_plugin_impl! {
            $plugin_name,
            $name,
            $children_binding,
            $params,
            $params_binding,
            $create
        }
    };
    (
        $plugin_name:ident :
        $name:expr; children($children_binding:ident),params:
        $params:expr,create:
        $create:expr,
    ) => {
        $crate::__control_plugin_impl! {
            $plugin_name,
            $name,
            $children_binding,
            $params,
            _parameters,
            $create
        }
    };
}

/// TT-muncher: parses plugin DSL tokens into
/// receivers/senders/params/params_binding and a final `create` expression,
/// then forwards parsed fields to the impl macro.
#[doc(hidden)]
#[macro_export]
macro_rules! __leaf_plugin_parse {
    (
        ctx: $ctx:tt,
        receivers: [$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),* $(,)?],
        senders: [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),* $(,)?],
        params: $params:tt,
        params_binding: $params_binding:ident,
        tokens: receivers: [$($new_receiver_name:ident : $new_receiver_ty:ty => $new_receiver_desc:literal),* $(,)?]; $($rest:tt)*
    ) => {
        $crate::__leaf_plugin_parse! {
            ctx: $ctx,
            receivers: [$($new_receiver_name : $new_receiver_ty => $new_receiver_desc),*],
            senders: [$($sender_name : $sender_ty => $sender_desc),*],
            params: $params,
            params_binding: $params_binding,
            tokens: $($rest)*
        }
    };
    (
        ctx: $ctx:tt,
        receivers: [$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),* $(,)?],
        senders: [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),* $(,)?],
        params: $params:tt,
        params_binding: $params_binding:ident,
        tokens: senders: [$($new_sender_name:ident : $new_sender_ty:ty => $new_sender_desc:literal),* $(,)?]; $($rest:tt)*
    ) => {
        $crate::__leaf_plugin_parse! {
            ctx: $ctx,
            receivers: [$($receiver_name : $receiver_ty => $receiver_desc),*],
            senders: [$($new_sender_name : $new_sender_ty => $new_sender_desc),*],
            params: $params,
            params_binding: $params_binding,
            tokens: $($rest)*
        }
    };
    (
        ctx: $ctx:tt,
        receivers: [$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),* $(,)?],
        senders: [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),* $(,)?],
        params: $params:tt,
        params_binding: $params_binding:ident,
        tokens: params($new_params_binding:ident): $new_params:expr; $($rest:tt)*
    ) => {
        $crate::__leaf_plugin_parse! {
            ctx: $ctx,
            receivers: [$($receiver_name : $receiver_ty => $receiver_desc),*],
            senders: [$($sender_name : $sender_ty => $sender_desc),*],
            params: $new_params,
            params_binding: $new_params_binding,
            tokens: $($rest)*
        }
    };
    (
        ctx: $ctx:tt,
        receivers: [$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),* $(,)?],
        senders: [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),* $(,)?],
        params: $params:tt,
        params_binding: $params_binding:ident,
        tokens: params: $new_params:expr; $($rest:tt)*
    ) => {
        $crate::__leaf_plugin_parse! {
            ctx: $ctx,
            receivers: [$($receiver_name : $receiver_ty => $receiver_desc),*],
            senders: [$($sender_name : $sender_ty => $sender_desc),*],
            params: $new_params,
            params_binding: _parameters,
            tokens: $($rest)*
        }
    };
    (
        ctx: $ctx:tt,
        receivers: [$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),* $(,)?],
        senders: [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),* $(,)?],
        params: $params:tt,
        params_binding: $params_binding:ident,
        tokens: create: $create:expr $(;)?
    ) => {
        $crate::__leaf_plugin_impl! {
            ctx: $ctx,
            receivers: [$($receiver_name : $receiver_ty => $receiver_desc),*],
            senders: [$($sender_name : $sender_ty => $sender_desc),*],
            params: $params,
            params_binding: $params_binding,
            create: $create
        }
    };
    (
        ctx: $ctx:tt,
        receivers: [$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),* $(,)?],
        senders: [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),* $(,)?],
        params: $params:tt,
        params_binding: $params_binding:ident,
        tokens: $($unexpected:tt)+
    ) => {
        compile_error!(
            "invalid plugin DSL. Expected fields separated by ';': \
             receivers: [...]; senders: [...]; params: <expr>; params(<ident>): <expr>; create: <expr>;"
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __leaf_plugin_impl {
    (
        ctx: {
            plugin_name: $plugin_name:ident,
            node_name: $name:expr,
            factory_type: $factory_type:ty,
            plugin_constructor: $plugin_constructor:ty,
            reconstruction_data: $reconstruction_data:ty,
            behavior_box_type: $behavior_box_type:ty,
            node_kind: $node_kind:expr,
        },
        receivers: [$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),* $(,)?],
        senders: [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),* $(,)?],
        params: $params:tt,
        params_binding: $params_binding:ident,
        create: $create:expr
    ) => {
        $crate::__leaf_plugin_impl! {
            plugin_name: $plugin_name,
            node_name: $name,
            factory_type: $factory_type,
            plugin_constructor: $plugin_constructor,
            reconstruction_data: $reconstruction_data,
            behavior_box_type: $behavior_box_type,
            node_kind: $node_kind,
            receivers: [$($receiver_name : $receiver_ty => $receiver_desc),*],
            senders: [$($sender_name : $sender_ty => $sender_desc),*],
            params: $params,
            params_binding: $params_binding,
            create: $create
        }
    };
    (
        plugin_name: $plugin_name:ident,
        node_name: $name:expr,
        factory_type: $factory_type:ty,
        plugin_constructor: $plugin_constructor:ty,
        reconstruction_data: $reconstruction_data:ty,
        behavior_box_type: $behavior_box_type:ty,
        node_kind: $node_kind:expr,
        receivers: [$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),* $(,)?],
        senders: [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),* $(,)?],
        params: $params:tt,
        params_binding: $params_binding:ident,
        create: $create:expr $(,)?
    ) => {
        pub struct $plugin_name {
            spec: $crate::__macro_support::NodeSpec,
            factory: $factory_type,
        }

        impl $crate::Plugin for $plugin_name {
            type Spec = $crate::__macro_support::NodeSpec;
            type Factory = $factory_type;

            fn new() -> Self
            where
                Self: Sized,
            {
                let factory_fn = |mut data: $reconstruction_data| {
                    $crate::__leaf_plugin_extract_receivers!([$($receiver_name : $receiver_ty => $receiver_desc),*], data);
                    $crate::__leaf_plugin_extract_senders!([$($sender_name : $sender_ty => $sender_desc),*], data);
                    let $params_binding = data.parameters;
                    Ok(Box::new($create) as $behavior_box_type)
                };

                let spec = $crate::__macro_support::NodeSpec::builder()
                    .key($crate::__macro_support::NodeSpecKey::new(
                        $crate::__macro_support::NodeName::new($name),
                        $node_kind,
                    ))
                    .maybe_ports($crate::__leaf_plugin_build_ports!(
                        [$($receiver_name : $receiver_ty => $receiver_desc),*],
                        [$($sender_name : $sender_ty => $sender_desc),*]
                    ))
                    .maybe_params($crate::__leaf_plugin_build_params!($params))
                    .build();

                Self {
                    spec,
                    factory: Self::Factory::new(Box::new(factory_fn)),
                }
            }

            fn spec(&self) -> &Self::Spec {
                &self.spec
            }

            fn factory(&self) -> &Self::Factory {
                &self.factory
            }

            fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
                (self.spec, self.factory)
            }
        }

        $crate::submit!(<$plugin_constructor>::new::<$plugin_name>());
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __leaf_plugin_extract_receivers {
    ([], $data:ident) => {};
    ([$($port_name:ident : $port_ty:ty => $port_desc:literal),+], $data:ident) => {
        $(
            let $port_name = $data
                .context
                .take_receiver(&$crate::__macro_support::PortKey::new(stringify!($port_name)))?
                .into_receiver_of::<$port_ty>()
                .map_err(|_| {
                    $crate::__macro_support::anyhow::anyhow!(
                        concat!(
                            "failed to obtain typed receiver for port '",
                            stringify!($port_name),
                            "'"
                        )
                    )
                })?;
        )+
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __leaf_plugin_extract_senders {
    ([], $data:ident) => {};
    ([$($port_name:ident : $port_ty:ty => $port_desc:literal),+], $data:ident) => {
        $(
            let $port_name = $data
                .context
                .take_sender(&$crate::__macro_support::PortKey::new(stringify!($port_name)))?
                .into_sender_of::<$port_ty>()
                .map_err(|_| {
                    $crate::__macro_support::anyhow::anyhow!(
                        concat!(
                            "failed to obtain typed sender for port '",
                            stringify!($port_name),
                            "'"
                        )
                    )
                })?;
        )+
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __leaf_plugin_build_ports {
    ([], []) => {
        None
    };
    ([$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),+], []) => {
        Some(
            <$crate::__macro_support::PortsSpec as $crate::__macro_support::FromIterator1<
                $crate::__macro_support::NodePortSpec,
            >>::from_iter1([
                $(
                    $crate::__macro_support::NodePortSpec {
                        key: $crate::__macro_support::PortKey::new(stringify!($receiver_name)),
                        kind: $crate::__macro_support::NodePortKind::Receiver,
                        msg_spec: $crate::__macro_support::MessageSpec::new::<$receiver_ty>($receiver_desc),
                    }
                ),+
            ]),
        )
    };
    ([], [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),+]) => {
        Some(
            <$crate::__macro_support::PortsSpec as $crate::__macro_support::FromIterator1<
                $crate::__macro_support::NodePortSpec,
            >>::from_iter1([
                $(
                    $crate::__macro_support::NodePortSpec {
                        key: $crate::__macro_support::PortKey::new(stringify!($sender_name)),
                        kind: $crate::__macro_support::NodePortKind::Sender,
                        msg_spec: $crate::__macro_support::MessageSpec::new::<$sender_ty>($sender_desc),
                    }
                ),+
            ]),
        )
    };
    ([$($receiver_name:ident : $receiver_ty:ty => $receiver_desc:literal),+], [$($sender_name:ident : $sender_ty:ty => $sender_desc:literal),+]) => {
        Some(
            <$crate::__macro_support::PortsSpec as $crate::__macro_support::FromIterator1<
                $crate::__macro_support::NodePortSpec,
            >>::from_iter1([
                $(
                    $crate::__macro_support::NodePortSpec {
                        key: $crate::__macro_support::PortKey::new(stringify!($receiver_name)),
                        kind: $crate::__macro_support::NodePortKind::Receiver,
                        msg_spec: $crate::__macro_support::MessageSpec::new::<$receiver_ty>($receiver_desc),
                    }
                ),+,
                $(
                    $crate::__macro_support::NodePortSpec {
                        key: $crate::__macro_support::PortKey::new(stringify!($sender_name)),
                        kind: $crate::__macro_support::NodePortKind::Sender,
                        msg_spec: $crate::__macro_support::MessageSpec::new::<$sender_ty>($sender_desc),
                    }
                ),+
            ]),
        )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __leaf_plugin_build_params {
    (none) => {
        None
    };
    ($params:expr) => {
        Some($params)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __control_plugin_impl {
    (
        $plugin_name:ident,
        $name:expr,
        $children_binding:ident,
        $params:tt,
        $params_binding:ident,
        $create:expr $(,)?
    ) => {
        struct $plugin_name {
            spec: NodeSpec,
            factory: ControlFactory,
        }

        impl Plugin for $plugin_name {
            type Spec = NodeSpec;
            type Factory = ControlFactory;

            fn new() -> Self
            where
                Self: Sized,
            {
                Self {
                    spec: NodeSpec::builder()
                        .key(NodeSpecKey::new(NodeName::new($name), NodeKind::Control))
                        .maybe_params($crate::__leaf_plugin_build_params!($params))
                        .build(),

                    factory: ControlFactory::new(Box::new(|data: ControlReconstructionData| {
                        let $params_binding = data.parameters;
                        let $children_binding = data.context.children;

                        Ok(Box::new($create) as BoxNode)
                    })),
                }
            }

            fn spec(&self) -> &Self::Spec {
                &self.spec
            }

            fn factory(&self) -> &Self::Factory {
                &self.factory
            }

            fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
                (self.spec, self.factory)
            }
        }

        $crate::submit!(ControlPluginConstructor::new::<$plugin_name>());
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __decorator_plugin_impl {
    (
        $plugin_name:ident,
        $name:expr,
        $child_binding:ident,
        $params:tt,
        $params_binding:ident,
        $create:expr $(,)?
    ) => {
        struct $plugin_name {
            spec: NodeSpec,
            factory: DecoratorFactory,
        }

        impl Plugin for $plugin_name {
            type Spec = NodeSpec;
            type Factory = DecoratorFactory;

            fn new() -> Self
            where
                Self: Sized,
            {
                Self {
                    spec: NodeSpec::builder()
                        .key(NodeSpecKey::new(NodeName::new($name), NodeKind::Decorator))
                        .maybe_params($crate::__leaf_plugin_build_params!($params))
                        .build(),

                    factory: DecoratorFactory::new(Box::new(
                        |data: DecoratorReconstructionData| {
                            let $params_binding = data.parameters;
                            let $child_binding = data.context.child;

                            Ok(Box::new($create) as BoxNode)
                        },
                    )),
                }
            }

            fn spec(&self) -> &Self::Spec {
                &self.spec
            }

            fn factory(&self) -> &Self::Factory {
                &self.factory
            }

            fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
                (self.spec, self.factory)
            }
        }

        $crate::submit!(DecoratorPluginConstructor::new::<$plugin_name>());
    };
}
