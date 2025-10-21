use crate::{TickStatus, task::NodeTask};
use anyhow::{Error, Result, anyhow};

#[cfg(test)]
use mockall::automock;

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

pub trait ExecutorConcept {
    fn run(&mut self) -> impl Future<Output = Result<()>>;
}

#[cfg_attr(test, automock)]
pub trait RegisterTask<T>
where
    T: TaskControl,
{
    fn register(&self, task: NodeTask) -> Result<T>;
}

pub trait TaskControl: QueryTask + AbortTask {}
impl<T: QueryTask + AbortTask> TaskControl for T {}

#[cfg_attr(test, automock)]
pub trait QueryTask {
    fn query(&mut self) -> TaskStatus;
}

#[cfg_attr(test, automock)]
pub trait AbortTask {
    fn abort(&mut self);
}
