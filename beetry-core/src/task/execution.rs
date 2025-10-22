use crate::{TaskStatus, task::NodeTask};
use anyhow::Result;

#[cfg(test)]
use mockall::automock;

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
