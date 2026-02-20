use crate::ExternalData;
use anyhow::Result;
use beetry_core::{self, ActionBehavior, NodeTask, Task, TickStatus};
use beetry_macros::receivers;
use beetry_plugin::action;
use bon::bon;

receivers! {ReadExternalDataReceivers {
    data: ExternalData,
}}

struct ReadExternalData<R> {
    receivers: ReadExternalDataReceivers<R>,
}

impl<R> ReadExternalData<R>
where
    R: beetry_core::Receiver<ExternalData>,
{
    pub fn new(receivers: ReadExternalDataReceivers<R>) -> Self {
        Self { receivers }
    }
}

impl<R> ActionBehavior for ReadExternalData<R>
where
    R: beetry_core::Receiver<ExternalData>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let data = self.receivers.data()?;
        Ok(NodeTask::new(ReadExternalDataTask::new(data)))
    }
    // no reset here, queue of messages on the external channel should not be drained
}

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

action! {
    ReadExternalDataPlugin: "ReadExternalData";
    receivers: [data: ExternalData => "External data"];
    create: ReadExternalData::new(ReadExternalDataReceivers::builder().data(data).build());
}
