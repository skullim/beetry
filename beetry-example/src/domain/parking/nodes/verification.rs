use beetry::{channel::Receiver, leaf::ConditionBehavior, plugin::condition};
use tracing::info;

use super::{
    super::messages::{ManeuverStatus, ProximityState, TargetSlot, VehicleState},
    ParkingMilestone,
};
use crate::domain::Pose;

pub struct VerifyFinalPose<PR, TR> {
    pose_recv: PR,
    target_recv: TR,
    last_pose: Pose,
    last_target: TargetSlot,
}

impl<PR, TR> VerifyFinalPose<PR, TR>
where
    PR: Receiver<Pose>,
    TR: Receiver<TargetSlot>,
{
    pub fn new(pose_recv: PR, target_recv: TR) -> Self {
        Self {
            pose_recv,
            target_recv,
            last_pose: Pose::default(),
            last_target: TargetSlot::default(),
        }
    }
}

impl<PR, TR> ConditionBehavior for VerifyFinalPose<PR, TR>
where
    PR: Receiver<Pose>,
    TR: Receiver<TargetSlot>,
{
    fn cond(&mut self) -> bool {
        if let Ok(v) = self.pose_recv.try_recv() {
            self.last_pose = v;
        }
        if let Ok(v) = self.target_recv.try_recv() {
            self.last_target = v;
        }

        let threshold = self.last_target.id as f32 + 2.0;
        let aligned = self.last_pose.y.abs() <= 1.0;
        let reached = self.last_pose.x >= threshold && aligned;
        info!(
            "VerifyFinalPose evaluated: reached={}, aligned={}, pose={:?}, target={:?}, threshold={}",
            reached, aligned, self.last_pose, self.last_target, threshold
        );
        if reached {
            ParkingMilestone::VerifyPoseTrue.emit();
        }
        reached
    }
}

condition! {
    VerifyFinalPosePlugin: "VerifyFinalPose";
    receivers: [
        pose_recv: Pose => "Current pose",
        target_recv: TargetSlot => "Target slot",
    ];
    create: VerifyFinalPose::new(pose_recv, target_recv);
}

pub struct VerifyClearance<R> {
    recv: R,
    last: ProximityState,
}

impl<R> VerifyClearance<R>
where
    R: Receiver<ProximityState>,
{
    pub fn new(recv: R) -> Self {
        Self {
            recv,
            last: ProximityState::default(),
        }
    }
}

impl<R> ConditionBehavior for VerifyClearance<R>
where
    R: Receiver<ProximityState>,
{
    fn cond(&mut self) -> bool {
        if let Ok(v) = self.recv.try_recv() {
            self.last = v;
        }
        let clear = !self.last.blocked;
        info!(
            "VerifyClearance evaluated: clear={}, proximity={:?}",
            clear, self.last
        );
        if clear {
            ParkingMilestone::VerifyClearTrue.emit();
        }
        clear
    }
}

condition! {
    VerifyClearancePlugin: "VerifyClearance";
    receivers: [recv: ProximityState => "Proximity alert"];
    create: VerifyClearance::new(recv);
}

pub struct ConfirmParkedState<VR, MR> {
    vehicle_recv: VR,
    maneuver_recv: MR,
    last_vehicle: VehicleState,
    last_maneuver: ManeuverStatus,
}

impl<VR, MR> ConfirmParkedState<VR, MR>
where
    VR: Receiver<VehicleState>,
    MR: Receiver<ManeuverStatus>,
{
    pub fn new(vehicle_recv: VR, maneuver_recv: MR) -> Self {
        Self {
            vehicle_recv,
            maneuver_recv,
            last_vehicle: VehicleState::default(),
            last_maneuver: ManeuverStatus::default(),
        }
    }
}

impl<VR, MR> ConditionBehavior for ConfirmParkedState<VR, MR>
where
    VR: Receiver<VehicleState>,
    MR: Receiver<ManeuverStatus>,
{
    fn cond(&mut self) -> bool {
        if let Ok(v) = self.vehicle_recv.try_recv() {
            self.last_vehicle = v;
        }
        if let Ok(v) = self.maneuver_recv.try_recv() {
            self.last_maneuver = v;
        }
        let parked = self.last_vehicle.parked || self.last_maneuver.done;
        info!(
            "ConfirmParkedState evaluated: parked={}, maneuver={:?}",
            parked, self.last_maneuver
        );
        if parked {
            ParkingMilestone::ConfirmParkedTrue.emit();
        }
        parked
    }
}

condition! {
    ConfirmParkedStatePlugin: "ConfirmParkedState";
    receivers: [
        vehicle_recv: VehicleState => "Vehicle state",
        maneuver_recv: ManeuverStatus => "Maneuver status",
    ];
    create: ConfirmParkedState::new(vehicle_recv, maneuver_recv);
}
