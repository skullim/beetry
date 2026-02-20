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
                    factory: $crate::channel::Factory::from_msg_type::<$msg_ty>(),
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
