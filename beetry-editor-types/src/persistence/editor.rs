use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::persistence::{tree::MaybeValid, ui::Store};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StateStore {
    pub tree: MaybeValid,
    pub ui_elements: Store,
}
