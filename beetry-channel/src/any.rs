use beetry_core::{BoxedReceiver, BoxedSender, Receiver};
use std::any::Any;

use anyhow::{Result, anyhow};

pub struct AnyBoxedReceiver(Box<dyn Any>);

impl AnyBoxedReceiver {
    pub fn new<T, R>(receiver: R) -> Self
    where
        R: Receiver<T> + 'static,
        T: 'static,
    {
        AnyBoxedReceiver(Box::new(Box::new(receiver) as BoxedReceiver<T>))
    }

    pub fn is_receiver_of<T: 'static>(&self) -> bool {
        self.0.is::<BoxedReceiver<T>>()
    }

    pub fn into_receiver_of<T: 'static>(self) -> Result<BoxedReceiver<T>> {
        let casted = self
            .0
            .downcast::<BoxedReceiver<T>>()
            .map_err(|err| anyhow!("failed to downcast to concrete receiver, err: {err:?}"))?;
        Ok(*casted)
    }
}

impl<T: 'static> From<BoxedReceiver<T>> for AnyBoxedReceiver {
    fn from(value: BoxedReceiver<T>) -> Self {
        AnyBoxedReceiver(Box::new(value))
    }
}

pub struct AnyBoxedSender(Box<dyn Any>);

impl AnyBoxedSender {
    pub fn is_sender_of<T: 'static>(&self) -> bool {
        self.0.is::<BoxedSender<T>>()
    }

    pub fn into_sender_of<T: 'static>(self) -> Result<BoxedSender<T>> {
        let casted = self
            .0
            .downcast::<BoxedSender<T>>()
            .map_err(|err| anyhow!("failed to downcast to concrete sender, err: {err:?}"))?;
        Ok(*casted)
    }
}
impl<T: 'static> From<BoxedSender<T>> for AnyBoxedSender {
    fn from(value: BoxedSender<T>) -> Self {
        AnyBoxedSender(Box::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beetry_core::{Receiver, Sender, TryRecvResult, TrySendResult};
    use std::marker::PhantomData;

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
        let stub = Box::new(ReceiverStub(Some(42u32))) as BoxedReceiver<u32>;
        let any: AnyBoxedReceiver = stub.into();
        assert!(any.is_receiver_of::<u32>());
        assert!(any.into_receiver_of::<u32>().is_ok());
    }

    #[test]
    fn receiver_wrong_type_test() {
        let stub = Box::new(ReceiverStub(Some(42u32))) as BoxedReceiver<u32>;
        let any: AnyBoxedReceiver = stub.into();
        assert!(!any.is_receiver_of::<i32>());
        assert!(any.into_receiver_of::<i32>().is_err());
    }

    #[test]
    fn downcast_sender_test() {
        let stub = Box::new(SenderStub::new()) as BoxedSender<u32>;
        let any: AnyBoxedSender = stub.into();
        assert!(any.is_sender_of::<u32>());
        assert!(any.into_sender_of::<u32>().is_ok());
    }

    #[test]
    fn sender_wrong_type_test() {
        let stub = Box::new(SenderStub::new()) as BoxedSender<u32>;
        let any: AnyBoxedSender = stub.into();
        assert!(!any.is_sender_of::<i32>());
        assert!(any.into_sender_of::<i32>().is_err());
    }

    #[test]
    fn new_constructor_test() {
        let stub = ReceiverStub(Some(42u32));
        let any = AnyBoxedReceiver::new(stub);
        assert!(any.is_receiver_of::<u32>());
        assert!(any.into_receiver_of::<u32>().is_ok());
    }

    #[test]
    fn new_constructor_wrong_type_test() {
        let stub = ReceiverStub(Some(42u32));
        let any = AnyBoxedReceiver::new(stub);
        assert!(!any.is_receiver_of::<i32>());
        assert!(any.into_receiver_of::<i32>().is_err());
    }
}
