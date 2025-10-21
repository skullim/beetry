use anyhow::{Result, anyhow};
use bon::Builder;

use beetry_channel::{AnyBoxedReceiver, AnyBoxedSender};
use beetry_core::{BoxedReceiver, Sender};
use beetry_definitions::{
    description::ChannelDescription,
    export::{ChannelImplKind, ChannelMetadata, TokioChannelConfig},
};

/// Defines channel for given data type. There should be at most one plugin for each data type.
pub trait ChannelPlugin: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;

    fn desc(&self) -> ChannelDescription;

    fn factory(self: Box<Self>) -> Factory;
}

#[derive(Builder)]
pub struct TypeErasedChannel {
    pub senders: Vec<AnyBoxedSender>,
    pub receivers: Vec<AnyBoxedReceiver>,
}

impl TypeErasedChannel {
    pub fn try_take_sender(&mut self) -> Result<AnyBoxedSender> {
        self.senders
            .pop()
            .ok_or_else(|| anyhow!("no free sender in the channel"))
    }

    pub fn try_take_receiver(&mut self) -> Result<AnyBoxedReceiver> {
        self.receivers
            .pop()
            .ok_or_else(|| anyhow!("no free receiver in the channel"))
    }
}

pub struct Factory {
    func: Box<dyn Fn(ChannelMetadata) -> TypeErasedChannel + Send + Sync>,
}

impl std::fmt::Debug for Factory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Factory")
            .field("func", &"<function>")
            .finish()
    }
}

impl Factory {
    pub fn from_msg_type<T: Clone + 'static>() -> Self {
        Self {
            func: (Box::new(|meta| {
                let capacity = meta.capacity();
                let (senders, receivers) = match meta.impl_kind() {
                    ChannelImplKind::Tokio(TokioChannelConfig::Broadcast(config)) => {
                        let (sender, receiver) =
                            beetry_channel::tokio::broadcast::channel::<T>(capacity);

                        let receivers: Vec<_> =
                            std::iter::once(Box::new(receiver) as BoxedReceiver<T>)
                                .chain(
                                    (1..config.n_receivers().into())
                                        .map(|_| Box::new(sender.subscribe()) as BoxedReceiver<T>),
                                )
                                .collect();
                        let senders: Vec<_> = (0..config.n_senders().into())
                            .map(|_| Box::new(sender.clone()) as Box<dyn Sender<T>>)
                            .collect();

                        (senders, receivers)
                    }
                    ChannelImplKind::Tokio(TokioChannelConfig::Mpsc(config)) => {
                        let (sender, receiver) =
                            beetry_channel::tokio::mpsc::channel::<T>(capacity);

                        let senders: Vec<_> = (0..config.n_senders().into())
                            .map(|_| Box::new(sender.clone()) as Box<dyn Sender<T>>)
                            .collect();
                        let receivers = vec![Box::new(receiver) as BoxedReceiver<T>];

                        (senders, receivers)
                    }
                };

                TypeErasedChannel::builder()
                    .senders(senders.into_iter().map(Into::into).collect())
                    .receivers(receivers.into_iter().map(Into::into).collect())
                    .build()
            })),
        }
    }

    pub fn create(&self, meta: ChannelMetadata) -> TypeErasedChannel {
        (self.func)(meta)
    }
}

pub struct ChannelPluginConstructor(fn() -> Box<dyn ChannelPlugin>);

impl ChannelPluginConstructor {
    pub const fn new<T: ChannelPlugin + 'static>() -> Self {
        ChannelPluginConstructor(|| Box::new(T::new()))
    }

    fn create(&self) -> Box<dyn ChannelPlugin> {
        (self.0)()
    }
}

inventory::collect!(ChannelPluginConstructor);

pub fn plugins() -> Vec<Box<dyn ChannelPlugin>> {
    let mut plugins = vec![];
    for plugin_constructor in inventory::iter::<ChannelPluginConstructor> {
        let plugin = plugin_constructor.create();
        plugins.push(plugin);
    }
    plugins
}
