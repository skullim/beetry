mod export;
mod import;

pub(crate) use export::{Export, Handlers as ExportHandlers};
pub(crate) use import::{Handlers as ImportHandlers, Import};

use dioxus::prelude::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) enum OperationStatus {
    #[default]
    None,
    Success,
    Error,
}

#[derive(Debug, Default, Clone, PartialEq, Props)]
pub(crate) struct OperationResult {
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
