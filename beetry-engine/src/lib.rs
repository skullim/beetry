//! # Beetry Engine
//!
//! [`TreeEngine`] is the high-level entry point for loading a tree, preparing
//! the executor, and driving the tree until it reaches a terminal state.
//! It is the most idiomatic way to set up and execute trees.
//!
//! ## Engine states
//!
//! [`TreeEngine`] uses the typestate pattern to make the initialization flow
//! explicit and keep required setup steps ordered by construction.
//!
//! 1. [`TreeEngine`] in the [`Configured`] state: engine created, executor
//!    configured
//! 2. [`TreeEngine`] in the [`TreeLoaded`] state: tree attached by one of the
//!    supported methods
//! 3. [`TreeEngine`] in the [`Runnable`] state: tree ready to be ticked

#[cfg(feature = "plugin")]
mod plugin_support;
#[cfg(feature = "plugin")]
mod reconstruct;

use std::thread::JoinHandle;

use anyhow::{Result, anyhow};
use beetry_core::{Action, ActionBehavior, Node, TickStatus, Ticker, TickerError, Tree, leaf};
use beetry_exec::{Executor, ExecutorConfig, Ready as ExecutorReady, TaskHandle, TaskRegistry};
use futures::Stream;
use thiserror::Error as ThisError;
use tokio::sync::oneshot;
use tracing::error;

/// Typed-state engine for loading and running trees.
pub struct TreeEngine<S> {
    state: S,
}

/// Marker state for an engine that is configured and ready to load a tree.
pub struct Configured {
    executor: Executor<ExecutorReady>,
    builder: leaf::Builder<TaskRegistry, TaskHandle>,
}

/// Marker state for an engine with a loaded tree that has not started running
/// yet.
pub struct TreeLoaded<N> {
    tree: Tree<N>,
    executor: Executor<ExecutorReady>,
}

/// Configuration for constructing a [`TreeEngine`].
#[derive(Default)]
pub struct TreeEngineConfig {
    /// Configuration forwarded to the underlying executor.
    pub executor: ExecutorConfig,
}

/// Errors that can occur while driving a tree with the engine.
#[derive(Debug, ThisError)]
pub enum Error {
    /// The ticker failed while producing ticks for the tree.
    #[error(transparent)]
    TickerError(#[from] TickerError),
    /// The executor failed before the tree reached a terminal state.
    #[error("executor failed before tree reached terminal state: {0}")]
    ExecutorFailure(String),
}

#[expect(
    clippy::multiple_inherent_impl,
    reason = "other implementation is gated behind a feature"
)]
impl TreeEngine<Configured> {
    /// Creates a new engine in the [`Configured`] state.
    pub fn new(config: TreeEngineConfig) -> Self {
        let (executor, registry) = Executor::new(config.executor).into_ready_with_registry();
        Self {
            state: Configured {
                executor,
                builder: leaf::Builder::new(registry),
            },
        }
    }

    /// Registers an action defined by a custom [`ActionBehavior`].
    ///
    /// Each action must be registered with the [`TreeEngine`] before it can be
    /// inserted into a fully constructed [`Tree`].
    pub fn register_action(
        &self,
        behavior: impl ActionBehavior + 'static,
    ) -> Action<TaskRegistry, TaskHandle, impl ActionBehavior> {
        self.state.builder.action(behavior)
    }

    /// Attaches an already constructed tree and returns an engine in the
    /// [`TreeLoaded`] state.
    pub fn tree<N>(self, tree: Tree<N>) -> TreeEngine<TreeLoaded<N>>
    where
        N: Node,
    {
        TreeEngine {
            state: TreeLoaded {
                tree,
                executor: self.state.executor,
            },
        }
    }
}

impl<N> TreeEngine<TreeLoaded<N>>
where
    N: Node,
{
    /// Starts the executor and transitions the engine into the [`Runnable`]
    /// state.
    pub fn start_executor(self) -> Result<TreeEngine<Runnable<N>>> {
        let TreeLoaded { tree, mut executor } = self.state;
        let (shutdown_send, shutdown_recv) = oneshot::channel();
        let handle = std::thread::spawn(move || -> Result<()> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            runtime.block_on(async move {
                tokio::select! {
                    result = beetry_core::ExecutorConcept::run(&mut executor) => result,
                    _ = shutdown_recv => Ok(()),
                }
            })
        });

        Ok(TreeEngine {
            state: Runnable {
                tree,
                handle: ExeThreadHandle::new(shutdown_send, handle),
            },
        })
    }
}

/// Marker state for an engine whose tree is ready to be ticked.
pub struct Runnable<N> {
    tree: Tree<N>,
    handle: ExeThreadHandle,
}

impl<N> Drop for Runnable<N> {
    fn drop(&mut self) {
        if let Err(e) = self.handle.shutdown() {
            error!("failed to drop runnable tree engine state, details: {e}");
        }
    }
}

impl<N> TreeEngine<Runnable<N>>
where
    N: Node,
{
    /// Ticks the tree until it reaches a terminal status, then resets the tree
    /// before returning that status.
    pub async fn tick_till_terminal<S>(
        &mut self,
        mut ticker: Ticker<S>,
    ) -> Result<TickStatus, Error>
    where
        S: Stream<Item = ()>,
    {
        let status = ticker.tick_till_terminal(&mut self.state.tree).await?;
        self.state.tree.reset();
        Ok(status)
    }
}

struct ExeThreadHandle {
    shutdown_send: Option<oneshot::Sender<()>>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl ExeThreadHandle {
    fn new(shutdown_send: oneshot::Sender<()>, handle: JoinHandle<Result<()>>) -> Self {
        Self {
            shutdown_send: Some(shutdown_send),
            handle: Some(handle),
        }
    }

    fn shutdown(&mut self) -> Result<()> {
        if let Some(shutdown_send) = self.shutdown_send.take() {
            shutdown_send
                .send(())
                .map_err(|()| anyhow!("failed to send shutdown signal to executor thread"))?;
        }

        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| anyhow!("failed to join executor thread"))??;
        }
        Ok(())
    }
}
