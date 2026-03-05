pub use crate::{
    channel::ChannelQueryView,
    edge::EdgeQueryView,
    node::{
        NodeTrackerQuery, ParameterValueParser, ParameterValueQuery, SpecByNodeIdQuery,
        SpecBySpecIdQuery,
    },
    ui::{ChannelUiQuery, NodeUiQuery, NodeUiQueryProcessor},
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
        NodeLifecycleApi::create(api, spec, ui_data)
    }

    pub fn remove(api: &mut impl NodeLifecycleApi, id: NodeId) -> Result<()> {
        NodeLifecycleApi::remove(api, id)
    }

    pub mod spec {
        use crate::{
            api::contract::NodeApi,
            node::{SpecByNodeIdQuery, SpecBySpecIdQuery},
        };

        pub fn by_spec_id(api: &impl NodeApi) -> impl SpecBySpecIdQuery {
            NodeApi::spec_by_spec_id(api)
        }

        pub fn by_node_id(api: &impl NodeApi) -> impl SpecByNodeIdQuery {
            NodeApi::spec_by_node_id(api)
        }
    }

    pub mod tracker {
        use crate::api::contract::NodeApi;
        use crate::service::node::NodeTrackerQuery;

        pub fn query(api: &impl NodeApi) -> impl NodeTrackerQuery {
            NodeApi::tracker(api)
        }
    }

    pub mod parameters {
        use crate::api::{ParameterValueQuery, contract::NodeApi};
        use crate::service::node::ParameterValueMut;
        use beetry_editor_types::{id::NodeId, output::node::Parameters};

        pub fn create(api: &mut impl NodeApi, id: NodeId, params: Parameters) {
            NodeApi::parameters_mut(api).create(id, params);
        }

        pub fn query(api: &impl NodeApi) -> impl ParameterValueQuery {
            api.parameters()
        }
    }

    pub mod ports {
        use crate::{
            api::contract::{PortApi, PortLifecycleApi},
            node::{PortConnectionQuery, PortStateQuery},
        };
        use anyhow::Result;
        use beetry_editor_types::{
            id::{NodeId, NodePortId, PortConnectionId},
            output::{node::PortState, ui::PortConnectionUiData},
            spec::node::NodePortKind,
        };
        pub type RowIndex = usize;

        pub fn connect(
            api: &mut impl PortLifecycleApi,
            conn_id: PortConnectionId,
            ui_data: PortConnectionUiData,
        ) -> Result<()> {
            PortLifecycleApi::connect_port(api, conn_id, ui_data)
        }

        pub fn disconnect(
            api: &mut impl PortLifecycleApi,
            conn_id: PortConnectionId,
        ) -> Result<()> {
            PortLifecycleApi::disconnect(api, conn_id)
        }

        pub fn set_state(
            api: &mut impl PortApi,
            node_id: NodeId,
            port_id: NodePortId,
            state: PortState,
        ) -> Result<()> {
            api.set_state(node_id, port_id, state)
        }

        pub fn state_query(api: &impl PortApi) -> impl PortStateQuery {
            api.port_state_query()
        }

        pub fn connections_query(api: &impl PortApi) -> impl PortConnectionQuery {
            api.connections_query()
        }

        pub fn port_order(
            api: &impl PortApi,
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

    pub fn query(api: &impl EdgeApi) -> impl EdgeQueryView {
        EdgeApi::query(api)
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
        ChannelLifecycleApi::create(api, spec, input, ui_data)
    }

    pub fn remove(api: &mut impl ChannelLifecycleApi, id: ChannelId) -> Result<()> {
        ChannelLifecycleApi::remove(api, id)
    }

    pub fn update_config(
        api: &mut impl ChannelApi,
        id: ChannelId,
        update: ChannelConfigUpdate,
    ) -> Result<()> {
        ChannelApi::update_config(api, id, update)
    }

    pub fn query(api: &impl ChannelApi) -> impl ChannelQueryView {
        ChannelApi::query(api)
    }
}

pub mod ui {
    pub mod node {
        use crate::{api::contract::NodeUiApi, ui::NodeUiQuery};
        use anyhow::Result;
        use beetry_editor_types::{id::NodeId, output::ui::Point};

        pub fn update_position(
            api: &mut impl NodeUiApi,
            id: NodeId,
            position: Point,
        ) -> Result<()> {
            NodeUiApi::update_position(api, id, position)
        }

        pub fn query(api: &impl NodeUiApi) -> impl NodeUiQuery {
            NodeUiApi::query(api)
        }
    }

    pub mod channel {
        use crate::{api::contract::ChannelUiApi, ui::ChannelUiQuery};
        use anyhow::Result;
        use beetry_editor_types::{id::ChannelId, output::ui::Point};

        pub fn update_position(
            api: &mut impl ChannelUiApi,
            id: ChannelId,
            position: Point,
        ) -> Result<()> {
            ChannelUiApi::update_position(api, id, position)
        }

        pub fn query(api: &impl ChannelUiApi) -> impl ChannelUiQuery {
            ChannelUiApi::query(api)
        }
    }

    pub mod port {
        use crate::{api::contract::PortConnectionUiApi, ui::PortConnectionUiQuery};
        use anyhow::Result;
        use beetry_editor_types::{id::PortConnectionId, output::ui::PortConnectionUiData};

        pub fn update_data(
            api: &mut impl PortConnectionUiApi,
            id: PortConnectionId,
            data: PortConnectionUiData,
        ) -> Result<()> {
            PortConnectionUiApi::update_data(api, id, data)
        }

        pub fn query(api: &impl PortConnectionUiApi) -> impl PortConnectionUiQuery {
            PortConnectionUiApi::query(api)
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
