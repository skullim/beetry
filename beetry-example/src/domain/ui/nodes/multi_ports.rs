use anyhow::Result;
use beetry::{
    channel::{Receiver, Sender, receivers},
    leaf::{ActionBehavior, NodeTask, Task},
    plugin::action,
    runtime::TickStatus,
};
use bon::bon;

use crate::domain::Pose;

receivers! {
    MultiPortSubscriberReceivers {
        in1: Pose,
        in2: Pose,
        in3: Pose,
        in4: Pose,
        in5: Pose,
    }
}

pub struct MultiPortSubscriber<R1, R2, R3, R4, R5>
where
    R1: Receiver<Pose>,
    R2: Receiver<Pose>,
    R3: Receiver<Pose>,
    R4: Receiver<Pose>,
    R5: Receiver<Pose>,
{
    receivers: MultiPortSubscriberReceivers<R1, R2, R3, R4, R5>,
}

impl<R1, R2, R3, R4, R5> MultiPortSubscriber<R1, R2, R3, R4, R5>
where
    R1: Receiver<Pose>,
    R2: Receiver<Pose>,
    R3: Receiver<Pose>,
    R4: Receiver<Pose>,
    R5: Receiver<Pose>,
{
    pub fn new(receivers: MultiPortSubscriberReceivers<R1, R2, R3, R4, R5>) -> Self {
        Self { receivers }
    }

    fn restore_initial_state(&mut self) {
        self.receivers.drain();
    }
}

impl<R1, R2, R3, R4, R5> ActionBehavior for MultiPortSubscriber<R1, R2, R3, R4, R5>
where
    R1: Receiver<Pose>,
    R2: Receiver<Pose>,
    R3: Receiver<Pose>,
    R4: Receiver<Pose>,
    R5: Receiver<Pose>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let _: [Pose; 5] = [
            self.receivers.in1()?,
            self.receivers.in2()?,
            self.receivers.in3()?,
            self.receivers.in4()?,
            self.receivers.in5()?,
        ];

        Ok(NodeTask::new(MultiPortSubscriberTask))
    }

    fn reset(&mut self) {
        self.restore_initial_state();
    }

    fn on_aborted(&mut self) {
        self.restore_initial_state();
    }
}

pub struct MultiPortPublisher<S1, S2, S3, S4, S5>
where
    S1: Sender<Pose>,
    S2: Sender<Pose>,
    S3: Sender<Pose>,
    S4: Sender<Pose>,
    S5: Sender<Pose>,
{
    out1: S1,
    out2: S2,
    out3: S3,
    out4: S4,
    out5: S5,
}

impl<S1, S2, S3, S4, S5> MultiPortPublisher<S1, S2, S3, S4, S5>
where
    S1: Sender<Pose>,
    S2: Sender<Pose>,
    S3: Sender<Pose>,
    S4: Sender<Pose>,
    S5: Sender<Pose>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(out1: S1, out2: S2, out3: S3, out4: S4, out5: S5) -> Self {
        Self {
            out1,
            out2,
            out3,
            out4,
            out5,
        }
    }
}

impl<S1, S2, S3, S4, S5> ActionBehavior for MultiPortPublisher<S1, S2, S3, S4, S5>
where
    S1: Sender<Pose>,
    S2: Sender<Pose>,
    S3: Sender<Pose>,
    S4: Sender<Pose>,
    S5: Sender<Pose>,
{
    fn task(&mut self) -> Result<NodeTask> {
        self.out1
            .try_send(Pose::new(1.0, 1.0))
            .map_err(|error| anyhow::anyhow!("failed to send out1: {error}"))?;
        self.out2
            .try_send(Pose::new(2.0, 2.0))
            .map_err(|error| anyhow::anyhow!("failed to send out2: {error}"))?;
        self.out3
            .try_send(Pose::new(3.0, 3.0))
            .map_err(|error| anyhow::anyhow!("failed to send out3: {error}"))?;
        self.out4
            .try_send(Pose::new(4.0, 4.0))
            .map_err(|error| anyhow::anyhow!("failed to send out4: {error}"))?;
        self.out5
            .try_send(Pose::new(5.0, 5.0))
            .map_err(|error| anyhow::anyhow!("failed to send out5: {error}"))?;

        Ok(NodeTask::new(MultiPortPublisherTask))
    }
}

struct MultiPortSubscriberTask;

impl Task for MultiPortSubscriberTask {
    async fn run(self) -> TickStatus {
        TickStatus::Success
    }
}

struct MultiPortPublisherTask;

impl Task for MultiPortPublisherTask {
    async fn run(self) -> TickStatus {
        TickStatus::Success
    }
}

action! {
    MultiPortSubscriberPlugin: "Multi Port Subscriber";
    receivers: [
        in1: Pose => "Pose A",
        in2: Pose => "Pose B",
        in3: Pose => "Pose C",
        in4: Pose => "Pose D",
        in5: Pose => "Pose E",
    ];
    create: MultiPortSubscriber::new(
        MultiPortSubscriberReceivers::builder()
            .in1(in1)
            .in2(in2)
            .in3(in3)
            .in4(in4)
            .in5(in5)
            .build()
    );
}

action! {
    MultiPortPublisherPlugin: "Multi Port Publisher";
    senders: [
        out1: Pose => "Pose A",
        out2: Pose => "Pose B",
        out3: Pose => "Pose C",
        out4: Pose => "Pose D",
        out5: Pose => "Pose E",
    ];
    create: MultiPortPublisher::new(out1, out2, out3, out4, out5);
}
