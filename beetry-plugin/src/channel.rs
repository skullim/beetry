use anyhow::{Result, anyhow};
use beetry_serde::{
    de::channel::{ChannelImplKind, ChannelMetadata, TokioChannelConfig},
    ser::channel::ChannelSpec,
};
use bon::Builder;

use beetry_channel::{AnyBoxReceiver, AnyBoxSender};
use beetry_core::{BoxReceiver, BoxSender};

use crate::Plugin;

pub trait ChannelPlugin: Plugin<Spec = ChannelSpec, Factory = Factory> {}
impl<P> ChannelPlugin for P where P: Plugin<Spec = ChannelSpec, Factory = Factory> {}
pub type BoxChannelPlugin = Box<dyn ChannelPlugin>;

#[derive(Builder)]
pub struct TypeErasedChannel {
    pub senders: Vec<AnyBoxSender>,
    pub receivers: Vec<AnyBoxReceiver>,
}

impl TypeErasedChannel {
    pub fn try_take_sender(&mut self) -> Result<AnyBoxSender> {
        self.senders
            .pop()
            .ok_or_else(|| anyhow!("no free sender in the channel"))
    }

    pub fn try_take_receiver(&mut self) -> Result<AnyBoxReceiver> {
        self.receivers
            .pop()
            .ok_or_else(|| anyhow!("no free receiver in the channel"))
    }
}

pub struct Factory {
    func: Box<dyn Fn(ChannelMetadata) -> TypeErasedChannel>,
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
                            std::iter::once(Box::new(receiver) as BoxReceiver<T>)
                                .chain(
                                    (1..config.n_receivers().into())
                                        .map(|_| Box::new(sender.subscribe()) as BoxReceiver<T>),
                                )
                                .collect();
                        let senders: Vec<_> = (0..config.n_senders().into())
                            .map(|_| Box::new(sender.clone()) as BoxSender<T>)
                            .collect();

                        (senders, receivers)
                    }
                    ChannelImplKind::Tokio(TokioChannelConfig::Mpsc(config)) => {
                        let (sender, receiver) =
                            beetry_channel::tokio::mpsc::channel::<T>(capacity);

                        let senders: Vec<_> = (0..config.n_senders().into())
                            .map(|_| Box::new(sender.clone()) as BoxSender<T>)
                            .collect();
                        let receivers = vec![Box::new(receiver) as BoxReceiver<T>];

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

pub struct ChannelPluginConstructor(fn() -> BoxChannelPlugin);

impl ChannelPluginConstructor {
    pub const fn new<T: ChannelPlugin + 'static>() -> Self {
        ChannelPluginConstructor(|| Box::new(T::new()))
    }

    fn create(&self) -> BoxChannelPlugin {
        (self.0)()
    }
}

inventory::collect!(ChannelPluginConstructor);

pub fn plugins() -> Vec<BoxChannelPlugin> {
    let mut plugins = vec![];
    for plugin_constructor in inventory::iter::<ChannelPluginConstructor> {
        let plugin = plugin_constructor.create();
        plugins.push(plugin);
    }
    plugins
}
