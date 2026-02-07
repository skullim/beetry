pub mod node {
    use crate::editor::NodeLifecycleApi;
    use anyhow::Result;
    use beetry_editor_types::{id::NodeId, output::ui::NodeUiData, spec::node::NodeSpec};

    pub fn create(
        api: &mut impl NodeLifecycleApi,
        spec: &NodeSpec,
        ui_data: NodeUiData,
    ) -> Result<NodeId> {
        api.create_with_ui(spec, ui_data)
    }

    pub fn remove(api: &mut impl NodeLifecycleApi, id: NodeId) -> Result<()> {
        api.remove_with_ui(id)
    }

    pub mod spec {
        use crate::{
            editor::NodeApi,
            node::{SpecByNodeIdQueryApi, SpecBySpecIdQueryApi},
        };

        pub fn by_spec_id(api: &impl NodeApi) -> impl SpecBySpecIdQueryApi {
            NodeApi::spec_by_spec_id(api)
        }

        pub fn by_node_id(api: &impl NodeApi) -> impl SpecByNodeIdQueryApi {
            NodeApi::spec_by_node_id(api)
        }
    }

    pub mod tracker {
        use crate::domain::service::node::NodeTrackerQueryApi;
        use crate::editor::NodeApi;

        pub fn borrow(api: &impl NodeApi) -> impl NodeTrackerQueryApi {
            NodeApi::tracker(api)
        }
    }

    pub mod parameters {
        use crate::domain::service::node::ParameterValueMutApi;
        use crate::editor::NodeApi;
        use beetry_editor_types::{id::NodeId, output::node::Parameters};

        pub fn set(api: &mut impl NodeApi, id: NodeId, params: Parameters) {
            NodeApi::parameters_mut(api).create(id, params);
        }
    }

    pub mod ports {
        use crate::editor::NodePortApi;
        use crate::node::PortConnectionView;
        use anyhow::Result;
        use beetry_editor_types::{
            id::{ChannelId, NodeId, NodePortId},
            spec::node::NodePortKind,
        };

        pub fn is_external(
            api: &impl NodePortApi,
            node_id: NodeId,
            port_id: NodePortId,
        ) -> Result<bool> {
            api.is_external(node_id, port_id)
        }

        pub fn connect(
            api: &mut impl NodePortApi,
            node_id: NodeId,
            port_id: NodePortId,
            channel_id: ChannelId,
        ) -> Result<()> {
            api.connect_port(node_id, port_id, channel_id)
        }

        pub fn disconnect(
            api: &mut impl NodePortApi,
            node_id: NodeId,
            port_id: NodePortId,
            channel_id: ChannelId,
        ) -> Result<()> {
            api.disconnect_port_connection(node_id, port_id, channel_id)
        }

        pub fn set_external(
            api: &mut impl NodePortApi,
            node_id: NodeId,
            port_id: NodePortId,
        ) -> Result<()> {
            api.set_port_external(node_id, port_id)
        }

        pub fn set_internal(
            api: &mut impl NodePortApi,
            node_id: NodeId,
            port_id: NodePortId,
        ) -> Result<()> {
            api.set_port_internal(node_id, port_id)
        }

        pub fn internal_connections(
            api: &impl NodePortApi,
        ) -> impl Iterator<Item = (&NodeId, &NodePortId, &ChannelId)> {
            api.internal_connections()
        }

        pub fn connection_views(
            api: &impl NodePortApi,
        ) -> impl Iterator<Item = Result<PortConnectionView<'_>>> {
            api.connection_views()
        }

        pub fn connection_views_by_kind(
            api: &impl NodePortApi,
            kind: NodePortKind,
        ) -> impl Iterator<Item = Result<PortConnectionView<'_>>> {
            api.connection_views_by_kind(kind)
        }
    }
}

pub mod edge {
    use crate::{edge::EdgeQueryApi, editor::EdgeApi};
    use anyhow::Result;
    use beetry_editor_types::{id::EdgeId, output::edge::NodeEdge};

    pub fn create(api: &mut impl EdgeApi, edge: NodeEdge) -> Result<EdgeId> {
        EdgeApi::create(api, edge)
    }

    pub fn remove(api: &mut impl EdgeApi, id: EdgeId) -> Result<()> {
        EdgeApi::remove(api, id)
    }

    pub fn borrow(api: &impl EdgeApi) -> impl EdgeQueryApi {
        EdgeApi::borrow(api)
    }
}

pub mod channel {
    use crate::{
        channel::ChannelQueryApi,
        editor::{ChannelApi, ChannelLifecycleApi},
    };
    use anyhow::Result;
    use beetry_editor_types::{
        id::ChannelId,
        output::{channel::ChannelConfig, ui::ChannelUiData},
        spec::channel::ChannelSpec,
    };

    pub fn create(
        api: &mut impl ChannelLifecycleApi,
        spec: &ChannelSpec,
        config: ChannelConfig,
        ui_data: ChannelUiData,
    ) -> Result<ChannelId> {
        api.create_with_ui(spec, config, ui_data)
    }

    pub fn remove(api: &mut impl ChannelLifecycleApi, id: ChannelId) -> Result<()> {
        api.remove_with_ui(id)
    }

    pub fn borrow(api: &impl ChannelApi) -> impl ChannelQueryApi {
        ChannelApi::borrow(api)
    }
}

pub mod ui {
    pub mod node {
        use crate::{editor::NodeUiApi, ui::NodeUiQueryApi};
        use anyhow::Result;
        use beetry_editor_types::{id::NodeId, output::ui::Point};

        pub fn update_position(
            api: &mut impl NodeUiApi,
            id: NodeId,
            position: Point,
        ) -> Result<()> {
            NodeUiApi::update_position(api, id, position)
        }

        pub fn borrow(api: &impl NodeUiApi) -> impl NodeUiQueryApi {
            NodeUiApi::borrow(api)
        }
    }

    pub mod channel {
        use crate::{editor::ChannelUiApi, ui::ChannelUiQueryApi};
        use anyhow::Result;
        use beetry_editor_types::{id::ChannelId, output::ui::Point};

        pub fn update_position(
            api: &mut impl ChannelUiApi,
            id: ChannelId,
            position: Point,
        ) -> Result<()> {
            ChannelUiApi::update_position(api, id, position)
        }

        pub fn borrow(api: &impl ChannelUiApi) -> impl ChannelUiQueryApi {
            ChannelUiApi::borrow(api)
        }
    }
}

pub mod project {
    use crate::editor::{ExportApi, ImportApi};
    use anyhow::Result;
    use beetry_editor_types::persistence::{EditorStateStore, ValidTree};

    pub fn import(api: &mut impl ImportApi, store: EditorStateStore) -> Result<()> {
        api.import_project(store)
    }

    pub fn export(api: &impl ExportApi) -> Result<EditorStateStore> {
        api.export_project()
    }

    pub fn export_valid_tree(api: &impl ExportApi) -> Result<ValidTree> {
        api.export_valid_tree()
    }
}
