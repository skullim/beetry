use beetry::Message;
use beetry_message::Message;
use type_hash::TypeHash;

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct VehicleState {
    pub ready: bool,
    pub parked: bool,
    pub speed_mps: f32,
}

beetry_plugin::channel! {VehicleStateChannel: VehicleState}

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct SlotCandidates {
    pub count: u32,
}

beetry_plugin::channel! {SlotCandidatesChannel: SlotCandidates}

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct TargetSlot {
    pub id: u32,
}

beetry_plugin::channel! {TargetSlotChannel: TargetSlot}

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct Trajectory {
    pub waypoints: u32,
}

beetry_plugin::channel! {TrajectoryChannel: Trajectory}

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct ManeuverStatus {
    pub progress: u32,
    pub done: bool,
}

beetry_plugin::channel! {ManeuverStatusChannel: ManeuverStatus}

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct ProximityState {
    pub blocked: bool,
}

beetry_plugin::channel! {ProximityAlertChannel: ProximityState}

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct BrakeState {
    pub engaged: bool,
}

beetry_plugin::channel! {EmergencyBrakeChannel: BrakeState}

#[derive(Debug, Clone, Copy, Default, TypeHash, Message)]
pub struct SafetyStatus {
    pub safe: bool,
}

beetry_plugin::channel! {SafetyStatusChannel: SafetyStatus}
