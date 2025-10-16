use std::path::PathBuf;

use anyhow::{Result, anyhow};
use beetry_serialization::{JsonSerializer, Serializer};
use dioxus::prelude::*;
use rfd::FileDialog;

use crate::{
    definitions::NodeId,
    project::{EditorMetadata, ProjectData},
    ui::{self, channel, edge, transfer},
};
#[derive(Debug, Clone)]
pub(crate) struct ToolbarHandlers {
    pub(crate) import: transfer::ImportHandlers,
    pub(crate) export: transfer::ExportHandlers,
}

#[component]
pub(crate) fn Toolbar(
    mut id: Signal<NodeId>,
    mut ui_nodes: Signal<ui::NodeMap>,
    mut edge_tracker: Signal<edge::Tracker>,
    mut channel_tracker: Signal<channel::Tracker>,
) -> Element {
    let mut export_result = use_signal(transfer::OperationResult::default);
    let mut import_result = use_signal(transfer::OperationResult::default);

    {
        let on_export = move |_| {
            let nodes = ui_nodes.peek();
            let edge_tracker = edge_tracker.read();
            let result = export_to_file(&nodes, id(), &edge_tracker, &channel_tracker.peek());

            match result {
                Ok(()) => {
                    export_result.set(transfer::OperationResult::new(
                        "Export successful",
                        transfer::OperationStatus::Success,
                    ));
                }
                Err(error) => {
                    export_result.set(transfer::OperationResult::new(
                        format!("Export failed:\n{}", error),
                        transfer::OperationStatus::Error,
                    ));
                }
            }
        };

        let on_import = move |_| {
            let result = import_from_file();

            match result {
                Ok(mut data) => {
                    ui_nodes.with_mut(|elements| std::mem::swap(elements, &mut data.nodes));
                    edge_tracker.with_mut(|tracker| {
                        let mut new = edge::Tracker::from_edges(data.edges);
                        std::mem::swap(tracker, &mut new);
                    });
                    id.with_mut(|id| *id = data.last_id);

                    channel_tracker
                        .with_mut(|tracker| std::mem::swap(tracker, &mut data.channel_tracker));

                    import_result.set(transfer::OperationResult::new(
                        "Import successful",
                        transfer::OperationStatus::Success,
                    ));
                }
                Err(error) => {
                    import_result.set(transfer::OperationResult::new(
                        format!("Import failed:\n{}", error),
                        transfer::OperationStatus::Error,
                    ));
                }
            }
        };

        use_context_provider(move || ToolbarHandlers {
            import: transfer::ImportHandlers::new(on_import),
            export: transfer::ExportHandlers::new(on_export),
        });
    }

    rsx! {
        transfer::Export { result: export_result }
        transfer::Import { result: import_result }
    }
}

fn export_to_file(
    nodes: &ui::NodeMap,
    last_id: NodeId,
    edge_tracker: &edge::Tracker,
    channel_tracker: &channel::Tracker,
) -> Result<()> {
    let export = ProjectData::export(nodes, last_id, edge_tracker, channel_tracker)?;
    let serialized = JsonSerializer::serialize(&export)?;
    let file_path = select_export_file()?;

    std::fs::write(&file_path, serialized)
        .map_err(|e| anyhow!("Failed to save file '{}': {}", file_path.display(), e))
}

fn select_export_file() -> Result<PathBuf> {
    FileDialog::new()
        .add_filter("JSON files", &["json"])
        .set_title("Save behavior tree project as...")
        .set_file_name("behavior_tree.json")
        .save_file()
        .ok_or_else(|| anyhow!("No file selected"))
}

fn import_from_file() -> Result<EditorMetadata> {
    let file_path = select_import_file()?;
    ProjectData::import(&file_path)
}

fn select_import_file() -> Result<PathBuf> {
    FileDialog::new()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .set_title("Select file to import")
        .pick_file()
        .ok_or_else(|| anyhow!("No file selected"))
}
