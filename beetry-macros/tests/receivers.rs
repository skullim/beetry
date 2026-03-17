#[cfg(test)]
mod tests {
    mod beetry {
        pub use beetry_channel::Input;
        pub use beetry_core::{Receiver, TryRecvResult};
    }

    use beetry_channel::tokio::mpsc;
    use beetry_core::Sender;
    use beetry_macros::receivers;
    use bon::bon;

    receivers! {
        TestReceivers {
            sensor_data: i32,
            command: String,
            status: bool,
        }
    }

    #[test]
    fn data_transmitted() {
        let (mut sensor_sender, sensor_receiver) = mpsc::channel::<i32>(1);
        let (mut command_sender, command_receiver) = mpsc::channel::<String>(1);
        let (mut status_sender, status_receiver) = mpsc::channel::<bool>(1);

        sensor_sender.try_send(42).unwrap();
        command_sender.try_send("move_forward".to_string()).unwrap();
        status_sender.try_send(true).unwrap();

        let mut inputs = TestReceivers::builder()
            .sensor_data(sensor_receiver)
            .command(command_receiver)
            .status(status_receiver)
            .build();

        assert_eq!(inputs.sensor_data().unwrap(), 42);
        assert_eq!(inputs.command().unwrap(), "move_forward");
        assert!(inputs.status().unwrap());

        inputs.sensor_data().unwrap_err();
        inputs.command().unwrap_err();
        inputs.status().unwrap_err();
    }

    #[test]
    fn drain() {
        let (mut sensor_sender, sensor_receiver) = mpsc::channel::<i32>(1);
        let (mut command_sender, command_receiver) = mpsc::channel::<String>(1);
        let (mut status_sender, status_receiver) = mpsc::channel::<bool>(1);

        sensor_sender.try_send(100).unwrap();
        command_sender.try_send("stop".to_string()).unwrap();
        status_sender.try_send(false).unwrap();

        let mut inputs = TestReceivers::builder()
            .sensor_data(sensor_receiver)
            .command(command_receiver)
            .status(status_receiver)
            .build();

        inputs.drain();

        inputs.sensor_data().unwrap_err();
        inputs.command().unwrap_err();
        inputs.status().unwrap_err();
    }
}
