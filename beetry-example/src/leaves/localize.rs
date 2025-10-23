use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use beetry_core::{self, ActionBehavior, NodeTask, Task, TickStatus};
use beetry_plugin::node::ActionFactory;
use beetry_plugin::node::{self, NodePlugin, NodeReconstructionData};
use beetry_serde::ser::channel::MessageSpec;
use beetry_serde::ser::node::{LeafKind, LeafSpec, NodeHashProvider};
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

pub struct LocalizePlugin {
    factory: node::ActionFactory,
}

impl NodePlugin for LocalizePlugin {
    type Spec = LeafSpec;
    type Factory = ActionFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        let closure = |mut data: NodeReconstructionData| {
            let any_sender = data
                .senders
                .pop()
                .ok_or_else(|| anyhow!("expected non empty senders vector"))?;
            if let Ok(sender) = any_sender.into_sender_of::<Pose>() {
                Ok(Box::new(Localize::new(sender)) as Box<dyn ActionBehavior>)
            } else {
                anyhow::bail!("failed to instantiate node from erased type");
            }
        };
        Self {
            factory: node::ActionFactory::new(Box::new(closure)),
        }
    }

    fn spec(&self) -> LeafSpec {
        LeafSpec::builder()
            .name("Localize")
            .hash(<Localize<()>>::hash())
            .kind(LeafKind::Action)
            .senders([MessageSpec::new::<Pose>("Localized pose")])
            .build()
    }

    fn factory(self: Box<Self>) -> node::ActionFactory {
        self.factory
    }
}
