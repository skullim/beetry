use crate::{TaskDescription, status::TreeStatus, task::NodeTask};
use anyhow::{Error, Result, anyhow};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use std::{future::poll_fn, sync::Arc, task::Poll};
use tokio::sync::{
    Notify,
    mpsc::{Receiver, Sender, channel, error::TryRecvError},
};
use tracing::{debug, instrument};

#[cfg(test)]
use mockall::automock;

pub(crate) struct Executor {
    task_recv: Receiver<ExecutionTask>,
}

impl Executor {
    pub(crate) fn new(task_recv: Receiver<ExecutionTask>) -> Self {
        Self { task_recv }
    }

    #[instrument(skip(self), name = "Executor::drive")]
    pub(crate) async fn drive(&mut self) -> Result<()> {
        debug!("start driving registered tasks");
        let mut tasks = FuturesUnordered::new();

        loop {
            let execute_next_task_fut = poll_fn(|cx| {
                if tasks.is_empty() {
                    Poll::Pending
                } else {
                    tasks.poll_next_unpin(cx)
                }
            });

            tokio::select! {
                Some(exe_task) = self.task_recv.recv() => {
                    debug!("received new task to execute: {}", exe_task.task.desc);
                    tasks.push(exe_task.execute());
                },
                _ = execute_next_task_fut => {
                }

            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TaskStatus {
    Success,
    Running,
    Failure,
    Aborted,
}

impl TaskStatus {
    pub(crate) fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failure | Self::Aborted)
    }
}

impl From<TreeStatus> for TaskStatus {
    fn from(value: TreeStatus) -> Self {
        match value {
            TreeStatus::Success => Self::Success,
            TreeStatus::Running => Self::Running,
            TreeStatus::Failure => Self::Failure,
        }
    }
}

impl TryFrom<TaskStatus> for TreeStatus {
    type Error = Error;
    fn try_from(value: TaskStatus) -> Result<Self, Self::Error> {
        match value {
            TaskStatus::Success => Ok(TreeStatus::Success),
            TaskStatus::Running => Ok(TreeStatus::Running),
            TaskStatus::Failure => Ok(TreeStatus::Failure),
            _ => Err(anyhow!("expected tree status subset of task status")),
        }
    }
}

pub(crate) struct ExecutionTask {
    task: NodeTask,
    status_sender: Sender<TaskStatus>,
    abort_notifier: Arc<Notify>,
}

impl ExecutionTask {
    fn new(task: NodeTask, status_sender: Sender<TaskStatus>, abort_notifier: Arc<Notify>) -> Self {
        Self {
            task,
            status_sender,
            abort_notifier,
        }
    }

    #[instrument(skip(self), fields(task = %self.task.desc))]
    async fn execute(self) -> Result<()> {
        tokio::select! {
            _ = self.abort_notifier.notified() => {
                debug!("aborting execution task");
                self.status_sender.send(TaskStatus::Aborted).await?;
            }

            status = self.task.execute() => {
                debug!("task reached terminal state: {status:?}");
                self.status_sender.send(status.into()).await?;
            },


        }
        Ok(())
    }
}

#[cfg_attr(test, automock)]
pub(crate) trait RegisterTask<T>
where
    T: TaskControl,
{
    fn register(&self, task: NodeTask) -> Result<T>;
}

#[derive(Debug, Clone)]
pub(crate) struct Registry {
    sender: Sender<ExecutionTask>,
}

impl Registry {
    pub(crate) fn new(sender: Sender<ExecutionTask>) -> Self {
        Self { sender }
    }
}

impl RegisterTask<TaskHandle> for Registry {
    #[instrument(skip_all, fields(task = %task.desc))]
    fn register(&self, task: NodeTask) -> Result<TaskHandle> {
        let (status_send, status_recv) = channel(2);
        let notify = Arc::new(Notify::new());
        let exe_task = ExecutionTask::new(task, status_send, Arc::clone(&notify));
        let handle = TaskHandle::new(
            StatusQuerier::new(status_recv),
            TaskAborter::new(notify),
            exe_task.task.desc.clone(),
        );
        self.sender.try_send(exe_task)?;

        Ok(handle)
    }
}

#[derive(Debug)]
pub(crate) struct TaskHandle {
    querier: StatusQuerier,
    aborter: TaskAborter,
    desc: TaskDescription,
}

impl TaskHandle {
    fn new(querier: StatusQuerier, aborter: TaskAborter, desc: TaskDescription) -> Self {
        Self {
            querier,
            aborter,
            desc,
        }
    }
}

impl QueryTask for TaskHandle {
    fn query(&mut self) -> TaskStatus {
        self.querier.query()
    }
}

impl AbortTask for TaskHandle {
    #[instrument(skip_all, fields(desc=%self.desc))]
    fn abort(&mut self) {
        self.aborter.abort();
    }
}

pub(crate) trait TaskControl: QueryTask + AbortTask {}
impl<T: QueryTask + AbortTask> TaskControl for T {}

#[cfg_attr(test, automock)]
pub(crate) trait QueryTask {
    fn query(&mut self) -> TaskStatus;
}

#[cfg_attr(test, automock)]
pub(crate) trait AbortTask {
    fn abort(&mut self);
}

#[derive(Debug)]
struct StatusQuerier {
    status_recv: Receiver<TaskStatus>,
}

impl StatusQuerier {
    fn new(status_recv: Receiver<TaskStatus>) -> Self {
        Self { status_recv }
    }

    fn query(&mut self) -> TaskStatus {
        match self.status_recv.try_recv() {
            Ok(status) => status,
            Err(e) => match e {
                TryRecvError::Empty => TaskStatus::Running,
                TryRecvError::Disconnected => {
                    panic!("querying task after it has been disconnected",);
                }
            },
        }
    }
}

#[derive(Debug)]
struct TaskAborter {
    abort_notifier: Arc<Notify>,
}

impl TaskAborter {
    fn new(abort_notifier: Arc<Notify>) -> Self {
        Self { abort_notifier }
    }

    fn abort(&self) {
        self.abort_notifier.notify_one();
    }
}
