use std::marker::PhantomData;

use beetry_core::{Receiver, TryRecvResult};

//@todo move to beetry-macros
pub struct Input<R, T> {
    receiver: R,
    _phantom: PhantomData<T>,
}

impl<R, T> Input<R, T>
where
    R: Receiver<T>,
{
    pub fn new(receiver: R) -> Self {
        Self {
            receiver,
            _phantom: PhantomData,
        }
    }

    pub fn get(&mut self) -> TryRecvResult<T> {
        self.receiver.try_recv()
    }

    pub fn drain(&mut self) {
        self.receiver.drain();
    }
}
