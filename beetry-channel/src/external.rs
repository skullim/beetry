use std::collections::HashMap;

use beetry_message::MessageHash;

use crate::any::AnyBoxReceiver;

//@todo external senders/receivers should not be part of channel, but rather
// node also, registry should identify which node the given (message hash,
// receiver) pair belongs to
#[derive(Default)]
pub struct ReceiverRegistry {
    registry: HashMap<MessageHash, AnyBoxReceiver>,
}

impl ReceiverRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, hash: MessageHash, recv: AnyBoxReceiver) {
        self.registry.insert(hash, recv);
    }

    pub fn take(&mut self, hash: MessageHash) -> Option<AnyBoxReceiver> {
        self.registry.remove(&hash)
    }
}
