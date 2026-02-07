pub mod node {
    use crate::editor::NodeOpsApi;
    use anyhow::Result;
    use beetry_editor_types::{id::NodeId, output::ui::NodeUiData, spec::node::NodeSpec};

    pub fn create(
        api: &mut impl NodeOpsApi,
        spec: &NodeSpec,
        ui_data: NodeUiData,
    ) -> Result<NodeId> {
        api.create_with_ui(spec, ui_data)
    }

    pub fn remove(api: &mut impl NodeOpsApi, id: NodeId) -> Result<()> {
        api.remove_with_ui(id)
    }

    pub mod spec {
        use crate::{
            editor::NodeExtApi,
            node::{SpecByNodeIdQueryApi, SpecBySpecIdQueryApi},
        };

        pub fn by_spec_id(api: &impl NodeExtApi) -> impl SpecBySpecIdQueryApi {
            NodeExtApi::spec_by_spec_id(api)
        }

        pub fn by_node_id(api: &impl NodeExtApi) -> impl SpecByNodeIdQueryApi {
            NodeExtApi::spec_by_node_id(api)
        }
    }

    pub mod tracker {
        use crate::domain::service::node::NodeTrackerQueryApi;
        use crate::editor::NodeExtApi;

        pub fn borrow(api: &impl NodeExtApi) -> impl NodeTrackerQueryApi {
            NodeExtApi::tracker(api)
        }
    }

    pub mod parameters {
        use crate::domain::service::node::ParameterValueMutApi;
        use crate::editor::NodeExtApi;
        use beetry_editor_types::{id::NodeId, output::node::Parameters};

        pub fn set(api: &mut impl NodeExtApi, id: NodeId, params: Parameters) {
            NodeExtApi::parameters_mut(api).create(id, params);
        }
    }

    pub mod ports {
        use crate::editor::NodePortOpsApi;
        use crate::node::PortConnectionView;
        use anyhow::Result;
        use beetry_editor_types::{
            id::{ChannelId, NodeId, NodePortId},
            spec::node::NodePortKind,
        };

        pub fn is_external(
            api: &impl NodePortOpsApi,
            node_id: NodeId,
            port_id: NodePortId,
        ) -> Result<bool> {
            api.is_external(node_id, port_id)
        }

        pub fn connect(
            api: &mut impl NodePortOpsApi,
            node_id: NodeId,
            port_id: NodePortId,
            channel_id: ChannelId,
        ) -> Result<()> {
            api.connect_port(node_id, port_id, channel_id)
        }

        pub fn disconnect(
            api: &mut impl NodePortOpsApi,
            node_id: NodeId,
            port_id: NodePortId,
            channel_id: ChannelId,
        ) -> Result<()> {
            api.disconnect_port_connection(node_id, port_id, channel_id)
        }

        pub fn set_external(
            api: &mut impl NodePortOpsApi,
            node_id: NodeId,
            port_id: NodePortId,
        ) -> Result<()> {
            api.set_port_external(node_id, port_id)
        }

        pub fn set_internal(
            api: &mut impl NodePortOpsApi,
            node_id: NodeId,
            port_id: NodePortId,
        ) -> Result<()> {
            api.set_port_internal(node_id, port_id)
        }

        pub fn internal_connections(
            api: &impl NodePortOpsApi,
        ) -> impl Iterator<Item = (&NodeId, &NodePortId, &ChannelId)> {
            api.internal_connections()
        }

        pub fn connection_views(
            api: &impl NodePortOpsApi,
        ) -> impl Iterator<Item = Result<PortConnectionView<'_>>> {
            api.connection_views()
        }

        pub fn connection_views_by_kind(
            api: &impl NodePortOpsApi,
            kind: NodePortKind,
        ) -> impl Iterator<Item = Result<PortConnectionView<'_>>> {
            api.connection_views_by_kind(kind)
        }
    }
}

pub mod edge {
    use crate::{edge::EdgeQueryApi, editor::EdgeExtApi};
    use anyhow::Result;
    use beetry_editor_types::{id::EdgeId, output::edge::NodeEdge};

    pub fn create(api: &mut impl EdgeExtApi, edge: NodeEdge) -> Result<EdgeId> {
        EdgeExtApi::create(api, edge)
    }

    pub fn remove(api: &mut impl EdgeExtApi, id: EdgeId) -> Result<()> {
        EdgeExtApi::remove(api, id)
    }

    pub fn borrow(api: &impl EdgeExtApi) -> impl EdgeQueryApi {
        EdgeExtApi::borrow(api)
    }
}

pub mod channel {
    use crate::{
        channel::ChannelQueryApi,
        editor::{ChannelExtApi, ChannelOpsApi},
    };
    use anyhow::Result;
    use beetry_editor_types::{
        id::ChannelId,
        output::{channel::ChannelConfig, ui::ChannelUiData},
        spec::channel::ChannelSpec,
    };

    pub fn create(
        api: &mut impl ChannelOpsApi,
        spec: &ChannelSpec,
        config: ChannelConfig,
        ui_data: ChannelUiData,
    ) -> Result<ChannelId> {
        api.create_with_ui(spec, config, ui_data)
    }

    pub fn remove(api: &mut impl ChannelOpsApi, id: ChannelId) -> Result<()> {
        api.remove_with_ui(id)
    }

    pub fn borrow(api: &impl ChannelExtApi) -> impl ChannelQueryApi {
        ChannelExtApi::borrow(api)
    }
}

pub mod ui {
    pub mod node {
        use crate::{editor::NodeUiExtApi, ui::NodeUiQueryApi};
        use anyhow::Result;
        use beetry_editor_types::{id::NodeId, output::ui::Point};

        pub fn update_position(
            api: &mut impl NodeUiExtApi,
            id: NodeId,
            position: Point,
        ) -> Result<()> {
            NodeUiExtApi::update_position(api, id, position)
        }

        pub fn borrow(api: &impl NodeUiExtApi) -> impl NodeUiQueryApi {
            NodeUiExtApi::borrow(api)
        }
    }

    pub mod channel {
        use crate::{editor::ChannelUiExtApi, ui::ChannelUiQueryApi};
        use anyhow::Result;
        use beetry_editor_types::{id::ChannelId, output::ui::Point};

        pub fn update_position(
            api: &mut impl ChannelUiExtApi,
            id: ChannelId,
            position: Point,
        ) -> Result<()> {
            ChannelUiExtApi::update_position(api, id, position)
        }

        pub fn borrow(api: &impl ChannelUiExtApi) -> impl ChannelUiQueryApi {
            ChannelUiExtApi::borrow(api)
        }
    }
}
