use beetry_core::{Receiver, Sender};

pub mod mpsc {
    mod error {
        use beetry_core::error;
        use tokio::sync::mpsc::error::{TryRecvError, TrySendError};

        pub struct TokioTrySendError<T>(pub TrySendError<T>);
        pub struct TokioTryRecvError(pub TryRecvError);

        impl<T> From<TokioTrySendError<T>> for error::TrySendError<T> {
            fn from(value: TokioTrySendError<T>) -> Self {
                match value.0 {
                    TrySendError::Full(v) => Self::Full(v),
                    TrySendError::Closed(v) => Self::Disconnected(v),
                }
            }
        }

        impl From<TokioTryRecvError> for error::TryRecvError {
            fn from(value: TokioTryRecvError) -> Self {
                match value.0 {
                    TryRecvError::Empty => Self::Empty,
                    TryRecvError::Disconnected => Self::Disconnected,
                }
            }
        }
    }

    use beetry_core::{TryRecvResult, TrySendResult};
    use tokio::sync::mpsc::channel as tokio_channel;

    use crate::tokio::mpsc::error::{TokioTryRecvError, TokioTrySendError};

    pub struct Receiver<T>(tokio::sync::mpsc::Receiver<T>);
    impl<T> super::Receiver<T> for Receiver<T> {
        fn try_recv(&mut self) -> TryRecvResult<T> {
            Ok(self.0.try_recv().map_err(TokioTryRecvError)?)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Sender<T>(tokio::sync::mpsc::Sender<T>);
    impl<T> super::Sender<T> for Sender<T> {
        fn try_send(&mut self, message: T) -> TrySendResult<T> {
            Ok(self.0.try_send(message).map_err(TokioTrySendError)?)
        }
    }

    pub fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
        let (send, recv) = tokio_channel(buffer);
        (Sender(send), Receiver(recv))
    }
}

pub mod broadcast {
    mod error {
        use beetry_core::error;
        use tokio::sync::broadcast::error::{SendError, TryRecvError};

        pub struct TokioSendError<T>(pub SendError<T>);
        pub struct TokioTryRecvError(pub TryRecvError);

        impl<T> From<TokioSendError<T>> for error::TrySendError<T> {
            fn from(value: TokioSendError<T>) -> Self {
                // error is returned only if there are no active receivers
                // check https://docs.rs/tokio/latest/tokio/sync/broadcast/error/struct.SendError.html for details
                Self::Disconnected(value.0.0)
            }
        }

        impl From<TokioTryRecvError> for error::TryRecvError {
            fn from(value: TokioTryRecvError) -> Self {
                match value.0 {
                    TryRecvError::Empty => Self::Empty,
                    TryRecvError::Closed => Self::Disconnected,
                    TryRecvError::Lagged(n) => Self::Lagged(n),
                }
            }
        }
    }

    use beetry_core::{TryRecvResult, TrySendResult};
    use tokio::sync::broadcast::channel as tokio_channel;

    use crate::tokio::broadcast::error::{TokioSendError, TokioTryRecvError};

    #[derive(Debug)]
    pub struct Receiver<T>(tokio::sync::broadcast::Receiver<T>);
    impl<T> super::Receiver<T> for Receiver<T>
    where
        T: Clone,
    {
        fn try_recv(&mut self) -> TryRecvResult<T> {
            Ok(self.0.try_recv().map_err(TokioTryRecvError)?)
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
            self.0.send(message).map_err(TokioSendError)?;
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
}
