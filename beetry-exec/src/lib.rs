//! # Beetry Exec
//!
//! This crate is an internal Beetry implementation crate and is not considered
//! part of the public API. For public APIs, use the `beetry` crate.
//!
//! `beetry-exec` provides a task executor for Beetry action tasks.
//!
//! The crate provides the following types:
//!
//! - [`Executor`] runs registered [`NodeTask`] values to completion
//! - [`TaskRegistry`] schedules new tasks and returns a [`TaskHandle`] for
//!   querying or aborting them
//!
//! ## Lifecycle
//!
//! The executor is constructed in a setup state:
//!
//! 1. Create an executor with [`Executor::new`]
//! 2. Split it into a runnable executor and a task registry with
//!    [`Executor::into_ready_with_registry`]
//! 3. Spawn or await [`ExecutorConcept::run`] on the ready executor
//! 4. Use the registry to register [`NodeTask`] instances from elsewhere in the
//!    application
//!
//! This staged API makes it easy to hand the registry to tree code while the
//! executor runs in a dedicated task.

use std::{future::poll_fn, sync::Arc, task::Poll};

use anyhow::{Result, anyhow};
use beetry_core::{
    AbortTask, ExecutorConcept, NodeTask, QueryTask, RegisterTask, TaskDescription, TaskStatus,
};
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::sync::{
    Notify,
    mpsc::{self, Receiver, Sender, error::TryRecvError},
};
use tracing::{debug, instrument};

/// Configuration for an [`Executor`].
pub struct ExecutorConfig {
    task_channel_capacity: usize,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            task_channel_capacity: 8,
        }
    }
}

impl ExecutorConfig {
    /// Creates a configuration with the provided task queue capacity.
    pub fn new(task_channel_capacity: usize) -> Self {
        Self {
            task_channel_capacity,
        }
    }
}

/// Executor state containing the task registry before startup is finalized.
pub struct WithRegistry {
    registry: TaskRegistry,
}

pub struct ExecutionTask {
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

    #[instrument(skip(self), fields(task = %self.task.desc()))]
    async fn execute(self) -> Result<()> {
        tokio::select! {
            () = self.abort_notifier.notified() => {
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

/// A Beetry executor parameterized by its lifecycle state.
pub struct Executor<S> {
    recv: Receiver<ExecutionTask>,
    state: S,
}

/// Marker type for the pre-initialization executor state.
pub struct Init;
/// Marker type for the runnable executor state.
pub struct Ready;

impl Executor<Init> {
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Config contains only copy types now, but not marked Copy for future extensions"
    )]
    /// Creates a new executor with an internal bounded task channel.
    ///
    /// The returned executor is still in its setup phase.
    pub fn new(config: ExecutorConfig) -> Executor<WithRegistry> {
        let (sender, recv) = mpsc::channel(config.task_channel_capacity);
        let registry = TaskRegistry::new(sender);

        Executor {
            recv,
            state: WithRegistry { registry },
        }
    }
}

impl Executor<WithRegistry> {
    /// Finalizes setup and returns both the runnable executor and its registry.
    ///
    /// This is the transition point between initialization and runtime:
    ///
    /// - the returned [`Executor<Ready>`] can be driven with
    ///   [`ExecutorConcept::run`]
    /// - the returned [`TaskRegistry`] can be shared with code that needs to
    ///   schedule [`NodeTask`] values
    pub fn into_ready_with_registry(self) -> (Executor<Ready>, TaskRegistry) {
        (
            Executor {
                recv: self.recv,
                state: Ready,
            },
            self.state.registry,
        )
    }
}

impl ExecutorConcept for Executor<Ready> {
    #[instrument(skip(self), name = "Executor::run")]
    async fn run(&mut self) -> Result<()> {
        debug!("start running registered tasks");
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
                Some(exe_task) = self.recv.recv() => {
                    debug!("received new task to execute: {}", exe_task.task.desc());
                    tasks.push(exe_task.execute());
                },
                _ = execute_next_task_fut => {
                }

            }
        }
    }
}

#[derive(Debug, Clone)]
/// Registers [`NodeTask`] values with a running [`Executor`].
///
/// Each successful registration returns a [`TaskHandle`] that can be used to:
///
/// - query whether the task is still running or has reached a terminal state
/// - request cooperative abort for the task
pub struct TaskRegistry {
    sender: Sender<ExecutionTask>,
}

impl TaskRegistry {
    fn new(sender: Sender<ExecutionTask>) -> Self {
        Self { sender }
    }
}

impl RegisterTask<TaskHandle> for TaskRegistry {
    #[instrument(skip_all, fields(task = %task.desc()))]
    fn register(&self, task: NodeTask) -> Result<TaskHandle> {
        let (status_send, status_recv) = mpsc::channel(1);
        let notify = Arc::new(Notify::new());
        let exe_task = ExecutionTask::new(task, status_send, Arc::clone(&notify));
        let handle = TaskHandle::new(
            StatusQuerier::new(status_recv),
            TaskAborter::new(notify),
            exe_task.task.desc().clone(),
        );
        self.sender.try_send(exe_task).map_err(|err| {
            anyhow!(
                "failed to send execution task: {}",
                err.into_inner().task.desc()
            )
        })?;

        Ok(handle)
    }
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

#[derive(Debug)]
/// Handle for querying and aborting a registered task.
///
/// The handle is a lightweight client-side view over task state:
///
/// - [`QueryTask::query`] returns the latest observed [`TaskStatus`]
/// - [`AbortTask::abort`] requests that the running task transitions to
///   [`TaskStatus::Aborted`]
///
/// Querying is non-blocking. Until a terminal status is received, querying
/// reports [`TaskStatus::Running`].
pub struct TaskHandle {
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
