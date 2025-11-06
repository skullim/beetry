mod export;
mod import;

pub use export::{Export, Handlers as ExportHandlers};
pub use import::{Handlers as ImportHandlers, Import};

use dioxus::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum OperationStatus {
    #[default]
    None,
    Success,
    Error,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Props)]
pub struct OperationResult {
    message: String,
    status: OperationStatus,
}

impl OperationResult {
    pub(crate) fn new(message: impl Into<String>, status: OperationStatus) -> Self {
        Self {
            message: message.into(),
            status,
        }
    }
}
