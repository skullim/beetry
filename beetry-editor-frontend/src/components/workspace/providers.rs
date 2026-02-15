pub(crate) mod channel;
pub(crate) mod edge;
pub(crate) mod node;
pub(crate) mod port;
pub(crate) mod render_request;

use crate::components::workspace::state;
use crate::editor::Backend;
use crate::signals::RenderRequests;
use beetry_editor_types::{
    id::{ChannelId, NodeId},
    output::ui::Point,
};
use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;
use std::ops::{Deref, DerefMut};

use crate::ui::{
    channel as ui_channel, edge as ui_edge,
    node::{self as ui_node, port_context_menu},
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DimensionState {
    width: Signal<f64>,
    height: Signal<f64>,
}

impl DimensionState {
    const DEFAULT_SIZE: f64 = 1600.0;
    const MARGIN: f64 = 400.0;

    fn new() -> Self {
        Self {
            width: Signal::new(Self::DEFAULT_SIZE),
            height: Signal::new(Self::DEFAULT_SIZE),
        }
    }

    pub(crate) fn width(&self) -> f64 {
        *self.width.read()
    }

    pub(crate) fn height(&self) -> f64 {
        *self.height.read()
    }

    pub(crate) fn resize_if_needed<'a>(&mut self, positions: impl Iterator<Item = &'a Point>) {
        let (new_width, new_height) = positions
            .map(|node_pos| (node_pos.x + Self::MARGIN, node_pos.y + Self::MARGIN))
            .fold(
                (Self::DEFAULT_SIZE, Self::DEFAULT_SIZE),
                |(width, height), (x, y)| (width.max(x), height.max(y)),
            );

        self.width.set(new_width);
        self.height.set(new_height);
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ZoomState(Signal<f64>);

impl ZoomState {
    fn new() -> Self {
        Self(Signal::new(1.0))
    }

    pub(crate) fn get(&self) -> f64 {
        *self.0.peek()
    }

    pub(crate) fn update(&mut self, wheel_delta: &WheelDelta) {
        let zoom_factor = match wheel_delta {
            WheelDelta::Pixels(vector) => {
                if vector.y > 0.0 {
                    0.95
                } else {
                    1.05
                }
            }
            WheelDelta::Lines(vector) => {
                if vector.y > 0.0 {
                    0.9
                } else {
                    1.1
                }
            }
            WheelDelta::Pages(vector) => {
                if vector.y > 0.0 {
                    0.8
                } else {
                    1.25
                }
            }
        };

        self.0
            .with_mut(|zoom| *zoom = (*zoom * zoom_factor).clamp(0.1, 10.0));
    }
}

impl Deref for ZoomState {
    type Target = Signal<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZoomState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DragNodeState {
    Idle,
    Dragged { id: NodeId, offset: Point },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DragChannelState {
    Idle,
    Dragged { id: ChannelId, offset: Point },
}

#[derive(Clone)]
pub(crate) struct WorkspaceCtx {
    pub(crate) state: WorkspaceState,
    pub(crate) backend: Backend,
    pub(crate) requests: RenderRequests,
    pub(crate) workspace_handlers: render_request::WorkspaceEventHandlers,
}

#[derive(Clone, Copy)]
pub(crate) struct WorkspaceState {
    pub(crate) drag: DragState,
    pub(crate) menus: MenuState,
    pub(crate) svg: WorkspaceSvgState,
    pub(crate) temp: TempState,
}

#[derive(Clone, Copy)]
pub(crate) struct DragState {
    pub(crate) node: Signal<DragNodeState>,
    pub(crate) channel: Signal<DragChannelState>,
}

#[derive(Clone, Copy)]
pub(crate) struct MenuState {
    pub(crate) node: Signal<ui_node::ContextMenuState>,
    pub(crate) edge: Signal<ui_edge::ContextMenuState>,
    pub(crate) channel: Signal<ui_channel::ContextMenuState>,
    pub(crate) port: Signal<port_context_menu::State>,
}

#[derive(Clone, Copy)]
pub(crate) struct WorkspaceSvgState {
    pub(crate) dimensions: DimensionState,
    pub(crate) zoom: ZoomState,
}

#[derive(Clone, Copy)]
pub(crate) struct TempState {
    pub(crate) edge: Signal<state::temporary::State>,
    pub(crate) channel: ui_channel::temporary::State,
}

impl WorkspaceCtx {
    pub(crate) fn new(backend: Backend, requests: RenderRequests) -> Self {
        let state = WorkspaceState {
            drag: DragState {
                node: Signal::new(DragNodeState::Idle),
                channel: Signal::new(DragChannelState::Idle),
            },
            menus: MenuState {
                node: Signal::new(ui_node::ContextMenuState::default()),
                edge: Signal::new(ui_edge::ContextMenuState::default()),
                channel: Signal::new(ui_channel::ContextMenuState::default()),
                port: Signal::new(port_context_menu::State::default()),
            },
            svg: WorkspaceSvgState {
                dimensions: DimensionState::new(),
                zoom: ZoomState::new(),
            },
            temp: TempState {
                edge: Signal::new(state::temporary::State::new()), //ui_edge::temporary::State::new(),
                channel: ui_channel::temporary::State::new(),
            },
        };

        let workspace_handlers = render_request::workspace_event_handlers(
            state.drag,
            state.menus,
            state.svg,
            state.temp,
            backend,
            requests,
        );
        Self {
            state,
            backend,
            requests,
            workspace_handlers,
        }
    }
}
