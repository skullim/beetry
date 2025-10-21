use std::collections::HashMap;

use beetry_definitions::description::MessageHash;

use crate::any::AnyBoxedReceiver;

#[derive(Default)]
pub struct ReceiverRegistry {
    registry: HashMap<MessageHash, AnyBoxedReceiver>,
}

impl ReceiverRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, hash: MessageHash, recv: AnyBoxedReceiver) {
        self.registry.insert(hash, recv);
    }

    pub fn take(&mut self, hash: MessageHash) -> Option<AnyBoxedReceiver> {
        self.registry.remove(&hash)
    }
}
