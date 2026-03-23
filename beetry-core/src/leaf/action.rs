use core::fmt;
use std::{sync::Arc, time::Duration};

use anyhow::Result;
use tracing::{debug, error};

use crate::{
    Node, NodeTask, TickStatus,
    task::{RegisterTask, TaskHandle, TaskStatus},
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait Behavior {
    /// Construct the task that should be scheduled for this action.
    ///
    /// This is called when the action transitions from idle into execution.
    fn task(&mut self) -> Result<NodeTask>;

    /// Reset any action-local state for a fresh run.
    fn reset(&mut self) {}

    /// Hook called on [`TaskStatus::Running`].
    fn on_running(&mut self) -> Result<()> {
        Ok(())
    }
    /// Hook called on [`TaskStatus::Success`].
    fn on_success(&mut self) -> Result<()> {
        Ok(())
    }

    /// Hook called on [`TaskStatus::Failure`]
    fn on_failure(&mut self) -> Result<()> {
        Ok(())
    }

    /// Hook called after the action is aborted.
    fn on_aborted(&mut self) -> Result<()> {
        Ok(())
    }
}

pub type BoxBehavior = Box<dyn Behavior>;
impl Behavior for BoxBehavior {
    fn task(&mut self) -> Result<NodeTask> {
        (**self).task()
    }
    fn reset(&mut self) {
        (**self).reset();
    }
    fn on_running(&mut self) -> Result<()> {
        (**self).on_running()
    }
    fn on_success(&mut self) -> Result<()> {
        (**self).on_success()
    }
    fn on_failure(&mut self) -> Result<()> {
        (**self).on_failure()
    }
    fn on_aborted(&mut self) -> Result<()> {
        (**self).on_aborted()
    }
}

fn visit_status(behavior: &mut impl Behavior, status: TaskStatus) -> TaskStatus {
    match status {
        TaskStatus::Success => behavior.on_success(),
        TaskStatus::Running => behavior.on_running(),
        TaskStatus::Failure => behavior.on_failure(),
        TaskStatus::Aborted => behavior.on_aborted(),
    }
    .map(|()| status)
    .unwrap_or(TaskStatus::Failure)
}

pub struct Action<R, TH, B>
where
    R: RegisterTask<TH>,
    TH: TaskHandle,
    B: Behavior,
{
    behavior: B,
    registry: Arc<R>,
    state: State<TH>,
}

impl<R, TH, B> Action<R, TH, B>
where
    R: RegisterTask<TH>,
    TH: TaskHandle,
    B: Behavior,
{
    pub fn new(behavior: B, registry: Arc<R>) -> Self {
        Self {
            behavior,
            registry,
            state: State::Idle,
        }
    }
}

impl<R, TH, B> Node for Action<R, TH, B>
where
    R: RegisterTask<TH>,
    TH: TaskHandle,
    B: Behavior,
{
    fn tick(&mut self) -> TickStatus {
        match &mut self.state {
            State::Idle => match self.behavior.task() {
                Ok(task) => match self.registry.register(task) {
                    Ok(handle) => {
                        self.state = State::Running(handle);
                        TickStatus::Running
                    }
                    Err(e) => {
                        error!("task registration failed: {e}");
                        TickStatus::Failure
                    }
                },
                Err(e) => {
                    error!("creating task failed: {e}");
                    TickStatus::Failure
                }
            },
            State::Running(handle) => {
                let status = handle.query();
                visit_status(&mut self.behavior, status);

                let status: TickStatus = status.try_into().unwrap();
                if status.is_terminal() {
                    debug!("task completed, returning to idle");
                    self.state = State::Idle;
                }

                status
            }
        }
    }

    fn reset(&mut self) {
        assert!(
            matches!(self.state, State::Idle),
            "requested action reset during task execution"
        );
        self.behavior.reset();
    }

    fn abort(&mut self) {
        let mut on_aborted = || {
            if let Err(e) = self.behavior.on_aborted() {
                error!("on aborted hook failed: {e}");
            }
        };

        match &mut self.state {
            State::Idle => {
                on_aborted();
            }
            State::Running(task_handle) => {
                task_handle.abort();
                loop {
                    let status = task_handle.query();
                    if status.is_terminal() {
                        debug!("aborted task terminal status: {status:?}");
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                on_aborted();
                debug!("switching state to idle");
                self.state = State::Idle;
            }
        }
    }
}

enum State<TH>
where
    TH: TaskHandle,
{
    Idle,
    Running(TH),
}

impl<TH> fmt::Display for State<TH>
where
    TH: TaskHandle,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Running(_) => write!(f, "Running"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bon::builder;
    use mockall::mock;

    use super::*;
    use crate::{
        Task, TaskDescription,
        task::{AbortTask, MockRegisterTask, QueryTask},
    };

    mock! {
        TaskHandle {}

    impl QueryTask for TaskHandle {
        fn query(&mut self) -> TaskStatus;
    }

    impl AbortTask for TaskHandle {
        fn abort(&mut self);
    }
    }

    struct TaskStub;

    impl TaskStub {
        fn new() -> Self {
            Self {}
        }
    }

    impl Task for TaskStub {
        async fn run(self) -> TickStatus {
            TickStatus::Success
        }
        fn task_desc(&self) -> TaskDescription {
            TaskDescription::from_str("TaskStub").unwrap()
        }
    }

    #[builder]
    fn task_handle(
        query_times: usize,
        statuses: Vec<TaskStatus>,
        abort_times: Option<usize>,
    ) -> MockTaskHandle {
        let mut m = MockTaskHandle::new();
        let mut it = statuses.into_iter();
        m.expect_query()
            .returning(move || it.next().unwrap())
            .times(query_times);

        if let Some(abort_times) = abort_times {
            m.expect_abort().times(abort_times).return_const(());
        }
        m
    }

    #[test]
    fn action_success() {
        let mut registry = MockRegisterTask::<MockTaskHandle>::new();
        registry
            .expect_register()
            .returning(|_| {
                Ok(task_handle()
                    .query_times(1)
                    .statuses(vec![TaskStatus::Success])
                    .call())
            })
            .once();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Ok(NodeTask::new(TaskStub::new())));
        behavior
            .expect_on_success()
            .once()
            .returning(|| Result::Ok(()));

        let mut action = Action::new(behavior, Arc::new(registry));
        assert_eq!(action.tick(), TickStatus::Running);
        assert_eq!(action.tick(), TickStatus::Success);
    }

    #[test]
    fn action_running() {
        let mut registry = MockRegisterTask::<MockTaskHandle>::new();
        registry
            .expect_register()
            .returning(|_| {
                Ok(task_handle()
                    .query_times(1)
                    .statuses(vec![TaskStatus::Running])
                    .call())
            })
            .once();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Ok(NodeTask::new(TaskStub::new())));
        behavior
            .expect_on_running()
            .once()
            .returning(|| Result::Ok(()));

        let mut action = Action::new(behavior, Arc::new(registry));
        assert_eq!(action.tick(), TickStatus::Running);
        assert_eq!(action.tick(), TickStatus::Running);
    }

    #[test]
    fn action_failure() {
        let mut registry = MockRegisterTask::<MockTaskHandle>::new();
        registry
            .expect_register()
            .returning(|_| {
                Ok(task_handle()
                    .query_times(1)
                    .statuses(vec![TaskStatus::Failure])
                    .call())
            })
            .once();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Ok(NodeTask::new(TaskStub::new())));
        behavior
            .expect_on_failure()
            .once()
            .returning(|| Result::Ok(()));

        let mut action = Action::new(behavior, Arc::new(registry));

        assert_eq!(action.tick(), TickStatus::Running);
        assert_eq!(action.tick(), TickStatus::Failure);
    }

    #[test]
    fn task_creation_failure() {
        let registry = MockRegisterTask::<MockTaskHandle>::new();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Err(anyhow::anyhow!("task creation failed")));

        let mut action = Action::new(behavior, Arc::new(registry));
        assert_eq!(action.tick(), TickStatus::Failure);
    }

    #[test]
    fn task_registration_failure() {
        let mut registry = MockRegisterTask::<MockTaskHandle>::new();
        registry
            .expect_register()
            .returning(|_| Err(anyhow::anyhow!("registration failed")))
            .once();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Ok(NodeTask::new(TaskStub::new())));

        let mut action = Action::new(behavior, Arc::new(registry));
        assert_eq!(action.tick(), TickStatus::Failure);
    }

    #[test]
    fn action_abort_when_running() {
        let mut registry = MockRegisterTask::<MockTaskHandle>::new();
        registry
            .expect_register()
            .returning(|_| {
                Ok(task_handle()
                    .query_times(3)
                    .statuses(vec![
                        TaskStatus::Running,
                        TaskStatus::Running,
                        TaskStatus::Aborted,
                    ])
                    .abort_times(1)
                    .call())
            })
            .once();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Ok(NodeTask::new(TaskStub::new())));
        behavior
            .expect_on_aborted()
            .once()
            .returning(|| Result::Ok(()));
        behavior
            .expect_on_running()
            .once()
            .returning(|| Result::Ok(()));

        let mut action = Action::new(behavior, Arc::new(registry));

        assert_eq!(action.tick(), TickStatus::Running);
        assert_eq!(action.tick(), TickStatus::Running);
        action.abort();
    }

    #[test]
    fn action_abort_when_idle() {
        let registry = MockRegisterTask::<MockTaskHandle>::new();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_on_aborted()
            .once()
            .returning(|| Result::Ok(()));

        let mut action = Action::new(behavior, Arc::new(registry));

        action.abort();
    }

    #[test]
    fn action_reset() {
        let registry = MockRegisterTask::<MockTaskHandle>::new();

        let mut behavior = MockBehavior::new();
        behavior.expect_reset().once().return_const(());

        let mut action = Action::new(behavior, Arc::new(registry));
        action.reset();
    }
}
