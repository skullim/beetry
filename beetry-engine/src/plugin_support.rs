use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use beetry_core::BoxNode;
use beetry_editor_types::persistence::tree::ValidTreeStore;
#[expect(unused_imports, reason = "import all built-in registered nodes")]
use beetry_node::plugin::*;
use beetry_serialization::json;

use crate::{Configured, TreeEngine, TreeLoaded, reconstruct::TreeReconstructor};

type BoxTreeLoaded = TreeLoaded<BoxNode>;

impl TreeEngine<Configured> {
    /// Loads a tree from a file path and returns an engine in the
    /// [`TreeLoaded`] state.
    ///
    ///```no_run
    /// use std::time::Duration;
    ///
    /// use anyhow::Result;
    /// use beetry_core::{PeriodicTick, Ticker};
    /// use beetry_engine::{TreeEngine, TreeEngineConfig};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// async fn main() -> Result<()> {
    ///     let mut engine = TreeEngine::new(TreeEngineConfig::default())
    ///         .tree_from_path("tree.json")?
    ///         .start_executor()?;
    ///     let ticker = Ticker::new(PeriodicTick::new(Duration::from_millis(50)));
    ///     let status = engine.tick_till_terminal(ticker).await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// `PeriodicTick` is the default choice for most applications, but the
    /// engine accepts any `Ticker<S>` where `S` is a `Stream<Item = ()>`.
    /// ```no_run
    /// use anyhow::Result;
    /// use beetry_core::Ticker;
    /// use beetry_engine::{TreeEngine, TreeEngineConfig};
    /// use futures::stream;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// async fn main() -> Result<()> {
    ///     let mut engine = TreeEngine::new(TreeEngineConfig::default())
    ///         .tree_from_path("tree.json")?
    ///         .start_executor()?;
    ///
    ///     let custom_ticks = stream::iter([(), (), (), ()]);
    ///     let ticker = Ticker::new(custom_ticks);
    ///     let _status = engine.tick_till_terminal(ticker).await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn tree_from_path(self, path: impl AsRef<Path>) -> Result<TreeEngine<BoxTreeLoaded>> {
        let valid_tree = load_valid_tree(path.as_ref())?;
        self.valid_tree(valid_tree)
    }

    /// Opens a file picker, loads the selected tree, and returns an engine in
    /// the [`TreeLoaded`] state.
    pub async fn tree_from_dialog(self) -> Result<TreeEngine<TreeLoaded<BoxNode>>> {
        let path = select_import_file().await?;
        self.tree_from_path(path)
    }

    /// Attaches an already validated tree store and reconstructs the runtime
    /// tree, returning an engine in the [`TreeLoaded`] state.
    ///
    /// This is the in-memory loading entry point when the caller already has a
    /// [`ValidTreeStore`].
    pub fn valid_tree(self, valid_tree: ValidTreeStore) -> Result<TreeEngine<BoxTreeLoaded>> {
        let reconstructor = TreeReconstructor::new()?;
        let tree = reconstructor.try_reconstruct(valid_tree, &self.state.builder)?;

        Ok(TreeEngine {
            state: TreeLoaded {
                tree,
                executor: self.state.executor,
            },
        })
    }
}

fn load_valid_tree(path: &Path) -> Result<ValidTreeStore> {
    let content = std::fs::read_to_string(path)?;
    json::load_from(&content)
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
