use beetry_editor_backend::{api, ui::PortConnectionUiQuery};
use beetry_editor_types::{
    id::PortConnectionId,
    output::ui::{PortConnectionUiData, VisibilityKind},
};
use dioxus::prelude::*;

use crate::{
    Backend, Point,
    ui::{
        error::ErrorQueueState, handler::define_handlers, node::port::ConnectionOrigin,
        style::curve::Curve,
    },
};

define_handlers!(
    on_menu: (PortConnectionId, Point)
);

impl Handlers {
    pub(crate) fn on_menu_handler(&self) -> EventHandler<(PortConnectionId, Point)> {
        self.on_menu
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct EdgeProps {
    pub start: Point,
    pub end: Point,
    pub port_center: Point,
    pub conn: PortConnectionId,
    pub stroke: &'static str,
    pub origin: ConnectionOrigin,
    pub port_width: f64,
}

#[derive(Clone, Copy)]
pub struct ConnectionUiState {
    signal: Signal<PortConnectionUiData>,
}

impl ConnectionUiState {
    fn new(initial: PortConnectionUiData) -> Self {
        Self {
            signal: Signal::new(initial),
        }
    }

    fn set(
        &mut self,
        state: PortConnectionUiData,
        mut backend: Backend,
        conn: PortConnectionId,
    ) -> Result<()> {
        backend.with_mut(|s| api::ui::port::update_data(s, conn, state.clone()))?;
        self.signal.set(state);
        Ok(())
    }
}

#[component]
pub fn Edge(props: EdgeProps) -> Element {
    let handlers = use_context::<Handlers>();
    let backend = use_context::<Backend>();
    let mut error_queue = use_context::<ErrorQueueState>();
    let on_menu = handlers.on_menu_handler();
    let mut state = use_hook(|| {
        let initial = backend.with(|s| {
            let query = api::ui::port::query(s);
            query
                .data(props.conn)
                .cloned()
                .unwrap_or_else(|_| PortConnectionUiData::new(VisibilityKind::Visible))
        });
        ConnectionUiState::new(initial)
    });

    rsx! {
        if matches!(state.signal.read().visibility(), VisibilityKind::Visible) {
            path {
                d: "{Curve::calculate_horizontal(&props.start, &props.end)}",
                stroke: props.stroke,
                stroke_width: "3",
                fill: "none",
                stroke_linecap: "round",
                cursor: "pointer",
                onclick: move |evt| {
                    evt.stop_propagation();
                    if let Err(e) = state
                        .set(PortConnectionUiData::new(VisibilityKind::Hidden), backend, props.conn)
                    {
                        error_queue.push(e);
                    }
                },
                oncontextmenu: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    let click_point = Point {
                        x: evt.element_coordinates().x,
                        y: evt.element_coordinates().y,
                    };
                    on_menu.call((props.conn, click_point));
                },
            }
        } else {
            RevealEdgeToggle {
                port_center: props.port_center,
                stroke: props.stroke,
                origin: props.origin,
                port_width: props.port_width,
                on_click: move |()| {
                    if let Err(e) = state
                        .set(PortConnectionUiData::new(VisibilityKind::Visible), backend, props.conn)
                    {
                        error_queue.push(e);
                    }
                },
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct RevealEdgeToggleProps {
    port_center: Point,
    port_width: f64,
    stroke: &'static str,
    origin: ConnectionOrigin,
    on_click: EventHandler<()>,
}

#[component]
fn RevealEdgeToggle(props: RevealEdgeToggleProps) -> Element {
    const PLUS_SIZE: f64 = 20.0;
    const PLUS_RADIUS: f64 = PLUS_SIZE / 2.0;
    let center = Point {
        x: match props.origin {
            ConnectionOrigin::Receiver => props.port_center.x - props.port_width / 2.0,
            ConnectionOrigin::Sender => props.port_center.x + props.port_width / 2.0,
        },
        y: props.port_center.y,
    };

    rsx! {
        g {
            cursor: "pointer",
            onclick: move |evt| {
                evt.stop_propagation();
                props.on_click.call(());
            },
            circle {
                cx: "{center.x}",
                cy: "{center.y}",
                r: "{PLUS_RADIUS}",
                fill: "#1f2937",
                stroke: props.stroke,
                stroke_width: "2",
            }
            text {
                x: "{center.x}",
                y: "{center.y + 4.0}",
                text_anchor: "middle",
                fill: "#f9fafb",
                font_size: "14",
                font_weight: "700",
                pointer_events: "none",
                "+"
            }
        }
    }
}
