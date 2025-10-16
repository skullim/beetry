use crate::{ExternalData, leaves::ReadExternalDataInput};
use anyhow::{Result, anyhow};
use beetry_backend::{ActionBehavior, NodeTask, Task, TreeStatus, channel::Receiver};
use beetry_definitions::description::{
    LeafDescription, LeafKind, MessageDescription, NodeHashProvider,
};
use beetry_plugin::node::{self, ActionFactory, NodePlugin, NodeReconstructionData};
use type_hash::TypeHash;

struct ReadExternalData<R>
where
    R: Receiver<ExternalData>,
{
    input: ReadExternalDataInput<R>,
}

impl<R> ReadExternalData<R>
where
    R: Receiver<ExternalData>,
{
    pub fn new(input: ReadExternalDataInput<R>) -> Self {
        Self { input }
    }
}

impl<R> ActionBehavior for ReadExternalData<R>
where
    R: Receiver<ExternalData>,
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
    async fn run(self) -> TreeStatus {
        println!("received external data: {:?}", self.data);
        TreeStatus::Success
    }
}

pub struct ReadExternalDataPlugin {
    factory: node::ActionFactory,
}

impl NodePlugin for ReadExternalDataPlugin {
    type Description = LeafDescription;
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
                )) as Box<dyn ActionBehavior>)
            } else {
                anyhow::bail!("failed to instantiate node from erased type");
            }
        };
        Self {
            factory: node::ActionFactory::new(Box::new(closure)),
        }
    }

    fn desc(&self) -> LeafDescription {
        LeafDescription::builder()
            .name("ReadExternalData")
            .hash(ReadExternalDataTask::hash())
            .kind(LeafKind::Action)
            .receivers([MessageDescription::new::<ExternalData>("External data")])
            .build()
    }

    fn factory(self: Box<Self>) -> node::ActionFactory {
        self.factory
    }
}
