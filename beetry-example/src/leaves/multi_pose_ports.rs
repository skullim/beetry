use anyhow::Result;
use beetry_core::{ActionBehavior, NodeTask, Receiver, Sender, Task, TickStatus};
use beetry_macros::receivers;
use beetry_plugin::action;
use bon::bon;

use crate::Pose;

receivers! {
    MultiPosePortsReceivers {
        in1: Pose,
        in2: Pose,
        in3: Pose,
        in4: Pose,
        in5: Pose,
    }
}

pub struct MultiPosePorts<R1, R2, R3, R4, R5, S1, S2, S3, S4, S5>
where
    R1: Receiver<Pose>,
    R2: Receiver<Pose>,
    R3: Receiver<Pose>,
    R4: Receiver<Pose>,
    R5: Receiver<Pose>,
    S1: Sender<Pose>,
    S2: Sender<Pose>,
    S3: Sender<Pose>,
    S4: Sender<Pose>,
    S5: Sender<Pose>,
{
    receivers: MultiPosePortsReceivers<R1, R2, R3, R4, R5>,
    out1: S1,
    out2: S2,
    out3: S3,
    out4: S4,
    out5: S5,
}

impl<R1, R2, R3, R4, R5, S1, S2, S3, S4, S5> MultiPosePorts<R1, R2, R3, R4, R5, S1, S2, S3, S4, S5>
where
    R1: Receiver<Pose>,
    R2: Receiver<Pose>,
    R3: Receiver<Pose>,
    R4: Receiver<Pose>,
    R5: Receiver<Pose>,
    S1: Sender<Pose>,
    S2: Sender<Pose>,
    S3: Sender<Pose>,
    S4: Sender<Pose>,
    S5: Sender<Pose>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        receivers: MultiPosePortsReceivers<R1, R2, R3, R4, R5>,
        out1: S1,
        out2: S2,
        out3: S3,
        out4: S4,
        out5: S5,
    ) -> Self {
        Self {
            receivers,
            out1,
            out2,
            out3,
            out4,
            out5,
        }
    }

    fn restore_initial_state(&mut self) {
        self.receivers.drain();
    }
}

impl<R1, R2, R3, R4, R5, S1, S2, S3, S4, S5> ActionBehavior
    for MultiPosePorts<R1, R2, R3, R4, R5, S1, S2, S3, S4, S5>
where
    R1: Receiver<Pose>,
    R2: Receiver<Pose>,
    R3: Receiver<Pose>,
    R4: Receiver<Pose>,
    R5: Receiver<Pose>,
    S1: Sender<Pose>,
    S2: Sender<Pose>,
    S3: Sender<Pose>,
    S4: Sender<Pose>,
    S5: Sender<Pose>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let in1 = self.receivers.in1()?;
        let in2 = self.receivers.in2()?;
        let in3 = self.receivers.in3()?;
        let in4 = self.receivers.in4()?;
        let in5 = self.receivers.in5()?;

        self.out1
            .try_send(in1)
            .map_err(|error| anyhow::anyhow!("failed to send out1: {error}"))?;
        self.out2
            .try_send(in2)
            .map_err(|error| anyhow::anyhow!("failed to send out2: {error}"))?;
        self.out3
            .try_send(in3)
            .map_err(|error| anyhow::anyhow!("failed to send out3: {error}"))?;
        self.out4
            .try_send(in4)
            .map_err(|error| anyhow::anyhow!("failed to send out4: {error}"))?;
        self.out5
            .try_send(in5)
            .map_err(|error| anyhow::anyhow!("failed to send out5: {error}"))?;

        Ok(NodeTask::new(MultiPosePortsTask))
    }

    fn reset(&mut self) {
        self.restore_initial_state();
    }

    fn on_aborted(&mut self) {
        self.restore_initial_state();
    }
}

struct MultiPosePortsTask;

impl Task for MultiPosePortsTask {
    async fn run(self) -> TickStatus {
        TickStatus::Success
    }
}

action! {
    MultiPosePortsPlugin: "MultiPosePorts";
    receivers: [
        in1: Pose => "Pose input 1",
        in2: Pose => "Pose input 2",
        in3: Pose => "Pose input 3",
        in4: Pose => "Pose input 4",
        in5: Pose => "Pose input 5",
    ];
    senders: [
        out1: Pose => "Pose output 1",
        out2: Pose => "Pose output 2",
        out3: Pose => "Pose output 3",
        out4: Pose => "Pose output 4",
        out5: Pose => "Pose output 5",
    ];
    create: MultiPosePorts::new(
        MultiPosePortsReceivers::builder()
            .in1(in1)
            .in2(in2)
            .in3(in3)
            .in4(in4)
            .in5(in5)
            .build(),
        out1,
        out2,
        out3,
        out4,
        out5
    );
}
