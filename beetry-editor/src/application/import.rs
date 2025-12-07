use crate::{EditorService, domain::persistence::EditorData};
use anyhow::Result;

struct ProjectImporter;

impl ProjectImporter {
    fn import<NRF, ER, CR>(data: EditorData) -> Result<EditorService<NRF, ER, CR>> {
        todo!()
    }
}

//@todo implement in next version
struct SubtreeImporter;
