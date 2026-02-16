use beetry_editor_backend::{EditorService, NodeSpecMap};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct Backend {
    service: CopyValue<EditorService>,
}

impl std::ops::Deref for Backend {
    type Target = CopyValue<EditorService>;
    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

impl std::ops::DerefMut for Backend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.service
    }
}

impl Backend {
    pub(crate) fn new(node_specs: NodeSpecMap) -> Self {
        Self {
            service: CopyValue::new(EditorService::new(node_specs)),
        }
    }
}
