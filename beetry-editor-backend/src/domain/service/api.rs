pub mod node {
    use crate::editor::{EditorServiceApi, NodeExtApi, NodeUiExtApi};
    use anyhow::Result;
    use beetry_editor_types::{id::NodeId, output::ui::NodeUiData, spec::node::NodeSpec};

    pub fn create(
        api: &mut impl EditorServiceApi,
        spec: &NodeSpec,
        ui_data: NodeUiData,
    ) -> Result<NodeId> {
        let id = NodeExtApi::create(api, spec)?;
        if NodeUiExtApi::create(api, id, ui_data).is_err() {
            NodeExtApi::remove(api, id)?;
        }
        Ok(id)
    }

    pub fn remove(api: &mut impl EditorServiceApi, id: NodeId) -> Result<()> {
        NodeExtApi::remove(api, id)?;
        NodeUiExtApi::remove(api, id);
        Ok(())
    }

    pub mod spec {
        use crate::{
            editor::{EditorServiceApi, NodeExtApi},
            node::{SpecByNodeIdQueryApi, SpecBySpecIdQueryApi},
        };

        pub fn by_spec_id(api: &impl EditorServiceApi) -> impl SpecBySpecIdQueryApi {
            NodeExtApi::spec_by_spec_id(api)
        }

        pub fn by_node_id(api: &impl EditorServiceApi) -> impl SpecByNodeIdQueryApi {
            NodeExtApi::spec_by_node_id(api)
        }
    }
}

pub mod edge {
    use crate::{
        edge::EdgeQueryApi,
        editor::{EdgeExtApi, EditorServiceApi},
    };
    use anyhow::Result;
    use beetry_editor_types::{id::EdgeId, output::edge::NodeEdge};

    pub fn create(api: &mut impl EditorServiceApi, edge: NodeEdge) -> Result<EdgeId> {
        EdgeExtApi::create(api, edge)
    }

    pub fn remove(api: &mut impl EditorServiceApi, id: EdgeId) -> Result<()> {
        EdgeExtApi::remove(api, id)
    }

    pub fn borrow(api: &impl EditorServiceApi) -> impl EdgeQueryApi {
        EdgeExtApi::borrow(api)
    }
}

pub mod channel {
    use crate::{
        channel::ChannelQueryApi,
        editor::{ChannelExtApi, ChannelUiExtApi, EditorServiceApi},
    };
    use anyhow::Result;
    use beetry_editor_types::{
        id::ChannelId,
        output::{channel::ChannelConfig, ui::ChannelUiData},
        spec::channel::ChannelSpec,
    };

    pub fn create(
        api: &mut impl EditorServiceApi,
        spec: &ChannelSpec,
        config: ChannelConfig,
        ui_data: ChannelUiData,
    ) -> Result<ChannelId> {
        let id = ChannelExtApi::create(api, spec, config)?;
        if ChannelUiExtApi::create(api, id, ui_data).is_err() {
            ChannelExtApi::remove(api, id);
        }
        Ok(id)
    }

    pub fn remove(api: &mut impl EditorServiceApi, id: ChannelId) -> Result<()> {
        ChannelExtApi::remove(api, id);
        ChannelUiExtApi::remove(api, id);
        Ok(())
    }

    pub fn borrow(api: &impl EditorServiceApi) -> impl ChannelQueryApi {
        ChannelExtApi::borrow(api)
    }
}

pub mod ui {
    pub mod node {
        use crate::{
            editor::{EditorServiceApi, NodeUiExtApi},
            ui::NodeUiQueryApi,
        };
        use anyhow::Result;
        use beetry_editor_types::{id::NodeId, output::ui::Point};

        pub fn update_position(
            api: &mut impl EditorServiceApi,
            id: NodeId,
            position: Point,
        ) -> Result<()> {
            NodeUiExtApi::update_position(api, id, position)
        }

        pub fn borrow(api: &impl EditorServiceApi) -> impl NodeUiQueryApi {
            NodeUiExtApi::borrow(api)
        }
    }

    pub mod channel {
        use crate::{
            editor::{ChannelUiExtApi, EditorServiceApi},
            ui::ChannelUiQueryApi,
        };
        use anyhow::Result;
        use beetry_editor_types::{id::ChannelId, output::ui::Point};

        pub fn update_position(
            api: &mut impl EditorServiceApi,
            id: ChannelId,
            position: Point,
        ) -> Result<()> {
            ChannelUiExtApi::update_position(api, id, position)
        }

        pub fn borrow(api: &impl EditorServiceApi) -> impl ChannelUiQueryApi {
            ChannelUiExtApi::borrow(api)
        }
    }
}
