use crate::{
    Node, NodeTask, TickStatus,
    task::{RegisterTask, TaskControl, TaskStatus},
};
use anyhow::Result;
use core::fmt;
use std::{sync::Arc, time::Duration};
use tracing::{debug, error};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait Behavior {
    fn task(&mut self) -> Result<NodeTask>;
    fn reset(&mut self) {}

    // hooks for additional behavior that is executed based on the task status
    // useful for propagating data between nodes and cleanup
    fn on_running(&mut self) {}
    fn on_success(&mut self) {}
    fn on_failure(&mut self) {}
    fn on_aborted(&mut self) {}
}

pub type BoxBehavior = Box<dyn Behavior>;
impl Behavior for BoxBehavior {
    fn task(&mut self) -> Result<NodeTask> {
        (**self).task()
    }
    fn reset(&mut self) {
        (**self).reset();
    }
    fn on_running(&mut self) {
        (**self).on_running();
    }
    fn on_success(&mut self) {
        (**self).on_success();
    }
    fn on_failure(&mut self) {
        (**self).on_failure();
    }
    fn on_aborted(&mut self) {
        (**self).on_aborted();
    }
}

fn visit_status(behavior: &mut dyn Behavior, status: TaskStatus) {
    match status {
        TaskStatus::Success => behavior.on_success(),
        TaskStatus::Running => behavior.on_running(),
        TaskStatus::Failure => behavior.on_failure(),
        TaskStatus::Aborted => behavior.on_aborted(),
    }
}

pub struct Action<R, TC, B>
where
    R: RegisterTask<TC>,
    TC: TaskControl,
    B: Behavior,
{
    behavior: B,
    registry: Arc<R>,
    state: State<TC>,
}

impl<R, TC, B> Action<R, TC, B>
where
    R: RegisterTask<TC>,
    TC: TaskControl,
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

impl<R, TC, B> Node for Action<R, TC, B>
where
    R: RegisterTask<TC>,
    TC: TaskControl,
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
                debug!("queried status: {status:?}");
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
        self.behavior.reset();
        self.state = State::Idle;
    }

    fn abort(&mut self) {
        match &mut self.state {
            State::Idle => {
                self.behavior.on_aborted();
            }
            State::Running(task_handle) => {
                task_handle.abort();
                //@todo caution: blocking call, long term refactor into strategy to choose for how long the status is polled on abort request
                loop {
                    let status = task_handle.query();
                    debug!("aborted task terminal status: {status:?}");
                    if status.is_terminal() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                self.behavior.on_aborted();
                debug!("switching state to idle");
                self.state = State::Idle;
            }
        }
    }
}

enum State<TC>
where
    TC: TaskControl,
{
    Idle,
    Running(TC),
}

impl<TC> fmt::Display for State<TC>
where
    TC: TaskControl,
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
        TaskControl {}

    impl QueryTask for TaskControl {
        fn query(&mut self) -> TaskStatus;
    }

    impl AbortTask for TaskControl {
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
    fn task_control(
        query_times: usize,
        statuses: Vec<TaskStatus>,
        abort_times: Option<usize>,
    ) -> MockTaskControl {
        let mut m = MockTaskControl::new();
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
    fn test_action_success() {
        let mut registry = MockRegisterTask::<MockTaskControl>::new();
        registry
            .expect_register()
            .returning(|_| {
                Ok(task_control()
                    .query_times(1)
                    .statuses(vec![TaskStatus::Success])
                    .call())
            })
            .once();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Ok(NodeTask::new(TaskStub::new())));
        behavior.expect_on_success().once().return_const(());

        let mut action = Action::new(behavior, Arc::new(registry));
        assert_eq!(action.tick(), TickStatus::Running);
        assert_eq!(action.tick(), TickStatus::Success);
    }

    #[test]
    fn test_action_running() {
        let mut registry = MockRegisterTask::<MockTaskControl>::new();
        registry
            .expect_register()
            .returning(|_| {
                Ok(task_control()
                    .query_times(1)
                    .statuses(vec![TaskStatus::Running])
                    .call())
            })
            .once();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Ok(NodeTask::new(TaskStub::new())));
        behavior.expect_on_running().once().return_const(());

        let mut action = Action::new(behavior, Arc::new(registry));
        assert_eq!(action.tick(), TickStatus::Running);
        assert_eq!(action.tick(), TickStatus::Running);
    }

    #[test]
    fn test_action_failure() {
        let mut registry = MockRegisterTask::<MockTaskControl>::new();
        registry
            .expect_register()
            .returning(|_| {
                Ok(task_control()
                    .query_times(1)
                    .statuses(vec![TaskStatus::Failure])
                    .call())
            })
            .once();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Ok(NodeTask::new(TaskStub::new())));
        behavior.expect_on_failure().once().return_const(());

        let mut action = Action::new(behavior, Arc::new(registry));

        assert_eq!(action.tick(), TickStatus::Running);
        assert_eq!(action.tick(), TickStatus::Failure);
    }

    #[test]
    fn test_task_creation_failure() {
        let registry = MockRegisterTask::<MockTaskControl>::new();

        let mut behavior = MockBehavior::new();
        behavior
            .expect_task()
            .returning(|| Err(anyhow::anyhow!("task creation failed")));

        let mut action = Action::new(behavior, Arc::new(registry));
        assert_eq!(action.tick(), TickStatus::Failure);
    }

    #[test]
    fn test_task_registration_failure() {
        let mut registry = MockRegisterTask::<MockTaskControl>::new();
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
    fn test_action_abort_when_running() {
        let mut registry = MockRegisterTask::<MockTaskControl>::new();
        registry
            .expect_register()
            .returning(|_| {
                Ok(task_control()
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
        behavior.expect_on_aborted().once().return_const(());
        behavior.expect_on_running().once().return_const(());

        let mut action = Action::new(behavior, Arc::new(registry));

        assert_eq!(action.tick(), TickStatus::Running);
        assert_eq!(action.tick(), TickStatus::Running);
        action.abort();
    }

    #[test]
    fn test_action_abort_when_idle() {
        let registry = MockRegisterTask::<MockTaskControl>::new();

        let mut behavior = MockBehavior::new();
        behavior.expect_on_aborted().once().return_const(());

        let mut action = Action::new(behavior, Arc::new(registry));

        action.abort();
    }

    #[test]
    fn test_action_reset() {
        let registry = MockRegisterTask::<MockTaskControl>::new();

        let mut behavior = MockBehavior::new();
        behavior.expect_reset().once().return_const(());

        let mut action = Action::new(behavior, Arc::new(registry));
        action.reset();
    }
}
