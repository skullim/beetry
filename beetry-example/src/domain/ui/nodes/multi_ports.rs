use anyhow::Result;
use beetry::{
    Message,
    channel::{Receiver, Sender, receivers},
    leaf::{ActionBehavior, NodeTask, Task},
    plugin::action,
    runtime::TickStatus,
};
use bon::bon;
use type_hash::TypeHash;

use crate::domain::Pose;

macro_rules! define_multi_port_pose {
    ($msg_ty:ident, $channel_ty:ident) => {
        #[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
        pub struct $msg_ty(pub Pose);

        impl From<Pose> for $msg_ty {
            fn from(value: Pose) -> Self {
                Self(value)
            }
        }

        impl From<$msg_ty> for Pose {
            fn from(value: $msg_ty) -> Self {
                value.0
            }
        }

        beetry::plugin::channel! {$channel_ty: $msg_ty}
    };
}

define_multi_port_pose!(PoseA, PoseAChannel);
define_multi_port_pose!(PoseB, PoseBChannel);
define_multi_port_pose!(PoseC, PoseCChannel);
define_multi_port_pose!(PoseD, PoseDChannel);
define_multi_port_pose!(PoseE, PoseEChannel);

receivers! {
    MultiPortSubscriberReceivers {
        in1: PoseA,
        in2: PoseB,
        in3: PoseC,
        in4: PoseD,
        in5: PoseE,
    }
}

pub struct MultiPortSubscriber<R1, R2, R3, R4, R5>
where
    R1: Receiver<PoseA>,
    R2: Receiver<PoseB>,
    R3: Receiver<PoseC>,
    R4: Receiver<PoseD>,
    R5: Receiver<PoseE>,
{
    receivers: MultiPortSubscriberReceivers<R1, R2, R3, R4, R5>,
}

impl<R1, R2, R3, R4, R5> MultiPortSubscriber<R1, R2, R3, R4, R5>
where
    R1: Receiver<PoseA>,
    R2: Receiver<PoseB>,
    R3: Receiver<PoseC>,
    R4: Receiver<PoseD>,
    R5: Receiver<PoseE>,
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
    R1: Receiver<PoseA>,
    R2: Receiver<PoseB>,
    R3: Receiver<PoseC>,
    R4: Receiver<PoseD>,
    R5: Receiver<PoseE>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let _: [Pose; 5] = [
            self.receivers.in1()?.into(),
            self.receivers.in2()?.into(),
            self.receivers.in3()?.into(),
            self.receivers.in4()?.into(),
            self.receivers.in5()?.into(),
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
    S1: Sender<PoseA>,
    S2: Sender<PoseB>,
    S3: Sender<PoseC>,
    S4: Sender<PoseD>,
    S5: Sender<PoseE>,
{
    out1: S1,
    out2: S2,
    out3: S3,
    out4: S4,
    out5: S5,
}

impl<S1, S2, S3, S4, S5> MultiPortPublisher<S1, S2, S3, S4, S5>
where
    S1: Sender<PoseA>,
    S2: Sender<PoseB>,
    S3: Sender<PoseC>,
    S4: Sender<PoseD>,
    S5: Sender<PoseE>,
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
    S1: Sender<PoseA>,
    S2: Sender<PoseB>,
    S3: Sender<PoseC>,
    S4: Sender<PoseD>,
    S5: Sender<PoseE>,
{
    fn task(&mut self) -> Result<NodeTask> {
        self.out1
            .try_send(Pose::new(1.0, 1.0).into())
            .map_err(|error| anyhow::anyhow!("failed to send out1: {error}"))?;
        self.out2
            .try_send(Pose::new(2.0, 2.0).into())
            .map_err(|error| anyhow::anyhow!("failed to send out2: {error}"))?;
        self.out3
            .try_send(Pose::new(3.0, 3.0).into())
            .map_err(|error| anyhow::anyhow!("failed to send out3: {error}"))?;
        self.out4
            .try_send(Pose::new(4.0, 4.0).into())
            .map_err(|error| anyhow::anyhow!("failed to send out4: {error}"))?;
        self.out5
            .try_send(Pose::new(5.0, 5.0).into())
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
        in1: PoseA => "Pose A",
        in2: PoseB => "Pose B",
        in3: PoseC => "Pose C",
        in4: PoseD => "Pose D",
        in5: PoseE => "Pose E",
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
        out1: PoseA => "Pose A",
        out2: PoseB => "Pose B",
        out3: PoseC => "Pose C",
        out4: PoseD => "Pose D",
        out5: PoseE => "Pose E",
    ];
    create: MultiPortPublisher::new(out1, out2, out3, out4, out5);
}
