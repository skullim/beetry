/// Creates a channel plugin for a message type.
///
/// The `channel!` macro creates a plugin that:
///
/// - exposes a `ChannelSpec` for the editor and persistence layer
/// - provides a channel factory for reconstruction
/// - registers itself through the plugin inventory
/// - makes the message type available in the editor's Channels section
///
/// Example:
///
/// ```no_run
/// use beetry_macros::Message;
/// use beetry_message::Message;
/// use type_hash::TypeHash;
///
/// #[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
/// pub struct VehicleState {
///     pub ready: bool,
///     pub parked: bool,
///     pub speed_mps: f32,
/// }
///
/// beetry_plugin::channel! {VehicleStateChannel: VehicleState}
/// ```
#[macro_export]
macro_rules! channel {
    ($plugin_id:ident : $msg_ty:ty) => {
        pub struct $plugin_id {
            spec: $crate::__macro_support::ChannelSpec,
            factory: $crate::channel::Factory,
        }

        impl $crate::Plugin for $plugin_id {
            type Spec = $crate::__macro_support::ChannelSpec;
            type Factory = $crate::channel::Factory;

            fn new() -> Self
            where
                Self: Sized,
            {
                Self {
                    spec: $crate::__macro_support::ChannelSpec::new::<$msg_ty>(),
                    factory: $crate::channel::Factory::from_msg::<$msg_ty>(),
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

        $crate::submit!($crate::channel::ChannelPluginConstructor::new::<$plugin_id>());
    };
}
