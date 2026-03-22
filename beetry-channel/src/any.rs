use std::any::Any;

use anyhow::{Result, anyhow};

use crate::{BoxReceiver, BoxSender, Receiver};

#[derive(Debug)]
pub struct AnyBoxReceiver(Box<dyn Any>);

impl AnyBoxReceiver {
    pub fn new<T, R>(receiver: R) -> Self
    where
        R: Receiver<T> + 'static,
        T: 'static,
    {
        Self(Box::new(Box::new(receiver) as BoxReceiver<T>))
    }

    #[must_use]
    pub fn is_receiver_of<T: 'static>(&self) -> bool {
        self.0.is::<BoxReceiver<T>>()
    }

    pub fn into_receiver_of<T: 'static>(self) -> Result<BoxReceiver<T>> {
        let casted = self
            .0
            .downcast::<BoxReceiver<T>>()
            .map_err(|err| anyhow!("failed to downcast to concrete receiver, err: {err:?}"))?;
        Ok(*casted)
    }
}

impl<T: 'static> From<BoxReceiver<T>> for AnyBoxReceiver {
    fn from(value: BoxReceiver<T>) -> Self {
        Self(Box::new(value))
    }
}

#[derive(Debug)]
pub struct AnyBoxSender(Box<dyn Any>);

impl AnyBoxSender {
    #[must_use]
    pub fn is_sender_of<T: 'static>(&self) -> bool {
        self.0.is::<BoxSender<T>>()
    }

    pub fn into_sender_of<T: 'static>(self) -> Result<BoxSender<T>> {
        let casted = self
            .0
            .downcast::<BoxSender<T>>()
            .map_err(|err| anyhow!("failed to downcast to concrete sender, err: {err:?}"))?;
        Ok(*casted)
    }
}
impl<T: 'static> From<BoxSender<T>> for AnyBoxSender {
    fn from(value: BoxSender<T>) -> Self {
        Self(Box::new(value))
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;
    use crate::{Receiver, Sender, TryRecvResult, TrySendResult};

    struct ReceiverStub<T>(pub Option<T>);

    impl<T> Receiver<T> for ReceiverStub<T> {
        fn try_recv(&mut self) -> TryRecvResult<T> {
            Ok(self.0.take().unwrap())
        }
    }

    struct SenderStub<T>(PhantomData<T>);

    impl<T> SenderStub<T> {
        fn new() -> Self {
            Self(PhantomData)
        }
    }

    impl<T> Sender<T> for SenderStub<T> {
        fn try_send(&mut self, _value: T) -> TrySendResult<T> {
            Ok(())
        }
    }

    #[test]
    fn downcast_receiver_test() {
        let stub = Box::new(ReceiverStub(Some(42_u32))) as BoxReceiver<u32>;
        let any: AnyBoxReceiver = stub.into();
        assert!(any.is_receiver_of::<u32>());
        any.into_receiver_of::<u32>().unwrap();
    }

    #[test]
    fn receiver_wrong_type_test() {
        let stub = Box::new(ReceiverStub(Some(42_u32))) as BoxReceiver<u32>;
        let any: AnyBoxReceiver = stub.into();
        assert!(!any.is_receiver_of::<i32>());
        assert!(any.into_receiver_of::<i32>().is_err());
    }

    #[test]
    fn downcast_sender_test() {
        let stub = Box::new(SenderStub::new()) as BoxSender<u32>;
        let any: AnyBoxSender = stub.into();
        assert!(any.is_sender_of::<u32>());
        any.into_sender_of::<u32>().unwrap();
    }

    #[test]
    fn sender_wrong_type_test() {
        let stub = Box::new(SenderStub::new()) as BoxSender<u32>;
        let any: AnyBoxSender = stub.into();
        assert!(!any.is_sender_of::<i32>());
        assert!(any.into_sender_of::<i32>().is_err());
    }

    #[test]
    fn new_constructor_test() {
        let stub = ReceiverStub(Some(42_u32));
        let any = AnyBoxReceiver::new(stub);
        assert!(any.is_receiver_of::<u32>());
        any.into_receiver_of::<u32>().unwrap();
    }

    #[test]
    fn new_constructor_wrong_type_test() {
        let stub = ReceiverStub(Some(42_u32));
        let any = AnyBoxReceiver::new(stub);
        assert!(!any.is_receiver_of::<i32>());
        assert!(any.into_receiver_of::<i32>().is_err());
    }
}
