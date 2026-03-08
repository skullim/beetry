use beetry_editor_backend::api;
use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use beetry_editor_types::persistence::{EditorStateStore, ValidTree};
use beetry_serialization::{Deserializer, JsonDeserializer, JsonSerializer, Serializer};
use beetry_editor_backend::api::NodeUiQuery;
use dioxus::prelude::{ReadableExt, WritableExt};
use rfd::FileDialog;

use crate::components::editor::state::svg::DimensionState;
use crate::Backend;
use crate::signals::RenderRequests;
use crate::ui::error::ErrorQueueState;
use crate::ui::transfer;

pub(super) fn export_handlers(
    mut error_queue: ErrorQueueState,
    backend: Backend,
) -> transfer::export::Handlers {
    let on_project = move |()| {
        if let Err(e) = do_export_project(backend) {
            error_queue.push(e);
        }
        Ok(())
    };

    let on_valid_tree = move |()| {
        if let Err(e) = do_export_valid_tree(backend) {
            error_queue.push(e);
        }
        Ok(())
    };

    transfer::export::Handlers::new(on_project, on_valid_tree)
}

pub(super) fn import_handlers(
    dimensions: DimensionState,
    mut error_queue: ErrorQueueState,
    backend: Backend,
    mut render_requests: RenderRequests,
) -> transfer::import::Handlers {
    let on_click = move |()| {
        match do_import(backend, dimensions) {
            Ok(()) => {
                render_requests.request_all();
            }
            Err(e) => {
                error_queue.push(e);
            }
        }
        Ok(())
    };

    transfer::import::Handlers::new(on_click)
}

fn do_export_project(backend: Backend) -> Result<()> {
    let state = backend.with_peek(api::project::export)?;
    export_project_to_file(&state)?;
    Ok(())
}

fn do_export_valid_tree(backend: Backend) -> Result<()> {
    let tree = backend.with_peek(api::project::export_valid_tree)?;
    export_valid_tree_to_file(tree)?;
    Ok(())
}

fn export_project_to_file(editor_state: &EditorStateStore) -> Result<()> {
    let serialized = JsonSerializer::serialize(&editor_state)?;
    let file_path = select_export_file()?;

    std::fs::write(&file_path, serialized)
        .with_context(|| format!("Failed to save file '{}'", file_path.display()))
}

fn export_valid_tree_to_file(valid_tree: ValidTree) -> Result<()> {
    let serialized = JsonSerializer::serialize(&valid_tree.into_inner())?;
    let file_path = select_export_file()?;

    std::fs::write(&file_path, serialized)
        .with_context(|| format!("Failed to save file '{}'", file_path.display()))
}

fn do_import(mut backend: Backend, mut dimensions: DimensionState) -> Result<()> {
    let state = import_project_from_file()?;
    backend.with_mut(|s| api::project::import(s, state))?;
    backend.with_peek(|s| {
        let query = api::ui::node::query(s);
        dimensions.resize(query.positions());
    });
    Ok(())
}

fn import_project_from_file() -> Result<EditorStateStore> {
    let path = select_import_file()?;
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("import path should contain extension"))?;
    if !matches!(ext, "json") {
        return Err(anyhow!("unsupported extension: {ext:?}"));
    }

    let mut file = std::fs::File::open(path)?;
    let mut content_buffer = String::new();
    file.read_to_string(&mut content_buffer)?;
    let store: EditorStateStore = JsonDeserializer::deserialize(&content_buffer)?;
    Ok(store)
}

fn select_export_file() -> Result<PathBuf> {
    FileDialog::new()
        .add_filter("JSON files", &["json"])
        .set_title("Save behavior tree project as...")
        .set_file_name("behavior_tree.json")
        .save_file()
        .ok_or_else(|| anyhow!("No file selected"))
}

fn select_import_file() -> Result<PathBuf> {
    FileDialog::new()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .set_title("Select file to import")
        .pick_file()
        .ok_or_else(|| anyhow!("No file selected"))
}
