use crate::TaskStatus;
use crate::task::NodeTask;
use anyhow::Result;

#[cfg(test)]
use mockall::automock;

pub trait ExecutorConcept {
    fn run(&mut self) -> impl Future<Output = Result<()>>;
}

#[cfg_attr(test, automock)]
pub trait RegisterTask<TH>
where
    TH: TaskHandle,
{
    fn register(&self, task: NodeTask) -> Result<TH>;
}

pub trait TaskHandle: QueryTask + AbortTask {}
impl<T: QueryTask + AbortTask> TaskHandle for T {}

#[cfg_attr(test, automock)]
pub trait QueryTask {
    fn query(&mut self) -> TaskStatus;
}

#[cfg_attr(test, automock)]
pub trait AbortTask {
    fn abort(&mut self);
}
