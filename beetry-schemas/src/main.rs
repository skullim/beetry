use std::path::PathBuf;

use anyhow::{Result, anyhow};
use beetry_editor_types::persistence::{editor::StateStore, tree::ValidTreeStore};
use schemars::schema_for;

fn main() -> Result<()> {
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| anyhow!("workspace root should exist"))?
        .join("schemas");
    std::fs::create_dir_all(&output_dir)?;

    write_schema(
        output_dir.join("valid-tree.schema.json"),
        &schema_for!(ValidTreeStore),
    )?;
    write_schema(
        output_dir.join("project.schema.json"),
        &schema_for!(StateStore),
    )?;

    Ok(())
}

fn write_schema(path: PathBuf, schema: &schemars::Schema) -> Result<()> {
    let content = serde_json::to_string_pretty(schema)?;
    std::fs::write(path, content)?;
    Ok(())
}
