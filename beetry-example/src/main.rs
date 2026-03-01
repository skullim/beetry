use beetry_editor_types::persistence::ValidTree;
use beetry_serialization::{Deserializer, JsonDeserializer};
use rfd::FileHandle;
use std::io::Read;
use std::time::Duration;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};
use tracing_tree::HierarchicalLayer;

use anyhow::{Result, anyhow};
use beetry_builder::Builder;
use beetry_core::{
    BoxNode, ExecutorConcept, PeriodicTick, PeriodicTicker, RegisterTask, Root, TaskHandle, Tree,
    TreeEngine,
};
use beetry_example::{
    BrakePublisher, BrakeState, CheckSystemReady, ConfirmParkedState, DetectParkingSlots,
    FollowTrajectory, LocalizationPublisher, ManeuverStatus, PlanParkingTrajectory, Pose,
    ProximityPublisher, ProximityState, SafetyMonitor, SafetyStatus, SelectBestSlot,
    SlotCandidates, TargetSlot, Trajectory, VehicleState, VehicleStatePublisher, VerifyClearance,
    VerifyFinalPose,
};
use beetry_exec::{Executor, ExecutorConfig};
use beetry_node::{MemSequence, Sequence, UntilSuccess};

#[tokio::main]
async fn main() -> Result<()> {
    let targets = Targets::new()
        .with_target("zbus", LevelFilter::ERROR)
        .with_default(LevelFilter::DEBUG);
    let subscriber = Registry::default().with(
        HierarchicalLayer::new(2)
            .with_targets(true)
            .with_filter(targets),
    );
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let executor = Executor::new(ExecutorConfig::default());
    let (mut ready_exec, registry) = executor.into_ready_with_registry();
    let builder = Builder::new(registry);

    let creation_type = BtCreationType::Editor;
    let bt = match creation_type {
        BtCreationType::Editor => bt_from_editor(&builder).await?,
        BtCreationType::Code => bt_from_code(&builder)?,
    };

    let exec_thread = std::thread::spawn(move || -> anyhow::Result<()> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(async move { ready_exec.run().await })
    });

    let mut engine = TreeEngine::new(bt);
    let ticker = PeriodicTicker::new(PeriodicTick::new(Duration::from_millis(10)));
    engine.tick_till_terminal(ticker).await?;
    let _ = exec_thread.join();

    Ok(())
}

enum BtCreationType {
    Code,
    Editor,
}

fn bt_from_code<R, T>(builder: &Builder<R, T>) -> Result<Tree<BoxNode>>
where
    R: RegisterTask<T> + 'static,
    T: TaskHandle + 'static,
{
    use beetry_channel::tokio::watch;
    let (vehicle_state_send, _vehicle_state_recv) = watch::channel::<VehicleState>();
    let (pose_send, _pose_recv) = watch::channel::<Pose>();
    let (proximity_send, _proximity_recv) = watch::channel::<ProximityState>();
    let (brake_send, _brake_recv) = watch::channel::<BrakeState>();
    let (safety_send, _safety_recv) = watch::channel::<SafetyStatus>();
    let (slot_candidates_send, _slot_candidates_recv) = watch::channel::<SlotCandidates>();
    let (target_slot_send, _target_slot_recv) = watch::channel::<TargetSlot>();
    let (trajectory_send, _trajectory_recv) = watch::channel::<Trajectory>();
    let (maneuver_send, _maneuver_recv) = watch::channel::<ManeuverStatus>();

    let plan_follow_verify: BoxNode = Box::new(UntilSuccess::new(Box::new(MemSequence::new([
        builder.action(PlanParkingTrajectory::new(
            pose_send.subscribe(),
            target_slot_send.subscribe(),
            trajectory_send.clone(),
        )),
        builder.action(FollowTrajectory::new(
            trajectory_send.subscribe(),
            pose_send.subscribe(),
            safety_send.subscribe(),
            maneuver_send.clone(),
        )),
        builder.condition(VerifyFinalPose::new(
            pose_send.subscribe(),
            target_slot_send.subscribe(),
        )),
        builder.condition(VerifyClearance::new(proximity_send.subscribe())),
    ])) as BoxNode));

    let mission_branch: BoxNode = Box::new(Sequence::new([
        builder.condition(CheckSystemReady::new(vehicle_state_send.subscribe())),
        Box::new(MemSequence::new([
            builder.action(DetectParkingSlots::new(
                pose_send.subscribe(),
                slot_candidates_send.clone(),
            )),
            builder.action(SelectBestSlot::new(
                slot_candidates_send.subscribe(),
                vehicle_state_send.subscribe(),
                target_slot_send.clone(),
            )),
            plan_follow_verify,
            builder.condition(ConfirmParkedState::new(
                vehicle_state_send.subscribe(),
                maneuver_send.subscribe(),
            )),
        ])),
    ]));

    let root = builder.parallel([
        builder.action(VehicleStatePublisher::new(vehicle_state_send.clone())),
        builder.action(LocalizationPublisher::new(pose_send.clone())),
        builder.action(ProximityPublisher::new(proximity_send.clone())),
        builder.action(BrakePublisher::new(brake_send.clone())),
        builder.action(SafetyMonitor::new(
            proximity_send.subscribe(),
            brake_send.subscribe(),
            safety_send.clone(),
        )),
        mission_branch,
    ]);

    Ok(builder.tree(Root::new(root)))
}

async fn select_import_file() -> Result<FileHandle> {
    use rfd;
    rfd::AsyncFileDialog::new()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .set_title("Select file to import")
        .pick_file()
        .await
        .ok_or_else(|| anyhow!("No file selected"))
}

async fn bt_from_editor<R, T>(builder: &Builder<R, T>) -> Result<Tree<BoxNode>>
where
    R: RegisterTask<T> + 'static,
    T: TaskHandle + 'static,
{
    use beetry_reconstruction::TreeReconstructor;

    let handle = select_import_file().await?;
    let mut file = std::fs::File::open(handle.path())?;
    let mut content_buffer = String::new();
    file.read_to_string(&mut content_buffer)?;
    let valid_tree: ValidTree = JsonDeserializer::deserialize(&content_buffer)?;
    let mut reconstructor = TreeReconstructor::new()?;
    reconstructor.try_reconstruct(valid_tree, builder)
}
