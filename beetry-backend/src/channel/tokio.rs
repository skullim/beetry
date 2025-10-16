use crate::channel::{Receiver, Sender};

pub mod mpsc {
    use crate::channel::{TryRecvResult, TrySendResult};
    use tokio::sync::mpsc::channel as tokio_channel;

    pub struct Receiver<T>(tokio::sync::mpsc::Receiver<T>);
    impl<T> super::Receiver<T> for Receiver<T> {
        fn try_recv(&mut self) -> TryRecvResult<T> {
            Ok(self.0.try_recv()?)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Sender<T>(tokio::sync::mpsc::Sender<T>);
    impl<T> super::Sender<T> for Sender<T> {
        fn try_send(&mut self, message: T) -> TrySendResult<T> {
            Ok(self.0.try_send(message)?)
        }
    }

    pub fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
        let (send, recv) = tokio_channel(buffer);
        (Sender(send), Receiver(recv))
    }

    mod error {
        use crate::channel::error;
        use tokio::sync::mpsc::error::{TryRecvError, TrySendError};

        impl<T> From<TrySendError<T>> for error::TrySendError<T> {
            fn from(value: TrySendError<T>) -> Self {
                match value {
                    TrySendError::Full(v) => error::TrySendError::Full(v),
                    TrySendError::Closed(v) => error::TrySendError::Disconnected(v),
                }
            }
        }

        impl From<TryRecvError> for error::TryRecvError {
            fn from(value: TryRecvError) -> Self {
                match value {
                    TryRecvError::Empty => error::TryRecvError::Empty,
                    TryRecvError::Disconnected => error::TryRecvError::Disconnected,
                }
            }
        }
    }
}

pub mod broadcast {
    use crate::channel::{TryRecvResult, TrySendResult};
    use tokio::sync::broadcast::channel as tokio_channel;

    #[derive(Debug)]
    pub struct Receiver<T>(tokio::sync::broadcast::Receiver<T>);
    impl<T> super::Receiver<T> for Receiver<T>
    where
        T: Clone,
    {
        fn try_recv(&mut self) -> TryRecvResult<T> {
            Ok(self.0.try_recv()?)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Sender<T>(tokio::sync::broadcast::Sender<T>);
    impl<T> Sender<T> {
        pub fn subscribe(&self) -> Receiver<T> {
            Receiver(self.0.subscribe())
        }
    }

    impl<T> super::Sender<T> for Sender<T> {
        fn try_send(&mut self, message: T) -> TrySendResult<T> {
            // broadcast send is not blocking hence safe to call from try_send abstraction
            self.0.send(message)?;
            Ok(())
        }
    }

    pub fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>)
    where
        T: Clone,
    {
        let (send, recv) = tokio_channel(buffer);
        (Sender(send), Receiver(recv))
    }

    mod error {
        use crate::channel::error;
        use tokio::sync::broadcast::error::{SendError, TryRecvError};

        impl<T> From<SendError<T>> for error::TrySendError<T> {
            fn from(value: SendError<T>) -> Self {
                // error is returned only if there are no active receivers
                // check https://docs.rs/tokio/latest/tokio/sync/broadcast/error/struct.SendError.html for details
                error::TrySendError::Disconnected(value.0)
            }
        }

        impl From<TryRecvError> for error::TryRecvError {
            fn from(value: TryRecvError) -> Self {
                match value {
                    TryRecvError::Empty => error::TryRecvError::Empty,
                    TryRecvError::Closed => error::TryRecvError::Disconnected,
                    TryRecvError::Lagged(n) => error::TryRecvError::Lagged(n),
                }
            }
        }
    }
}

//@todo add support for watch channel
