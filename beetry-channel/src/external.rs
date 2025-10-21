use std::collections::HashMap;

use beetry_definitions::description::MessageHash;

use crate::any::AnyBoxReceiver;

#[derive(Default)]
pub struct ReceiverRegistry {
    registry: HashMap<MessageHash, AnyBoxReceiver>,
}

impl ReceiverRegistry {
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
