use crate::{ExternalData, leaves::ReadExternalDataInput};
use anyhow::{Result, anyhow};
use beetry_core::{self, ActionBehavior, BoxActionBehavior, NodeTask, Task, TickStatus};
use beetry_plugin::{
    Plugin,
    node::{self, ActionFactory, NodeReconstructionData},
};
use beetry_serde::ser::{
    channel::MessageSpec,
    node::{LeafKind, LeafSchema, LeafSpec, NodeName},
};
use type_hash::TypeHash;

struct ReadExternalData<R>
where
    R: beetry_core::Receiver<ExternalData>,
{
    input: ReadExternalDataInput<R>,
}

impl<R> ReadExternalData<R>
where
    R: beetry_core::Receiver<ExternalData>,
{
    pub fn new(input: ReadExternalDataInput<R>) -> Self {
        Self { input }
    }
}

impl<R> ActionBehavior for ReadExternalData<R>
where
    R: beetry_core::Receiver<ExternalData>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let data = self.input.data()?;
        Ok(NodeTask::new(ReadExternalDataTask::new(data)))
    }
    // no reset here, queue of messages on the external channel should not be drained
}

#[derive(TypeHash)]
struct ReadExternalDataTask {
    data: ExternalData,
}

impl ReadExternalDataTask {
    fn new(data: ExternalData) -> Self {
        Self { data }
    }
}

impl Task for ReadExternalDataTask {
    async fn run(self) -> TickStatus {
        println!("received external data: {:?}", self.data);
        TickStatus::Success
    }
}

pub struct ReadExternalDataPlugin {
    factory: node::ActionFactory,
}

impl Plugin for ReadExternalDataPlugin {
    type Spec = LeafSpec;
    type Factory = ActionFactory;

    fn new() -> Self
    where
        Self: Sized,
    {
        let closure = |mut data: NodeReconstructionData| {
            let recv = data
                .receivers
                .pop()
                .ok_or_else(|| anyhow!("expected non empty receivers vector"))?;
            if let Ok(recv) = recv.into_receiver_of::<ExternalData>() {
                Ok(Box::new(ReadExternalData::new(
                    ReadExternalDataInput::builder().data(recv).build(),
                )) as BoxActionBehavior)
            } else {
                anyhow::bail!("failed to instantiate node from erased type");
            }
        };
        Self {
            factory: node::ActionFactory::new(Box::new(closure)),
        }
    }

    fn spec(&self) -> LeafSpec {
        LeafSpec::new(
            NodeName::new("ReadExternalData"),
            LeafSchema::builder()
                .kind(LeafKind::Action)
                .receivers([MessageSpec::new::<ExternalData>("External data")])
                .build(),
        )
    }

    fn factory(self: Box<Self>) -> node::ActionFactory {
        self.factory
    }
}
