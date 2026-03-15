mod monitoring;
mod perception;
mod planning;
mod publishers;
mod verification;

use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParkingMilestone {
    CheckReady(bool),
    DetectStart,
    DetectSuccess,
    SelectStart,
    SelectSuccess,
    PlanStart,
    PlanSuccess,
    FollowStart,
    FollowProgress(u32),
    FollowSuccess,
    VerifyPoseTrue,
    VerifyClearTrue,
    ConfirmParkedTrue,
}

impl ParkingMilestone {
    pub fn from_log_message(message: &str) -> Option<Self> {
        message.parse().ok()
    }

    pub fn emit(self) {
        tracing::info!("{self}");
    }
}

impl fmt::Display for ParkingMilestone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CheckReady(false) => write!(f, "check_ready_false"),
            Self::CheckReady(true) => write!(f, "check_ready_true"),
            Self::DetectStart => write!(f, "detect_start"),
            Self::DetectSuccess => write!(f, "detect_success"),
            Self::SelectStart => write!(f, "select_start"),
            Self::SelectSuccess => write!(f, "select_success"),
            Self::PlanStart => write!(f, "plan_start"),
            Self::PlanSuccess => write!(f, "plan_success"),
            Self::FollowStart => write!(f, "follow_start"),
            Self::FollowProgress(progress) => write!(f, "follow_progress_{progress}"),
            Self::FollowSuccess => write!(f, "follow_success"),
            Self::VerifyPoseTrue => write!(f, "verify_pose_true"),
            Self::VerifyClearTrue => write!(f, "verify_clear_true"),
            Self::ConfirmParkedTrue => write!(f, "confirm_parked_true"),
        }
    }
}

impl FromStr for ParkingMilestone {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "check_ready_false" => Ok(Self::CheckReady(false)),
            "check_ready_true" => Ok(Self::CheckReady(true)),
            "detect_start" => Ok(Self::DetectStart),
            "detect_success" => Ok(Self::DetectSuccess),
            "select_start" => Ok(Self::SelectStart),
            "select_success" => Ok(Self::SelectSuccess),
            "plan_start" => Ok(Self::PlanStart),
            "plan_success" => Ok(Self::PlanSuccess),
            "follow_start" => Ok(Self::FollowStart),
            "follow_success" => Ok(Self::FollowSuccess),
            "verify_pose_true" => Ok(Self::VerifyPoseTrue),
            "verify_clear_true" => Ok(Self::VerifyClearTrue),
            "confirm_parked_true" => Ok(Self::ConfirmParkedTrue),
            _ => {
                if let Some(progress_str) = s.strip_prefix("follow_progress_") {
                    let progress = progress_str.parse::<u32>().map_err(|_| ())?;
                    Ok(Self::FollowProgress(progress))
                } else {
                    Err(())
                }
            }
        }
    }
}

pub use monitoring::{
    CheckSystemReady, CheckSystemReadyPlugin, SafetyMonitor, SafetyMonitorPlugin,
};
pub use perception::{
    DetectParkingSlots, DetectParkingSlotsPlugin, SelectBestSlot, SelectBestSlotPlugin,
};
pub use planning::{
    FollowTrajectory, FollowTrajectoryPlugin, PlanParkingTrajectory, PlanParkingTrajectoryPlugin,
};
pub use publishers::{
    BrakePublisher, BrakePublisherPlugin, LocalizationPublisher, LocalizationPublisherPlugin,
    ProximityPublisher, ProximityPublisherPlugin, VehicleStatePublisher,
    VehicleStatePublisherPlugin,
};
pub use verification::{
    ConfirmParkedState, ConfirmParkedStatePlugin, VerifyClearance, VerifyClearancePlugin,
    VerifyFinalPose, VerifyFinalPosePlugin,
};
