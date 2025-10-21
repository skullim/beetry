mod execution;

#[cfg(test)]
pub(crate) use execution::MockRegisterTask;

pub use execution::{AbortTask, ExecutorConcept, QueryTask, RegisterTask, TaskControl, TaskStatus};

use crate::TreeStatus;
use std::{pin::Pin, str::FromStr};

pub trait Task {
    fn run(self) -> impl Future<Output = TreeStatus> + Send + Sync + 'static;
    fn task_desc(&self) -> TaskDescription {
        TaskDescription::from_str(std::any::type_name::<Self>()).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct TaskDescription {
    desc: String,
}

impl FromStr for TaskDescription {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TaskDescription { desc: s.into() })
    }
}

impl std::fmt::Display for TaskDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.desc)
    }
}

pub type NodeTaskFuture = Box<dyn Future<Output = TreeStatus> + Send + Sync + 'static>;

pub struct NodeTask {
    task: NodeTaskFuture,
    pub desc: TaskDescription,
}

impl NodeTask {
    pub fn new(task: impl Task) -> Self {
        let desc = task.task_desc();
        let task = Box::new(task.run());

        Self { task, desc }
    }

    pub async fn execute(self) -> TreeStatus {
        let fut = Pin::from(self.task);
        fut.await
    }
}
