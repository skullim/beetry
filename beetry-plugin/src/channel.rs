use anyhow::{Result, anyhow};
use beetry_editor_types::{
    output::channel::{ChannelConfig, ChannelKind, TokioChannelKind},
    spec::channel::ChannelSpec,
};
use bon::Builder;

use beetry_channel::{AnyBoxReceiver, AnyBoxSender};
use beetry_core::{BoxReceiver, BoxSender};

use crate::{BoxPlugin, ConstructPlugin, Named, PluginConstructor, PluginError, unique_plugins};

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

impl Named for ChannelSpec {
    fn name(&self) -> &str {
        self.msg_type_name().as_str()
    }
}

pub struct Factory {
    func: Box<dyn Fn(ChannelConfig) -> TypeErasedChannel>,
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
            func: (Box::new(|config| {
                let capacity = config.capacity();
                let count = config.count();
                // client is responsible for providing correct and valid number of senders and receivers
                let n_senders = count.sender();
                let n_receivers = count.receiver();

                let (senders, receivers) = match config.kind() {
                    ChannelKind::Tokio(TokioChannelKind::Broadcast) => {
                        let (sender, receiver) =
                            beetry_channel::tokio::broadcast::channel::<T>(capacity);

                        let receivers: Vec<_> =
                            std::iter::once(Box::new(receiver) as BoxReceiver<T>)
                                .chain(
                                    (1..n_receivers)
                                        .map(|_| Box::new(sender.subscribe()) as BoxReceiver<T>),
                                )
                                .collect();
                        let senders: Vec<_> = (0..n_senders)
                            .map(|_| Box::new(sender.clone()) as BoxSender<T>)
                            .collect();

                        (senders, receivers)
                    }
                    ChannelKind::Tokio(TokioChannelKind::Mpsc) => {
                        let (sender, receiver) =
                            beetry_channel::tokio::mpsc::channel::<T>(capacity);

                        let senders: Vec<_> = (0..n_senders)
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

    pub fn create(&self, config: ChannelConfig) -> TypeErasedChannel {
        (self.func)(config)
    }
}

pub type BoxChannelPlugin = BoxPlugin<ChannelSpec, Factory>;
pub type ChannelPluginConstructor = PluginConstructor<ChannelSpec, Factory>;

impl ChannelPluginConstructor {
    pub fn plugins() -> Result<Vec<BoxChannelPlugin>, PluginError> {
        unique_plugins::<Self, <Self as ConstructPlugin>::Spec, <Self as ConstructPlugin>::Factory>(
        )
    }
}

inventory::collect!(ChannelPluginConstructor);
