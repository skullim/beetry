use std::marker::PhantomData;

use crate::channel::{Receiver, TryRecvResult};

pub struct Input<R, T>
where
    R: Receiver<T>,
{
    receiver: R,
    _phantom: PhantomData<T>,
}

pub trait Metadata {
    type Type;
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
//@todo improve by counting the number of fields and utilizing it for generic parameters declaration

#[macro_export]
macro_rules! input {
    (
        $name:ident {
            $($field:ident : $ty:ty),* $(,)?
        }
    ) => {
        pub struct $name<$($field),*>
        where
            $( $field:  $crate::channel::Receiver<$ty>, )*
        {
            $( $field: $crate::channel::Input<$field, $ty>, )*
        }

        #[bon]
        impl<$($field),*> $name<$($field),*>
        where
            $( $field: $crate::channel::Receiver<$ty>, )*
        {
            #[builder]
            pub fn new($($field: $field),*) -> Self {
                Self {
                    $( $field: $crate::channel::Input::new($field), )*
                }
            }

            pub fn drain(&mut self) {
                $( self.$field.drain(); )*
            }

            $(
                pub fn $field(&mut self) -> $crate::channel::TryRecvResult<$ty> {
                    self.$field.get()
                }
            )*
        }

        impl<$($field),*> $crate::channel::Metadata for $name<$($field),*>
                where
            $( $field: $crate::channel::Receiver<$ty>, )*
        {
            type Type = ($($ty,)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::{Sender, tokio::mpsc};
    use bon::bon;

    input! {
        TestInput {
            sensor_data: i32,
            command: String,
            status: bool,
        }
    }

    #[test]
    fn test_get() {
        let (mut sensor_sender, sensor_receiver) = mpsc::channel::<i32>(1);
        let (mut command_sender, command_receiver) = mpsc::channel::<String>(1);
        let (mut status_sender, status_receiver) = mpsc::channel::<bool>(1);

        sensor_sender.try_send(42).unwrap();
        command_sender.try_send("move_forward".to_string()).unwrap();
        status_sender.try_send(true).unwrap();

        let mut inputs = TestInput::builder()
            .sensor_data(sensor_receiver)
            .command(command_receiver)
            .status(status_receiver)
            .build();

        assert_eq!(inputs.sensor_data().unwrap(), 42);
        assert_eq!(inputs.command().unwrap(), "move_forward");
        assert!(inputs.status().unwrap());

        assert!(inputs.sensor_data().is_err());
        assert!(inputs.command().is_err());
        assert!(inputs.status().is_err());
    }

    #[test]
    fn test_drain() {
        let (mut sensor_sender, sensor_receiver) = mpsc::channel::<i32>(1);
        let (mut command_sender, command_receiver) = mpsc::channel::<String>(1);
        let (mut status_sender, status_receiver) = mpsc::channel::<bool>(1);

        sensor_sender.try_send(100).unwrap();
        command_sender.try_send("stop".to_string()).unwrap();
        status_sender.try_send(false).unwrap();

        let mut inputs = TestInput::builder()
            .sensor_data(sensor_receiver)
            .command(command_receiver)
            .status(status_receiver)
            .build();

        inputs.drain();

        assert!(inputs.sensor_data().is_err());
        assert!(inputs.command().is_err());
        assert!(inputs.status().is_err());
    }
}
