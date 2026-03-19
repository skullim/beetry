use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use beetry_core::{BoxNode, leaf::Builder};
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
    /// This is the path-based loading entry point for serialized trees.
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
        let (executor, registry) = self.state.executor.into_ready_with_registry();
        let builder = Builder::new(registry);
        let reconstructor = TreeReconstructor::new()?;
        let tree = reconstructor.try_reconstruct(valid_tree, &builder)?;

        Ok(TreeEngine {
            state: TreeLoaded { tree, executor },
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
