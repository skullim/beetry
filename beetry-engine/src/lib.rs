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
//!
//! ## Running a tree
//!
//! Typical flow with the built-in periodic ticker:
//!
//! ```no_run
//! use anyhow::Result;
//! use std::time::Duration;
//! use beetry_core::{PeriodicTick, Ticker};
//! use beetry_engine::{TreeEngine, TreeEngineConfig};
//!
//! # #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<()> {
//!     let mut engine = TreeEngine::new(TreeEngineConfig::default())
//!         .tree_from_path("tree.json")?
//!         .start_executor()?;
//!     let ticker = Ticker::new(PeriodicTick::new(Duration::from_millis(50)));
//!     let status = engine.tick_till_terminal(ticker).await?;
//!     Ok(())
//! }

//! ```
//! 
//! `PeriodicTick` is the default choice for most applications, but the engine
//! accepts any `Ticker<S>` where `S` is a `Stream<Item = ()>`.
//!
//! That means applications can define their own ticking policy and still use the
//! same engine:
//! ```no_run
//! use anyhow::Result;
//! use beetry_core::Ticker;
//! use beetry_engine::{TreeEngine, TreeEngineConfig};
//! use futures::stream;
//!
//! # #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<()> {
//!     let mut engine = TreeEngine::new(TreeEngineConfig::default())
//!         .tree_from_path("tree.json")?
//!         .start_executor()?;
//!
//!     let custom_ticks = stream::iter([(), (), (), ()]);
//!     let ticker = Ticker::new(custom_ticks);
//!     let _status = engine.tick_till_terminal(ticker).await?;
//!     Ok(())
//! }
//! ```
//! 
//! This keeps the default periodic model simple while making it easy to integrate
//! custom scheduling, external wake-up signals, or mixed ticking strategies.

#[cfg(feature = "plugin")]
mod plugin_support;
#[cfg(feature = "plugin")]
mod reconstruct;

use std::thread::JoinHandle;

use anyhow::{Result, anyhow};
use beetry_core::{Node, TickStatus, Ticker, TickerError, Tree};
use beetry_exec::{Executor, ExecutorConfig, Ready as ExecutorReady, WithRegistry};
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
    executor: Executor<WithRegistry>,
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
        Self {
            state: Configured {
                executor: Executor::new(config.executor),
            },
        }
    }

    /// Attaches an already constructed tree and returns an engine in the
    /// [`TreeLoaded`] state.
    pub fn tree<N>(self, tree: Tree<N>) -> TreeEngine<TreeLoaded<N>>
    where
        N: Node,
    {
        let (executor, _registry) = self.state.executor.into_ready_with_registry();
        TreeEngine {
            state: TreeLoaded { tree, executor },
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
