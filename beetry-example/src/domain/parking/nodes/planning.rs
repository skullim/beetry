use anyhow::Result;
use beetry_core::{ActionBehavior, NodeTask, Receiver, Sender, Task, TickStatus};
use beetry_plugin::action;
use tokio::sync::mpsc::{
    Receiver as TokioReceiver, Sender as TokioSender, channel as mpsc_channel,
};
use tracing::info;

use crate::Pose;

use super::super::messages::{ManeuverStatus, SafetyStatus, TargetSlot, Trajectory};
use super::ParkingMilestone;

pub struct PlanParkingTrajectory<PR, TR, S> {
    pose_recv: PR,
    target_recv: TR,
    send: S,
    last_pose: Pose,
    last_target: TargetSlot,
    task_trajectory_recv: Option<TokioReceiver<Trajectory>>,
}

impl<PR, TR, S> PlanParkingTrajectory<PR, TR, S>
where
    PR: Receiver<Pose>,
    TR: Receiver<TargetSlot>,
    S: Sender<Trajectory>,
{
    pub fn new(pose_recv: PR, target_recv: TR, send: S) -> Self {
        Self {
            pose_recv,
            target_recv,
            send,
            last_pose: Pose::default(),
            last_target: TargetSlot::default(),
            task_trajectory_recv: None,
        }
    }
}

struct PlanParkingTrajectoryTask {
    pose: Pose,
    target: TargetSlot,
    send: TokioSender<Trajectory>,
}

impl PlanParkingTrajectoryTask {
    fn new(pose: Pose, target: TargetSlot, send: TokioSender<Trajectory>) -> Self {
        Self { pose, target, send }
    }
}

impl Task for PlanParkingTrajectoryTask {
    async fn run(self) -> TickStatus {
        info!(
            "PlanParkingTrajectory task started with pose={:?}, target={:?}",
            self.pose, self.target
        );
        ParkingMilestone::PlanStart.emit();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let waypoints = if self.target.id > 0 && self.pose.x >= 0.0 {
            4
        } else {
            0
        };
        let trajectory = Trajectory { waypoints };
        info!("PlanParkingTrajectory produced: {:?}", trajectory);
        if self.send.send(trajectory).await.is_err() {
            info!("PlanParkingTrajectory task failed: receiver disconnected");
            return TickStatus::Failure;
        }
        info!("PlanParkingTrajectory task succeeded");
        ParkingMilestone::PlanSuccess.emit();
        TickStatus::Success
    }
}

impl<PR, TR, S> ActionBehavior for PlanParkingTrajectory<PR, TR, S>
where
    PR: Receiver<Pose>,
    TR: Receiver<TargetSlot>,
    S: Sender<Trajectory>,
{
    fn task(&mut self) -> Result<NodeTask> {
        while let Ok(v) = self.pose_recv.try_recv() {
            self.last_pose = v;
        }
        while let Ok(v) = self.target_recv.try_recv() {
            self.last_target = v;
        }
        let (send, recv) = mpsc_channel(8);
        self.task_trajectory_recv = Some(recv);
        Ok(NodeTask::new(PlanParkingTrajectoryTask::new(
            self.last_pose,
            self.last_target,
            send,
        )))
    }

    fn on_running(&mut self) {
        if let Some(recv) = &mut self.task_trajectory_recv {
            while let Ok(trajectory) = recv.try_recv() {
                let _ = self.send.try_send(trajectory);
            }
        }
    }

    fn on_success(&mut self) {
        self.on_running();
    }

    fn reset(&mut self) {
        self.task_trajectory_recv = None;
    }

    fn on_aborted(&mut self) {
        self.task_trajectory_recv = None;
    }

    fn on_failure(&mut self) {
        self.task_trajectory_recv = None;
    }
}

action! {
    PlanParkingTrajectoryPlugin: "PlanParkingTrajectory";
    receivers: [
        pose_recv: Pose => "Current pose",
        target_recv: TargetSlot => "Target slot",
    ];
    senders: [send: Trajectory => "Planned trajectory"];
    create: PlanParkingTrajectory::new(pose_recv, target_recv, send);
}

pub struct FollowTrajectory<TR, PR, SR, S> {
    trajectory_recv: TR,
    pose_recv: PR,
    safety_recv: SR,
    send: S,
    last_trajectory: Trajectory,
    last_pose: Pose,
    last_safety: SafetyStatus,
    task_maneuver_recv: Option<TokioReceiver<ManeuverStatus>>,
}

