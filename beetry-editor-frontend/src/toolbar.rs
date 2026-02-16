use std::io::Read;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use beetry_editor_types::persistence::{EditorStateStore, ValidTree};
use beetry_serialization::{Deserializer, JsonDeserializer, JsonSerializer, Serializer};
use dioxus::prelude::*;
use rfd::FileDialog;

use crate::Backend;
use crate::signals::RenderRequests;
use crate::ui::error;
use crate::ui::error::ErrorQueueState;
use crate::ui::transfer;

#[component]
pub(crate) fn Toolbar(render_requests: RenderRequests) -> Element {
    let mut error_queue = use_context::<ErrorQueueState>();
    let backend = use_context::<Backend>();

    {
        let on_project = move |()| {
            match do_export_project(backend) {
                Ok(()) => {}
                Err(e) => {
                    error_queue.with_mut(|q| q.push("export-project", e.to_string()));
                }
            };
            Ok(())
        };

        let on_valid_tree = move |()| {
            match do_export_valid_tree(backend) {
                Ok(()) => {}
                Err(e) => {
                    error_queue.with_mut(|q| q.push("export-valid-tree", e.to_string()));
                }
            };
            Ok(())
        };

        use_context_provider(move || transfer::export::Handlers::new(on_project, on_valid_tree));
    }

    {
        let on_click = move |()| {
            match do_import(backend) {
                Ok(()) => {
                    render_requests.nodes.request();
                    render_requests.edges.request();
                    render_requests.channels.request();
                    render_requests.channel_edges.request();
                }
                Err(e) => {
                    error_queue.with_mut(|q| q.push("import-project", e.to_string()));
                }
            };
            Ok(())
        };

        use_context_provider(move || transfer::import::Handlers::new(on_click));
    }

    rsx! {
        transfer::ExportProject {}
        transfer::ExportValidTree {}
        transfer::Import {}
        error::Dialog {}
    }
}

fn do_export_project(backend: Backend) -> Result<()> {
    let state = backend.with_peek(beetry_editor_backend::api::project::export)?;
    export_project_to_file(state)?;
    Ok(())
}

fn do_export_valid_tree(backend: Backend) -> Result<()> {
    let tree = backend.with_peek(beetry_editor_backend::api::project::export_valid_tree)?;
    export_valid_tree_to_file(tree)?;
    Ok(())
}

fn export_project_to_file(editor_state: EditorStateStore) -> Result<()> {
    let serialized = JsonSerializer::serialize(&editor_state)?;
    let file_path = select_export_file()?;

    std::fs::write(&file_path, serialized)
        .map_err(|e| anyhow!("Failed to save file '{}': {}", file_path.display(), e))
}

fn export_valid_tree_to_file(valid_tree: ValidTree) -> Result<()> {
    let serialized = JsonSerializer::serialize(&valid_tree.into_inner())?;
    let file_path = select_export_file()?;

    std::fs::write(&file_path, serialized)
        .map_err(|e| anyhow!("Failed to save file '{}': {}", file_path.display(), e))
}

fn do_import(mut backend: Backend) -> Result<()> {
    let state = import_project_from_file()?;
    backend.with_mut(|s| beetry_editor_backend::api::project::import(s, state))?;
    Ok(())
}

pub(crate) fn import_project_from_file() -> Result<EditorStateStore> {
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
