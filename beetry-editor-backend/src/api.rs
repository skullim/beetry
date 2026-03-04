pub use crate::{
    channel::ChannelQueryView,
    edge::EdgeQueryView,
    node::{
        NodeTrackerQueryView, ParameterValueParser, ParameterValueQueryView, PortConnectionView,
        SpecByNodeIdQueryView, SpecBySpecIdQueryView,
    },
    ui::{ChannelUiQueryApi, NodeUiQueryApi, NodeUiQueryProcessor},
};

pub mod contract;

pub mod node {
    use crate::api::contract::NodeLifecycleApi;
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
            api::contract::NodeApi,
            node::{SpecByNodeIdQueryView, SpecBySpecIdQueryView},
        };

        pub fn by_spec_id(api: &impl NodeApi) -> impl SpecBySpecIdQueryView {
            NodeApi::spec_by_spec_id(api)
        }

        pub fn by_node_id(api: &impl NodeApi) -> impl SpecByNodeIdQueryView {
            NodeApi::spec_by_node_id(api)
        }
    }

    pub mod tracker {
        use crate::api::contract::NodeApi;
        use crate::service::node::NodeTrackerQueryView;

        pub fn query_view(api: &impl NodeApi) -> impl NodeTrackerQueryView {
            NodeApi::tracker(api)
        }
    }

    pub mod parameters {
        use crate::api::contract::NodeApi;
        use crate::service::node::ParameterValueMut;
        use beetry_editor_types::{id::NodeId, output::node::Parameters};

        pub fn create(api: &mut impl NodeApi, id: NodeId, params: Parameters) {
            NodeApi::parameters_mut(api).create(id, params);
        }

        pub fn get(api: &impl NodeApi, id: NodeId) -> anyhow::Result<&Parameters> {
            NodeApi::parameters_by_node_id(api, id)
        }
    }

    pub mod ports {
        use crate::{api::contract::NodePortApi, node::PortConnectionDataView};
        use anyhow::Result;
        use beetry_editor_types::{
            id::{ChannelId, NodeId, NodePortId},
            spec::node::NodePortKind,
        };
        pub type RowIndex = usize;

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
            api.disconnect_port(node_id, port_id, channel_id)
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
        ) -> impl Iterator<Item = Result<PortConnectionDataView<'_>>> {
            api.connection_views()
        }

        pub fn connection_views_by_kind(
            api: &impl NodePortApi,
            kind: NodePortKind,
        ) -> impl Iterator<Item = Result<PortConnectionDataView<'_>>> {
            api.connection_views_by_kind(kind)
        }

        pub fn port_order(
            api: &impl NodePortApi,
            kind: NodePortKind,
            node_id: NodeId,
            port_id: NodePortId,
        ) -> Result<RowIndex> {
            api.port_order(kind, node_id, port_id)
        }
    }
}

pub mod edge {
    use crate::{api::contract::EdgeApi, edge::EdgeQueryView};
    use anyhow::Result;
    use beetry_editor_types::{id::EdgeId, output::edge::NodeEdge};

    pub fn create(api: &mut impl EdgeApi, edge: NodeEdge) -> Result<EdgeId> {
        EdgeApi::create(api, edge)
    }

    pub fn remove(api: &mut impl EdgeApi, id: EdgeId) -> Result<()> {
        EdgeApi::remove(api, id)
    }

    pub fn borrow(api: &impl EdgeApi) -> impl EdgeQueryView {
        EdgeApi::borrow(api)
    }
}

pub mod channel {
    use crate::{
        api::contract::{ChannelApi, ChannelLifecycleApi},
        channel::ChannelQueryView,
    };
    use anyhow::Result;
    use beetry_editor_types::{
        id::ChannelId,
        output::{
            channel::{ChannelConfigInput, ChannelConfigUpdate},
            ui::ChannelUiData,
        },
        spec::channel::ChannelSpec,
    };

    pub fn create(
        api: &mut impl ChannelLifecycleApi,
        spec: &ChannelSpec,
        input: ChannelConfigInput,
        ui_data: ChannelUiData,
    ) -> Result<ChannelId> {
        api.create_with_ui(spec, input, ui_data)
    }

    pub fn remove(api: &mut impl ChannelLifecycleApi, id: ChannelId) -> Result<()> {
        api.remove_with_ui(id)
    }

    pub fn update_config(
        api: &mut impl ChannelApi,
        id: ChannelId,
        update: ChannelConfigUpdate,
    ) -> Result<()> {
        ChannelApi::update_config(api, id, update)
    }

    pub fn borrow(api: &impl ChannelApi) -> impl ChannelQueryView {
        ChannelApi::borrow(api)
    }
}

pub mod ui {
    pub mod node {
        use crate::{api::contract::NodeUiApi, ui::NodeUiQueryApi};
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
        use crate::{api::contract::ChannelUiApi, ui::ChannelUiQueryApi};
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
    use crate::api::contract::{ExportApi, ImportApi};
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
