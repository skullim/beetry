use anyhow::{Result, anyhow};
use beetry_core::leaf::Builder;
use beetry_core::{BoxNode, Node, TickStatus, Ticker, TickerError, Tree};
use beetry_editor_types::persistence::ValidTree;
use beetry_exec::{Executor, ExecutorConfig, Ready as ExecutorReady, WithRegistry};
use beetry_reconstruction::TreeReconstructor;
use beetry_serialization::{Deserializer, JsonDeserializer};
use futures::Stream;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use thiserror::Error as ThisError;
use tokio::sync::oneshot;
use tracing::error;

#[cfg(feature = "registry")]
#[expect(unused_imports, reason = "import all built-in registered nodes")]
use beetry_node::registry::*;

pub struct TreeEngine<S> {
    state: S,
}

pub struct Configured {
    executor: Executor<WithRegistry>,
}

pub struct Ready<N> {
    tree: Tree<N>,
    executor: Executor<ExecutorReady>,
}

#[derive(Default)]
pub struct TreeEngineConfig {
    pub executor: ExecutorConfig,
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    TickerError(#[from] TickerError),
    #[error("executor failed before tree reached terminal state: {0}")]
    ExecutorFailure(String),
}

impl TreeEngine<Configured> {
    pub fn new(config: TreeEngineConfig) -> Self {
        Self {
            state: Configured {
                executor: Executor::new(config.executor),
            },
        }
    }

    pub fn tree<N>(self, tree: Tree<N>) -> TreeEngine<Ready<N>>
    where
        N: Node,
    {
        let (executor, _registry) = self.state.executor.into_ready_with_registry();
        TreeEngine {
            state: Ready { tree, executor },
        }
    }

    pub fn tree_from_path(self, path: impl AsRef<Path>) -> Result<TreeEngine<Ready<BoxNode>>> {
        let valid_tree = load_valid_tree(path.as_ref())?;
        self.valid_tree(valid_tree)
    }

    pub async fn tree_from_dialog(self) -> Result<TreeEngine<Ready<BoxNode>>> {
        let path = select_import_file().await?;
        self.tree_from_path(path)
    }

    pub fn valid_tree(self, valid_tree: ValidTree) -> Result<TreeEngine<Ready<BoxNode>>> {
        let (executor, registry) = self.state.executor.into_ready_with_registry();
        let builder = Builder::new(registry);
        let mut reconstructor = TreeReconstructor::new()?;
        let tree = reconstructor.try_reconstruct(valid_tree, &builder)?;

        Ok(TreeEngine {
            state: Ready { tree, executor },
        })
    }
}

fn load_valid_tree(path: &Path) -> Result<ValidTree> {
    let content = std::fs::read_to_string(path)?;
    JsonDeserializer::deserialize(&content)
}

async fn select_import_file() -> Result<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .set_title("Select file to import")
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
        .ok_or_else(|| anyhow!("no file selected"))
}

impl<N> TreeEngine<Ready<N>>
where
    N: Node,
{
    pub fn start_executor(self) -> Result<TreeEngine<Runnable<N>>> {
        let Ready { tree, mut executor } = self.state;
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
