use std::time::Duration;

use crate::Pose;
use anyhow::{Result, anyhow};
use beetry_channel::downcast;
use beetry_core::{ActionBehavior, BoxActionBehavior, NodeTask, Receiver, Task, TickStatus};
use beetry_editor_types::{NodeSpecKey, PortsSpec};
use beetry_macros::receivers;
use beetry_plugin::Plugin;
use beetry_plugin::node::ActionFactory;
use beetry_plugin::node::ActionReconstructionData;
use beetry_plugin_types::channel::MessageSpec;
use beetry_plugin_types::node::NodeName;
use tracing::{debug, instrument};
use type_hash::TypeHash;

use bon::bon;

receivers! {
    DriveReceivers {
    pose: Pose,
}}

pub struct Drive<R>
where
    R: Receiver<Pose>,
{
    receivers: DriveReceivers<R>,
}

impl<R> Drive<R>
where
    R: Receiver<Pose>,
{
    pub fn new(receivers: DriveReceivers<R>) -> Self {
        Self { receivers }
    }

    fn restore_initial_state(&mut self) {
        self.receivers.drain();
        debug!("restored initial state");
    }
}

impl<R> ActionBehavior for Drive<R>
where
    R: Receiver<Pose>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let pose = self.receivers.pose()?;
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

// plugin! {
//   DrivePlugin: Action {
//     spec = spec! {type = action, name = "Drive", receivers = [Pose, desc = "Drive pose"] },
//     factory_fn = |mut data: ActionReconstructionData| {
//       let receivers = downcast! {receivers = &mut data.inner.receivers, expected = [Pose]}
//       .map_err(|_| anyhow!("failed to obtain typed receivers"))?;
//       Ok(Box::new(Drive::new(
//          DriveReceivers::builder().pose(receivers.0).build())) as BoxActionBehavior)
//       }
//     }
// }

// plugin2! {
//     DrivePlugin: Action {
//         name: "Drive",
//         receivers: {
//             Pose => "Drive pose",
//         }
//     }
// }

pub struct DrivePlugin {
    spec: beetry_editor_types::NodeSpec,
    factory: ActionFactory,
}

impl Plugin for DrivePlugin {
    type Spec = beetry_editor_types::NodeSpec;
    type Factory = ActionFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        let factory_fn = |mut data: ActionReconstructionData| {
            let receivers = downcast! {receivers = &mut data.inner.receivers, expected = [Pose]}
                .map_err(|_| anyhow!("failed to obtain typed receivers"))?;
            Ok(Box::new(Drive::new(
                DriveReceivers::builder().pose(receivers.0).build(),
            )) as BoxActionBehavior)
        };
        let spec = beetry_editor_types::NodeSpec::builder()
            .key(NodeSpecKey::new(
                NodeName::new("Drive"),
                beetry_editor_types::NodeKind::Action,
            ))
            .ports(PortsSpec::new(
                std::iter::empty(),
                std::iter::once(MessageSpec::new::<Pose>("Drive pose")),
            ))
            .build();

        Self {
            spec,
            factory: Self::Factory::new(Box::new(factory_fn)),
        }
    }

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }

    fn factory(&self) -> &Self::Factory {
        &self.factory
    }

    fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory) {
        (self.spec, self.factory)
    }
}

#[cfg(test)]
mod tests {
    use beetry_channel::tokio;
    use beetry_core::BoxReceiver;
    use beetry_plugin::Plugin;
    use beetry_plugin::node::{ActionReconstructionData, LeafMetadata};

    use crate::Pose;
    use crate::leaves::drive::DrivePlugin;

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
