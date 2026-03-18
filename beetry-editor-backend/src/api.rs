//! Public backend API for editor operations.
//!
//! This module provides a thin, task-oriented surface over the backend
//! services. Its submodules group operations by domain such as nodes, edges,
//! channels, UI state, and project import/export.

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
    use anyhow::Result;
    use beetry_editor_types::{id::NodeId, output::ui::NodeUiData, spec::node::NodeSpec};

    use crate::api::contract::NodeLifecycleApi;

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
            api::contract::NodeQueryApi,
            node::{SpecByNodeIdQuery, SpecBySpecIdQuery},
        };

        pub fn by_spec_id(api: &impl NodeQueryApi) -> impl SpecBySpecIdQuery {
            NodeQueryApi::spec_by_spec_id(api)
        }

        pub fn by_node_id(api: &impl NodeQueryApi) -> impl SpecByNodeIdQuery {
            NodeQueryApi::spec_by_node_id(api)
        }
    }

    pub mod tracker {
        use crate::{api::contract::NodeQueryApi, service::node::NodeTrackerQuery};

        pub fn query(api: &impl NodeQueryApi) -> impl NodeTrackerQuery {
            NodeQueryApi::tracker(api)
        }
    }

    pub mod parameters {
        use beetry_editor_types::{id::NodeId, output::node::Parameters};

        use crate::{
            api::{
                ParameterValueQuery,
                contract::{ParameterCommandApi, ParameterQueryApi},
            },
            service::node::ParameterValueMut,
        };

        pub fn create(api: &mut impl ParameterCommandApi, id: NodeId, params: Parameters) {
            ParameterCommandApi::parameters_mut(api).create(id, params);
        }

        pub fn query(api: &impl ParameterQueryApi) -> impl ParameterValueQuery {
            ParameterQueryApi::parameters(api)
        }
    }

    pub mod ports {
        use anyhow::Result;
        use beetry_editor_types::{
            id::{NodeId, NodePortId, PortConnectionId},
            output::{node::PortState, ui::PortConnectionUiData},
            spec::node::NodePortKind,
        };

        use crate::{
            api::contract::{PortCommandApi, PortLifecycleApi, PortQueryApi},
            node::{PortConnectionQuery, PortStateQuery},
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
            api: &mut impl PortCommandApi,
            node_id: NodeId,
            port_id: NodePortId,
            state: PortState,
        ) -> Result<()> {
            PortCommandApi::set_state(api, node_id, port_id, state)
        }

        pub fn state_query(api: &impl PortQueryApi) -> impl PortStateQuery {
            PortQueryApi::port_state_query(api)
        }

        pub fn connections_query(api: &impl PortQueryApi) -> impl PortConnectionQuery {
            PortQueryApi::connections_query(api)
        }

        pub fn order(
            api: &impl PortQueryApi,
            kind: NodePortKind,
            node_id: NodeId,
            port_id: NodePortId,
        ) -> Result<RowIndex> {
            PortQueryApi::port_order(api, kind, node_id, port_id)
        }
    }
}

pub mod edge {
    use anyhow::Result;
    use beetry_editor_types::{id::EdgeId, output::edge::NodeEdge};

    use crate::{
        api::contract::{EdgeCommandApi, EdgeQueryApi},
        edge::EdgeQueryView,
    };

    pub fn create(api: &mut impl EdgeCommandApi, edge: NodeEdge) -> Result<EdgeId> {
        EdgeCommandApi::create(api, edge)
    }

    pub fn remove(api: &mut impl EdgeCommandApi, id: EdgeId) -> Result<()> {
        EdgeCommandApi::remove(api, id)
    }

    pub fn query(api: &impl EdgeQueryApi) -> impl EdgeQueryView {
        EdgeQueryApi::query(api)
    }
}

pub mod channel {
    use anyhow::Result;
    use beetry_editor_types::{
        id::ChannelId,
        output::{
            channel::{ChannelConfigInput, ChannelConfigUpdate},
            ui::ChannelUiData,
        },
        spec::channel::ChannelSpec,
    };

    use crate::{
        api::contract::{ChannelCommandApi, ChannelLifecycleApi, ChannelQueryApi},
        channel::ChannelQueryView,
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
        api: &mut impl ChannelCommandApi,
        id: ChannelId,
        update: ChannelConfigUpdate,
    ) -> Result<()> {
        ChannelCommandApi::update_config(api, id, update)
    }

    pub fn query(api: &impl ChannelQueryApi) -> impl ChannelQueryView {
        ChannelQueryApi::query(api)
    }
}

pub mod ui {
    pub mod node {
        use anyhow::Result;
        use beetry_editor_types::{id::NodeId, output::ui::Point};

        use crate::{
            api::contract::{NodeUiCommandApi, NodeUiQueryApi},
            ui::NodeUiQuery,
        };

        pub fn update_position(
            api: &mut impl NodeUiCommandApi,
            id: NodeId,
            position: Point,
        ) -> Result<()> {
            NodeUiCommandApi::update_position(api, id, position)
        }

        pub fn query(api: &impl NodeUiQueryApi) -> impl NodeUiQuery {
            NodeUiQueryApi::query(api)
        }
    }

    pub mod channel {
        use anyhow::Result;
        use beetry_editor_types::{id::ChannelId, output::ui::Point};

        use crate::{
            api::contract::{ChannelUiCommandApi, ChannelUiQueryApi},
            ui::ChannelUiQuery,
        };

        pub fn update_position(
            api: &mut impl ChannelUiCommandApi,
            id: ChannelId,
            position: Point,
        ) -> Result<()> {
            ChannelUiCommandApi::update_position(api, id, position)
        }

        pub fn query(api: &impl ChannelUiQueryApi) -> impl ChannelUiQuery {
            ChannelUiQueryApi::query(api)
        }
    }

    pub mod port {
        use anyhow::Result;
        use beetry_editor_types::{id::PortConnectionId, output::ui::PortConnectionUiData};

        use crate::{
            api::contract::{PortConnectionUiCommandApi, PortConnectionUiQueryApi},
            ui::PortConnectionUiQuery,
        };

        pub fn update_data(
            api: &mut impl PortConnectionUiCommandApi,
            id: PortConnectionId,
            data: PortConnectionUiData,
        ) -> Result<()> {
            PortConnectionUiCommandApi::update_data(api, id, data)
        }

        pub fn query(api: &impl PortConnectionUiQueryApi) -> impl PortConnectionUiQuery {
            PortConnectionUiQueryApi::query(api)
        }
    }
}

pub mod project {
    use anyhow::Result;
    use beetry_editor_types::persistence::{editor, tree::ValidTreeStore};

    use crate::api::contract::{ExportApi, ImportApi};

    pub fn import(api: &mut impl ImportApi, store: editor::StateStore) -> Result<()> {
        api.import_project(store)
    }

    pub fn export(api: &impl ExportApi) -> Result<editor::StateStore> {
        api.export_project()
    }

    pub fn export_valid_tree(api: &impl ExportApi) -> Result<ValidTreeStore> {
        api.export_valid_tree()
    }
}
