use std::time::Duration;

use anyhow::{Result, anyhow};
use beetry_channel::downcast;
use beetry_core::{self, ActionBehavior, BoxActionBehavior, NodeTask, Task, TickStatus};
use beetry_editor_types::{NodeSpecKey, PortsSpec};
use beetry_plugin::node::{ActionFactory, ActionReconstructionData};
use beetry_plugin::{Plugin, plugin};
use beetry_plugin_types::channel::MessageSpec;
use beetry_plugin_types::node::NodeName;
use beetry_plugin_types::spec;
use tokio::sync::mpsc::{Receiver, Sender, channel as mpsc_channel};
use tracing::{debug, instrument};
use type_hash::TypeHash;

use crate::Pose;

#[derive(TypeHash)]
pub struct Localize<S> {
    #[type_hash(skip)]
    task_pose_recv: Option<Receiver<Pose>>,
    pose_send: S,
    prev_pose: Pose,
}

impl<S> Localize<S>
where
    S: beetry_core::Sender<Pose>,
{
    pub fn new(pose_send: S) -> Self {
        Self {
            task_pose_recv: None,
            pose_send,
            prev_pose: Pose::new(0.0, 0.0),
        }
    }

    fn restore_initial_state(&mut self) {
        self.task_pose_recv = None;
        self.prev_pose = Pose::new(0.0, 0.0);
        debug!("restored initial state");
    }
}

impl<S> ActionBehavior for Localize<S>
where
    S: beetry_core::Sender<Pose>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let (sender, recv) = mpsc_channel(1);
        let task = LocalizeTask::new(self.prev_pose, sender);
        self.task_pose_recv = Some(recv);
        Ok(NodeTask::new(task))
    }

    #[instrument(skip(self))]
    fn reset(&mut self) {
        self.restore_initial_state();
    }

    #[instrument(skip(self))]
    fn on_aborted(&mut self) {
        debug!("aborting");
        self.task_pose_recv = None;
    }

    #[instrument(skip(self))]
    fn on_success(&mut self) {
        while let Ok(pose) = self.task_pose_recv.as_mut().unwrap().try_recv() {
            debug!("propagating localized pose: {pose:?}");
            self.prev_pose = pose;
            self.pose_send.try_send(pose).unwrap()
        }
    }
}

pub struct LocalizeTask {
    sender: Sender<Pose>,
    prev_pose: Pose,
}

impl LocalizeTask {
    fn new(prev_pose: Pose, sender: Sender<Pose>) -> Self {
        Self { sender, prev_pose }
    }
}

impl Task for LocalizeTask {
    #[instrument(skip(self))]
    async fn run(mut self) -> TickStatus {
        tokio::time::sleep(Duration::from_millis(500)).await;
        self.prev_pose.x += 1.0;
        let localized_pose = self.prev_pose;
        debug!("sending localized pose: {localized_pose:?}");
        if self.sender.send(localized_pose).await.is_err() {
            return TickStatus::Failure;
        }
        TickStatus::Success
    }
}

// plugin! {
//     LocalizePlugin: Action {
//       spec = spec! {type = action, name = "Localize", senders = [Pose, desc = "Localized pose"] },
//       factory_fn = |mut data: ActionReconstructionData| {
//         let senders = downcast! {senders = &mut data.inner.senders, expected = [Pose]}
//         .map_err(|_| anyhow!("failed to obtain typed senders"))?;
//         Ok(Box::new(Localize::new(senders.0)) as BoxActionBehavior)

//         }
//     }
// }

pub struct LocalizePlugin {
    spec: beetry_editor_types::NodeSpec,
    factory: ActionFactory,
}

impl Plugin for LocalizePlugin {
    type Spec = beetry_editor_types::NodeSpec;
    type Factory = ActionFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        let factory_fn = |mut data: ActionReconstructionData| {
            let senders = downcast! {senders = &mut data.inner.senders, expected = [Pose]}
                .map_err(|_| anyhow!("failed to obtain typed senders"))?;
            Ok(Box::new(Localize::new(senders.0)) as BoxActionBehavior)
        };
        let spec = beetry_editor_types::NodeSpec::builder()
            .key(NodeSpecKey::new(
                NodeName::new("Localize"),
                beetry_editor_types::NodeKind::Action,
            ))
            .ports(PortsSpec::new(
                std::iter::once(MessageSpec::new::<Pose>("Localized pose")),
                std::iter::empty(),
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
