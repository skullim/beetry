use serde::{Deserialize, Serialize};

use crate::persistence::{tree::MaybeValid, ui::Store};

#[derive(Debug, Serialize, Deserialize)]
pub struct StateStore {
    pub tree: MaybeValid,
    pub ui_elements: Store,
}
