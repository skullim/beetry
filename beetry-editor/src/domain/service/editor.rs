use crate::domain::{
    models::ChannelPosition,
    ports::{
        ChannelRepositoryConcept, EdgeRepositoryConcept, EditorRepository,
        NodeKindRepositoryConcept, ParamRepositoryConcept,
    },
    service::node::{NodeService, NodeServiceApi},
};
use anyhow::Result;
use beetry_serde::{de::channel::ChannelId, ser::channel::ChannelSpec};

pub struct EditorService<ER, KR, CR, PR> {
    node_service: NodeService,
    repo: EditorRepository<ER, KR, CR, PR>,
}

impl<ER, KR, CR, PR> EditorService<ER, KR, CR, PR>
where
    ER: EdgeRepositoryConcept,
    KR: NodeKindRepositoryConcept,
    CR: ChannelRepositoryConcept,
    PR: ParamRepositoryConcept,
{
    pub fn new(repo: EditorRepository<ER, KR, CR, PR>) -> Self {
        Self {
            node_service: NodeService::new(),
            repo,
        }
    }

    pub fn node(&mut self) -> NodeServiceApi<'_, '_, ER, KR, CR, PR> {
        NodeServiceApi::new(&mut self.repo, &mut self.node_service)
    }

    pub fn create_channel(&mut self, id: ChannelId, spec: ChannelSpec) -> Result<ChannelId> {
        todo!();
    }

    pub fn update_channel_position(
        &mut self,
        id: ChannelId,
        position: ChannelPosition,
    ) -> Result<()> {
        todo!()
    }
}
