use anyhow::{Result, anyhow, bail};
use beetry::{
    channel::{Receiver, Sender},
    leaf::{ActionBehavior, NodeTask, Task},
    plugin::action,
    runtime::TickStatus,
};
use tokio::sync::mpsc::{
    Receiver as TokioReceiver, Sender as TokioSender, channel as mpsc_channel,
};
use tracing::info;

use super::{
    super::messages::{SlotCandidates, TargetSlot, VehicleState},
    ParkingMilestone,
};
use crate::domain::Pose;

pub struct DetectParkingSlots<R, S> {
    pose_recv: R,
    send: S,
    last_pose: Pose,
    task_candidates_recv: Option<TokioReceiver<SlotCandidates>>,
}

impl<R, S> DetectParkingSlots<R, S>
where
    R: Receiver<Pose>,
    S: Sender<SlotCandidates>,
{
    pub fn new(pose_recv: R, send: S) -> Self {
        Self {
            pose_recv,
            send,
            last_pose: Pose::default(),
            task_candidates_recv: None,
        }
    }
}

impl<R, S> ActionBehavior for DetectParkingSlots<R, S>
where
    R: Receiver<Pose>,
    S: Sender<SlotCandidates>,
{
    fn task(&mut self) -> Result<NodeTask> {
        while let Ok(v) = self.pose_recv.try_recv() {
            self.last_pose = v;
        }
        let (send, recv) = mpsc_channel(8);
        self.task_candidates_recv = Some(recv);
        Ok(NodeTask::new(DetectParkingSlotsTask::new(
            self.last_pose,
            send,
        )))
    }

    fn on_running(&mut self) -> Result<()> {
        let Some(task_candidates_receiver) = &mut self.task_candidates_recv else {
            bail!("slot candidates receiver should have been set");
        };
        while let Ok(candidates) = task_candidates_receiver.try_recv() {
            self.send
                .try_send(candidates)
                .map_err(|error| anyhow!("failed to send slot candidates: {error}"))?;
        }
        Ok(())
    }

    fn on_success(&mut self) -> Result<()> {
        self.on_running()?;
        Ok(())
    }

    fn reset(&mut self) {
        self.task_candidates_recv = None;
    }

    fn on_aborted(&mut self) -> Result<()> {
        self.reset();
        Ok(())
    }

    fn on_failure(&mut self) -> Result<()> {
        self.reset();
        Ok(())
    }
}

struct DetectParkingSlotsTask {
    pose: Pose,
    send: TokioSender<SlotCandidates>,
}

impl DetectParkingSlotsTask {
    fn new(pose: Pose, send: TokioSender<SlotCandidates>) -> Self {
        Self { pose, send }
    }
}

impl Task for DetectParkingSlotsTask {
    async fn run(self) -> TickStatus {
        info!("DetectParkingSlots task started with pose: {:?}", self.pose);
        ParkingMilestone::DetectStart.emit();
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let candidates = SlotCandidates { count: 3 };
        info!("DetectParkingSlots produced: {:?}", candidates);
        if let Err(e) = self.send.send(candidates).await {
            info!("DetectParkingSlots task failed due to: {e}");
            return TickStatus::Failure;
        }
        info!("DetectParkingSlots task succeeded");
        ParkingMilestone::DetectSuccess.emit();
        TickStatus::Success
    }
}

action! {
    DetectParkingSlotsPlugin: "DetectParkingSlots";
    receivers: [pose_recv: Pose => "Current pose"];
    senders: [send: SlotCandidates => "Slot candidates"];
    create: DetectParkingSlots::new(pose_recv, send);
}

pub struct SelectBestSlot<CR, VR, S> {
    candidates_recv: CR,
    vehicle_recv: VR,
    send: S,
    last_candidates: SlotCandidates,
    last_vehicle: VehicleState,
    task_target_recv: Option<TokioReceiver<TargetSlot>>,
}

impl<CR, VR, S> SelectBestSlot<CR, VR, S>
where
    CR: Receiver<SlotCandidates>,
    VR: Receiver<VehicleState>,
    S: Sender<TargetSlot>,
{
    pub fn new(candidates_recv: CR, vehicle_recv: VR, send: S) -> Self {
        Self {
            candidates_recv,
            vehicle_recv,
            send,
            last_candidates: SlotCandidates::default(),
            last_vehicle: VehicleState::default(),
            task_target_recv: None,
        }
    }
}

impl<CR, VR, S> ActionBehavior for SelectBestSlot<CR, VR, S>
where
    CR: Receiver<SlotCandidates>,
    VR: Receiver<VehicleState>,
    S: Sender<TargetSlot>,
{
    fn task(&mut self) -> Result<NodeTask> {
        while let Ok(v) = self.candidates_recv.try_recv() {
            self.last_candidates = v;
        }
        while let Ok(v) = self.vehicle_recv.try_recv() {
            self.last_vehicle = v;
        }
        let (send, recv) = mpsc_channel(8);
        self.task_target_recv = Some(recv);
        Ok(NodeTask::new(SelectBestSlotTask::new(
            self.last_candidates,
            self.last_vehicle,
            send,
        )))
    }

    fn on_running(&mut self) -> Result<()> {
        let Some(task_target_receiver) = &mut self.task_target_recv else {
            bail!("target slot receiver should have been set");
        };
        while let Ok(target) = task_target_receiver.try_recv() {
            self.send
                .try_send(target)
                .map_err(|error| anyhow!("failed to send target slot: {error}"))?;
        }
        Ok(())
    }

    fn on_success(&mut self) -> Result<()> {
        self.on_running()?;
        Ok(())
    }

    fn reset(&mut self) {
        self.task_target_recv = None;
    }

    fn on_aborted(&mut self) -> Result<()> {
        self.reset();
        Ok(())
    }

    fn on_failure(&mut self) -> Result<()> {
        self.reset();
        Ok(())
    }
}

struct SelectBestSlotTask {
    candidates: SlotCandidates,
    vehicle: VehicleState,
    send: TokioSender<TargetSlot>,
}

impl SelectBestSlotTask {
    fn new(
        candidates: SlotCandidates,
        vehicle: VehicleState,
        send: TokioSender<TargetSlot>,
    ) -> Self {
        Self {
            candidates,
            vehicle,
            send,
        }
    }
}

impl Task for SelectBestSlotTask {
    async fn run(self) -> TickStatus {
        info!(
            "SelectBestSlot task started with candidates={:?}, vehicle={:?}",
            self.candidates, self.vehicle
        );
        ParkingMilestone::SelectStart.emit();
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        let id = u32::from(self.candidates.count > 0 && self.vehicle.ready);
        let target = TargetSlot { id };
        info!("SelectBestSlot selected: {:?}", target);
        if let Err(e) = self.send.send(target).await {
            info!("SelectBestSlot task failed due to: {e}");
            return TickStatus::Failure;
        }
        info!("SelectBestSlot task succeeded");
        ParkingMilestone::SelectSuccess.emit();
        TickStatus::Success
    }
}

action! {
    SelectBestSlotPlugin: "SelectBestSlot";
    receivers: [
        candidates_recv: SlotCandidates => "Slot candidates",
        vehicle_recv: VehicleState => "Vehicle state",
    ];
    senders: [send: TargetSlot => "Selected target slot"];
    create: SelectBestSlot::new(candidates_recv, vehicle_recv, send);
}
