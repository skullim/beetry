use std::time::Duration;

use crate::{Pose, leaves::DriveInput};
use anyhow::{Result, anyhow};
use beetry_core::{ActionBehavior, BoxActionBehavior, NodeTask, Receiver, Task, TickStatus};
use beetry_plugin::{
    Plugin,
    node::{self, ActionFactory, ActionReconstructionData},
};
use beetry_serde::ser::{
    channel::MessageSpec,
    node::{ActionLeafSchema, ActionSpec},
};
use tracing::{debug, instrument};
use type_hash::TypeHash;

pub struct Drive<R>
where
    R: Receiver<Pose>,
{
    input: DriveInput<R>,
}

impl<R> Drive<R>
where
    R: Receiver<Pose>,
{
    pub fn new(input: DriveInput<R>) -> Self {
        Self { input }
    }

    fn restore_initial_state(&mut self) {
        self.input.drain();
        debug!("restored initial state");
    }
}

impl<R> ActionBehavior for Drive<R>
where
    R: Receiver<Pose>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let pose = self.input.pose()?;
        Ok(NodeTask::new(DriveTask::new(pose)))
    }

    #[instrument(skip(self))]
    fn reset(&mut self) {
        self.restore_initial_state();
    }

    #[instrument(skip(self))]
    fn on_aborted(&mut self) {
        self.restore_initial_state();
    }
}

#[derive(TypeHash)]
struct DriveTask {
    pose: Pose,
}

impl DriveTask {
    fn new(pose: Pose) -> Self {
        Self { pose }
    }
}

impl Task for DriveTask {
    #[instrument(skip(self))]
    async fn run(self) -> TickStatus {
        let pose = self.pose;
        debug!("received pose {pose:?}");
        let target_pose = Pose::new(pose.x + 1.2, pose.y);
        debug!("driving to: {target_pose:?}");
        tokio::time::sleep(Duration::from_millis(100)).await;
        debug!("finishing drive task with status success");
        TickStatus::Success
    }
}

pub struct DrivePlugin;

impl Plugin for DrivePlugin {
    type Spec = ActionSpec;
    type Factory = ActionFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn spec(&self) -> Self::Spec {
        ActionSpec::builder()
            .name("Drive".to_string())
            .schema(
                ActionLeafSchema::builder()
                    .receivers([MessageSpec::new::<Pose>("Drive pose")])
                    .build(),
            )
            .build()
    }

    fn factory(self: Box<Self>) -> node::ActionFactory {
        let factory_fn = |mut data: ActionReconstructionData| {
            let recv = data
                .inner
                .receivers
                .pop()
                .ok_or_else(|| anyhow!("expected non empty receivers vector"))?;
            if let Ok(recv) = recv.into_receiver_of::<Pose>() {
                Ok(
                    Box::new(Drive::new(DriveInput::builder().pose(recv).build()))
                        as BoxActionBehavior,
                )
            } else {
                anyhow::bail!("failed to instantiate node from erased type");
            }
        };
        node::ActionFactory::new(Box::new(factory_fn))
    }
}

#[cfg(test)]
mod tests {
    use beetry_channel::tokio;
    use beetry_core::BoxReceiver;
    use beetry_plugin::{
        Plugin,
        node::{ActionReconstructionData, LeafMetadata},
    };

    use crate::{Pose, leaves::drive::DrivePlugin};

    #[test]
    fn test_reconstruction() {
        let (_, receiver) = tokio::mpsc::channel::<Pose>(1);
        let receiver: BoxReceiver<Pose> = Box::new(receiver);
        let data = ActionReconstructionData::builder()
            .inner(LeafMetadata::builder().receivers([receiver.into()]).build())
            .build();

        let plugin = Box::new(DrivePlugin::new());
        let factory = plugin.factory();

        assert!(factory.try_create(data).is_ok());
    }
}
