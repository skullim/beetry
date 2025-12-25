use std::io::Read;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use beetry_editor_types::{EditorStateStore, ValidTree};
use beetry_serialization::{Deserializer, JsonDeserializer, JsonSerializer, Serializer};
use dioxus::prelude::*;
use rfd::FileDialog;

use crate::editor::ServiceContext;
use crate::ui::transfer;

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

#[component]
pub(crate) fn Toolbar() -> Element {
    let mut export_result = use_signal(transfer::OperationResult::default);
    let mut valid_tree_export_result = use_signal(transfer::OperationResult::default);
    let mut import_result = use_signal(transfer::OperationResult::default);

    {
        let on_project_export = move |()| {
            let service = use_context::<ServiceContext>();
            let read = service.service.read();

            let export_project_result = read.export_api().export_project();
            match export_project_result {
                Ok(state) => match export_project_to_file(state) {
                    Ok(()) => {
                        export_result.set(transfer::OperationResult::new(
                            "Export successful",
                            transfer::OperationStatus::Success,
                        ));
                    }
                    Err(e) => {
                        export_result.set(transfer::OperationResult::new(
                            format!("Export failed:\n{e}"),
                            transfer::OperationStatus::Error,
                        ));
                    }
                },
                Err(e) => {
                    export_result.set(transfer::OperationResult::new(
                        format!("Export failed:\n{e}"),
                        transfer::OperationStatus::Error,
                    ));
                }
            };
        };

        let on_valid_tree_export = move |()| {
            let service = use_context::<ServiceContext>();
            let read = service.service.read();

            let export_valid_tree_result = read.export_api().export_valid_tree();
            match export_valid_tree_result {
                Ok(tree) => match export_valid_tree_to_file(tree) {
                    Ok(()) => {
                        valid_tree_export_result.set(transfer::OperationResult::new(
                            "Valid tree export successful",
                            transfer::OperationStatus::Success,
                        ));
                    }
                    Err(e) => {
                        valid_tree_export_result.set(transfer::OperationResult::new(
                            format!("Export failed:\n{e}"),
                            transfer::OperationStatus::Error,
                        ));
                    }
                },
                Err(e) => {
                    valid_tree_export_result.set(transfer::OperationResult::new(
                        format!("Export failed:\n{e}"),
                        transfer::OperationStatus::Error,
                    ));
                }
            };
        };

        let on_import = move |()| {
            let result = import_project_from_file();

            match result {
                Ok(state) => {
                    let mut service = use_context::<ServiceContext>();
                    let mut write = service.service.write();
                    match write.import_api().import_project(state) {
                        Ok(()) => {
                            import_result.set(transfer::OperationResult::new(
                                "Import successful",
                                transfer::OperationStatus::Success,
                            ));
                        }
                        Err(e) => {
                            import_result.set(transfer::OperationResult::new(
                                format!("Import failed:\n{e}"),
                                transfer::OperationStatus::Error,
                            ));
                        }
                    }
                }
                Err(e) => {
                    import_result.set(transfer::OperationResult::new(
                        format!("Import failed:\n{e}"),
                        transfer::OperationStatus::Error,
                    ));
                }
            }
        };

        use_context_provider(move || ToolbarHandlers {
            import: transfer::ImportHandlers::new(on_import),
            export: transfer::ExportHandlers::new(on_project_export, on_valid_tree_export),
        });
    }

    rsx! {
        transfer::ExportProject { result: export_result }
        transfer::ExportValidTree { result: valid_tree_export_result }
        transfer::Import { result: import_result }
    }
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

#[derive(Debug, Clone)]
pub struct ToolbarHandlers {
    pub(crate) import: transfer::ImportHandlers,
    pub(crate) export: transfer::ExportHandlers,
}
