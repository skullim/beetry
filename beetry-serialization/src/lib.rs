//! Serialization related functionality.
//!
//! This crate is an internal Beetry implementation crate and is not considered
//! part of the public API. For public APIs, use the `beetry` crate.

use anyhow::Result;

pub mod json {
    use std::path::Path;

    use anyhow::Context;
    use serde::Serialize;

    use super::Result;

    fn serialize<T: Serialize>(value: &T) -> Result<String> {
        Ok(serde_json::to_string(value)?)
    }

    pub fn save_to_file<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<()> {
        let serialized = serialize(value)?;
        std::fs::write(&path, serialized)
            .with_context(|| format!("failed to save to file {}", path.as_ref().display()))
    }

    pub fn load_from<T>(raw: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_str(raw).context("failed to deserialize")
    }

    pub fn load_from_file<T>(path: impl AsRef<Path>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to load file {}", path.as_ref().display()))?;
        load_from(&raw)
    }
}
