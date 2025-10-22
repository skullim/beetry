use std::result::Result as StdResult;

pub type TryRecvResult<T> = StdResult<T, error::TryRecvError>;
pub type TrySendResult<T> = StdResult<(), error::TrySendError<T>>;

pub trait Receiver<T> {
    fn try_recv(&mut self) -> TryRecvResult<T>;
    fn drain(&mut self) {
        while self.try_recv().is_ok() {}
    }
}

pub type BoxReceiver<T> = Box<dyn Receiver<T>>;

impl<T: 'static> Receiver<T> for BoxReceiver<T> {
    fn try_recv(&mut self) -> TryRecvResult<T> {
        (**self).try_recv()
    }
}

pub trait Sender<T> {
    fn try_send(&mut self, message: T) -> TrySendResult<T>;
}

pub type BoxSender<T> = Box<dyn Sender<T>>;

impl<T: 'static> Sender<T> for Box<dyn Sender<T>> {
    fn try_send(&mut self, message: T) -> TrySendResult<T> {
        (**self).try_send(message)
    }
}

pub mod error {
    use thiserror::Error as ThisError;

    #[derive(Debug, ThisError)]
    #[error("failure when trying to send via channel")]
    pub enum TrySendError<T> {
        #[error("channel is full")]
        Full(T),
        #[error("channel got disconnected")]
        Disconnected(T),
    }

    #[derive(Debug, ThisError)]
    #[error("failure when trying to receive via channel")]
    pub enum TryRecvError {
        #[error("channel is empty")]
        Empty,
        #[error("channel got disconnected")]
        Disconnected,
        #[error("receiver lagged behind {0} messages")]
        Lagged(u64),
    }
}
