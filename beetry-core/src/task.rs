mod execution;

#[cfg(test)]
pub(crate) use execution::MockRegisterTask;

pub use execution::{AbortTask, ExecutorConcept, QueryTask, RegisterTask, TaskControl};

use crate::TickStatus;
use anyhow::{Error, Result, anyhow};
use std::{pin::Pin, str::FromStr};

pub trait Task {
    fn run(self) -> impl Future<Output = TickStatus> + Send + Sync + 'static;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Success,
    Running,
    Failure,
    Aborted,
}

impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failure | Self::Aborted)
    }
}

impl From<TickStatus> for TaskStatus {
    fn from(value: TickStatus) -> Self {
        match value {
            TickStatus::Success => Self::Success,
            TickStatus::Running => Self::Running,
            TickStatus::Failure => Self::Failure,
        }
    }
}

impl TryFrom<TaskStatus> for TickStatus {
    type Error = Error;
    fn try_from(value: TaskStatus) -> Result<Self, Self::Error> {
        match value {
            TaskStatus::Success => Ok(TickStatus::Success),
            TaskStatus::Running => Ok(TickStatus::Running),
            TaskStatus::Failure => Ok(TickStatus::Failure),
            _ => Err(anyhow!("expected tree status subset of task status")),
        }
    }
}

pub type BoxTaskFuture = Box<dyn Future<Output = TickStatus> + Send + Sync + 'static>;

pub struct NodeTask {
    task: BoxTaskFuture,
    pub desc: TaskDescription,
}

impl NodeTask {
    pub fn new(task: impl Task) -> Self {
        let desc = task.task_desc();
        let task = Box::new(task.run());

        Self { task, desc }
    }

    pub async fn execute(self) -> TickStatus {
        let fut = Pin::from(self.task);
        fut.await
    }
}
