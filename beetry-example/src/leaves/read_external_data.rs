use crate::ExternalData;
use anyhow::Result;
use anyhow::anyhow;
use beetry_channel::downcast;
use beetry_core::BoxActionBehavior;
use beetry_core::{self, ActionBehavior, NodeTask, Task, TickStatus};
use beetry_macros::receivers;
use bon::bon;
use type_hash::TypeHash;

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

// plugin! {
//   ReadExternalDataPlugin: Action {
//     spec = spec! {type = action, name = "ReadExternalData", receivers = [ExternalData, desc = "External data"] },
//     factory_fn = |mut data: ActionReconstructionData| {
//       let receivers = downcast! {receivers = &mut data.inner.receivers, expected = [ExternalData]}
//       .map_err(|_| anyhow!("failed to obtain typed receivers"))?;
//       Ok(Box::new(ReadExternalData::new(
//          ReadExternalDataReceivers::builder().data(receivers.0).build())) as BoxActionBehavior)
//       }
//     }
// }