impl<TR, PR, SR, S> FollowTrajectory<TR, PR, SR, S>
where
    TR: Receiver<Trajectory>,
    PR: Receiver<Pose>,
    SR: Receiver<SafetyStatus>,
    S: Sender<ManeuverStatus>,
{
    pub fn new(trajectory_recv: TR, pose_recv: PR, safety_recv: SR, send: S) -> Self {
        Self {
            trajectory_recv,
            pose_recv,
            safety_recv,
            send,
            last_trajectory: Trajectory::default(),
            last_pose: Pose::default(),
            last_safety: SafetyStatus::default(),
            task_maneuver_recv: None,
        }
    }
}

struct FollowTrajectoryTask {
    trajectory: Trajectory,
    pose: Pose,
    safety: SafetyStatus,
    send: TokioSender<ManeuverStatus>,
}

impl FollowTrajectoryTask {
    fn new(
        trajectory: Trajectory,
        pose: Pose,
        safety: SafetyStatus,
        send: TokioSender<ManeuverStatus>,
    ) -> Self {
        Self {
            trajectory,
            pose,
            safety,
            send,
        }
    }
}

impl Task for FollowTrajectoryTask {
    async fn run(self) -> TickStatus {
        info!(
            "FollowTrajectory task started with trajectory={:?}, pose={:?}, safety={:?}",
            self.trajectory, self.pose, self.safety
        );
        ParkingMilestone::FollowStart.emit();
        if !self.safety.safe || self.trajectory.waypoints == 0 || self.pose.x < 0.0 {
            info!("FollowTrajectory cannot execute: preconditions not satisfied");
            let _ = self
                .send
                .send(ManeuverStatus {
                    progress: 0,
                    done: false,
                })
                .await;
            return TickStatus::Failure;
        }

        for progress in 1..=self.trajectory.waypoints {
            tokio::time::sleep(std::time::Duration::from_millis(120)).await;
            let status = ManeuverStatus {
                progress,
                done: progress >= self.trajectory.waypoints,
            };
            info!("FollowTrajectory progress: {:?}", status);
            ParkingMilestone::FollowProgress(progress).emit();
            if self.send.send(status).await.is_err() {
                info!("FollowTrajectory task failed: receiver disconnected");
                return TickStatus::Failure;
            }
        }

        info!("FollowTrajectory task succeeded");
        ParkingMilestone::FollowSuccess.emit();
        TickStatus::Success
    }
}

impl<TR, PR, SR, S> ActionBehavior for FollowTrajectory<TR, PR, SR, S>
where
    TR: Receiver<Trajectory>,
    PR: Receiver<Pose>,
    SR: Receiver<SafetyStatus>,
    S: Sender<ManeuverStatus>,
{
    fn task(&mut self) -> Result<NodeTask> {
        while let Ok(v) = self.trajectory_recv.try_recv() {
            self.last_trajectory = v;
        }
        while let Ok(v) = self.pose_recv.try_recv() {
            self.last_pose = v;
        }
        while let Ok(v) = self.safety_recv.try_recv() {
            self.last_safety = v;
        }
        let (send, recv) = mpsc_channel(8);
        self.task_maneuver_recv = Some(recv);
        Ok(NodeTask::new(FollowTrajectoryTask::new(
            self.last_trajectory,
            self.last_pose,
            self.last_safety,
            send,
        )))
    }

    fn on_running(&mut self) {
        if let Some(recv) = &mut self.task_maneuver_recv {
            while let Ok(status) = recv.try_recv() {
                let _ = self.send.try_send(status);
            }
        }
    }

    fn on_success(&mut self) {
        self.on_running();
    }

    fn reset(&mut self) {
        self.task_maneuver_recv = None;
    }

    fn on_aborted(&mut self) {
        self.task_maneuver_recv = None;
    }

    fn on_failure(&mut self) {
        self.task_maneuver_recv = None;
    }
}

action! {
    FollowTrajectoryPlugin: "FollowTrajectory";
    receivers: [
        trajectory_recv: Trajectory => "Trajectory",
        pose_recv: Pose => "Current pose",
        safety_recv: SafetyStatus => "Safety status",
    ];
    senders: [send: ManeuverStatus => "Maneuver status"];
    create: FollowTrajectory::new(trajectory_recv, pose_recv, safety_recv, send);
}
