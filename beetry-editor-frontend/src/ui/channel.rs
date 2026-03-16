pub mod dialog;
pub mod edge;
pub mod edge_menu;
pub mod menu;
pub mod renderer;

use std::rc::Rc;

use beetry_editor_backend::{api, api::ChannelQueryView};
use beetry_editor_types::id::ChannelId;
use beetry_plugin::Named;
use dioxus::prelude::*;
pub use menu::Menu;
pub use renderer::{ConnectionRenderer, Renderer};

use crate::{
    Backend, Point,
    signals::RenderRequests,
    ui::{
        error::ErrorQueueState,
        handler::define_handlers,
        style::{
            connection::{Fill, FillHover, Stroke},
            shadow,
            text::{self, text_width_from},
        },
        tooltip::TooltipCard,
    },
};

pub mod layout {
    use crate::Point;

    pub const WIDTH: f64 = 40.0;
    pub const HEIGHT: f64 = 25.0;
    pub const TOOLTIP_GAP: Point = Point { x: 8.0, y: 0.0 };
    pub const PORT_CENTER: Point = Point {
        x: WIDTH / 2.0,
        y: HEIGHT / 2.0,
    };
    pub const PORT_DIMENSIONS: Point = Point { x: 10.0, y: 10.0 };
}
const PORT_CIRCLE_INDICATOR_FILL: &str = "rgba(255,255,255,0.8)";

define_handlers!(receiver_on_mouse_up: ChannelId,
          sender_on_mouse_up: ChannelId,
          on_menu: (ChannelId, Event<MouseData>),
          on_mouse_down: (ChannelId, Point, Event<MouseData>)
);

#[derive(Props, PartialEq, Clone)]
pub struct ChannelProps {
    id: ChannelId,
    position: Point,
}

struct BodyData {
    name: String,
    kind_label: String,
    capacity_label: String,
}

fn query_connected(backend: &Backend, errors: &mut ErrorQueueState, id: ChannelId) -> (bool, bool) {
    backend.with(|s| {
        let query = api::channel::query(s);
        match query.config(id) {
            Ok(config) => (config.count().sender() > 0, config.count().receiver() > 0),
            Err(e) => {
                errors.push(e);
                (false, false)
            }
        }
    })
}

#[component]
pub(crate) fn Channel(props: ChannelProps) -> Element {
    let id = props.id;
    debug!("rendering channel {id}");
    let position = props.position;

    let backend = use_context::<Backend>();
    let backend_peek = backend.peek();
    let channel_query_api = api::channel::query(&(*backend_peek));

    let mut errors = use_context::<ErrorQueueState>();
    let Ok(spec) = channel_query_api.spec(id).map_err(|e| {
        errors.push(e);
    }) else {
        return rsx! {};
    };
    let mut is_connected = use_signal(|| query_connected(&backend, &mut errors, id));
    {
        let render_requests = use_context::<RenderRequests>();
        use_memo(move || {
            render_requests.channel_edges.track();
            is_connected.set(query_connected(&backend, &mut errors, id));
        });
    }

    let (is_sender_connected, is_receiver_connected) = is_connected();
    let body_width = text_width_from(spec.name(), text::FONT_SIZE_SMALL);
    let handlers = use_context::<Handlers>();

    rsx! {
        g {
            onmousedown: move |evt| {
                handlers.on_mouse_down.call((id, position, evt));
            },

            SenderBody {
                id,
                position,
                is_connected: is_sender_connected,
            }
            MainBody {
                id,
                position,
                body_width,
            }
            ReceiverBody {
                id,
                position,
                body_width,
                is_connected: is_receiver_connected,
            }
        }
    }
}

#[derive(Clone, Copy)]
struct VisualOptions {
    fill: &'static str,
    stroke: &'static str,
    filter: &'static str,
}

impl VisualOptions {
    fn sender(connected: bool, is_hovered: bool) -> Self {
        let fill = if connected {
            if is_hovered {
                FillHover::SENDER
            } else {
                Fill::SENDER
            }
        } else {
            Fill::UNCONNECTED
        };
        let stroke = if connected {
            Stroke::DEFAULT
        } else if is_hovered {
            Stroke::UNCONNECTED_HOVER
        } else {
            Stroke::UNCONNECTED_DEFAULT
        };

        Self {
            fill,
            stroke,
            filter: Self::filter(is_hovered),
        }
    }

    fn receiver(connected: bool, is_hovered: bool) -> Self {
        let fill = if connected {
            if is_hovered {
                FillHover::RECEIVER
            } else {
                Fill::RECEIVER
            }
        } else {
            Fill::UNCONNECTED
        };
        let stroke = if connected {
            Stroke::DEFAULT
        } else if is_hovered {
            Stroke::UNCONNECTED_HOVER
        } else {
            Stroke::UNCONNECTED_DEFAULT
        };

        Self {
            fill,
            stroke,
            filter: Self::filter(is_hovered),
        }
    }

    fn body(is_hovered: bool) -> Self {
        let fill = if is_hovered {
            FillHover::BODY
        } else {
            Fill::BODY
        };

        Self {
            fill,
            stroke: Stroke::DEFAULT,
            filter: Self::filter(is_hovered),
        }
    }

    fn filter(is_hovered: bool) -> &'static str {
        if is_hovered {
            shadow::FilterUrl::SHADOW_HOVER
        } else {
            shadow::FilterUrl::SHADOW
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SenderBodyProps {
    id: ChannelId,
    position: Point,
    is_connected: bool,
}

#[component]
fn SenderBody(props: SenderBodyProps) -> Element {
    let mut is_hovered = use_signal(|| false);
    let visuals = VisualOptions::sender(props.is_connected, is_hovered());
    let handlers = use_context::<Handlers>();
    let on_menu = handlers.on_menu;

    rsx! {
        g {
            rect {
                onmouseup: move |_| { handlers.sender_on_mouse_up.call(props.id) },
                oncontextmenu: move |evt| on_menu.call((props.id, evt)),
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
                x: "{props.position.x}",
                y: "{props.position.y}",
                width: "{layout::WIDTH}",
                height: "{layout::HEIGHT}",
                rx: "{layout::PORT_DIMENSIONS.x}",
                ry: "{layout::PORT_DIMENSIONS.y}",
                fill: visuals.fill,
                filter: visuals.filter,
                stroke: visuals.stroke,
                stroke_width: Stroke::WIDTH,
                style: "cursor: grab;",
            }
            circle {
                cx: "{props.position.x + layout::PORT_CENTER.x}",
                cy: "{props.position.y + layout::PORT_CENTER.y}",
                r: "3",
                fill: PORT_CIRCLE_INDICATOR_FILL,
                pointer_events: "none",
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct MainBodyProps {
    id: ChannelId,
    position: Point,
    body_width: f64,
}

#[component]
fn MainBody(props: MainBodyProps) -> Element {
    let backend = use_context::<Backend>();
    let data = use_hook(move || {
        backend.with(|s| {
            let query = api::channel::query(s);
            let on_unknown = || "unknown".to_string();

            let name = query
                .spec(props.id)
                .map_or_else(|_| on_unknown(), |spec| spec.name().to_string());
            let (kind_label, capacity_label) = query.config(props.id).map_or_else(
                |_| (on_unknown(), on_unknown()),
                |config| (config.kind().to_string(), config.capacity().to_string()),
            );

            Rc::new(BodyData {
                name,
                kind_label,
                capacity_label,
            })
        })
    });

    let handlers = use_context::<Handlers>();
    let mut is_hovered = use_signal(|| false);
    let visuals = VisualOptions::body(is_hovered());
    rsx! {
        g {
            rect {
                oncontextmenu: move |evt| handlers.on_menu.call((props.id, evt)),
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
                x: "{props.position.x + layout::WIDTH}",
                y: "{props.position.y}",
                width: "{props.body_width}",
                height: "{layout::HEIGHT}",
                rx: "{layout::PORT_DIMENSIONS.x}",
                ry: "{layout::PORT_DIMENSIONS.y}",
                fill: visuals.fill,
                filter: visuals.filter,
                style: "cursor: grab;",
            }

            text {
                class: "bt-text-sm",
                x: "{props.position.x + layout::WIDTH + (props.body_width / 2.0)}",
                y: "{props.position.y + 16.0}",
                fill: "white",
                font_weight: "medium",
                text_anchor: "middle",
                pointer_events: "none",
                "{data.name}"
            }

            if is_hovered() {
                TooltipCard {
                    anchor: Point {
                        x: props.position.x + (layout::WIDTH * 2.0) + props.body_width + layout::TOOLTIP_GAP.x,
                        y: props.position.y + layout::TOOLTIP_GAP.y,
                    },
                    lines: vec![
                        format!("ID : {}", props.id),
                        format!("Type : {}", data.kind_label),
                        format!("Capacity : {}", data.capacity_label),
                    ],
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct ReceiverBodyProps {
    id: ChannelId,
    position: Point,
    body_width: f64,
    is_connected: bool,
}

#[component]
fn ReceiverBody(props: ReceiverBodyProps) -> Element {
    let mut is_hovered = use_signal(|| false);
    let visuals = VisualOptions::receiver(props.is_connected, is_hovered());
    let handlers = use_context::<Handlers>();

    rsx! {
        g {
            rect {
                onmouseup: move |_| { handlers.receiver_on_mouse_up.call(props.id) },
                oncontextmenu: move |evt|  handlers.on_menu.call((props.id, evt)),
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
                x: "{props.position.x + layout::WIDTH + props.body_width}",
                y: "{props.position.y}",
                width: "{layout::WIDTH}",
                height: "{layout::HEIGHT}",
                rx: "{layout::PORT_DIMENSIONS.x}",
                ry: "{layout::PORT_DIMENSIONS.y}",
                fill: visuals.fill,
                filter: visuals.filter,
                stroke: visuals.stroke,
                stroke_width: Stroke::WIDTH,
                style: "cursor: grab;",
            }
            circle {
                cx: "{props.position.x + layout::WIDTH + props.body_width + layout::PORT_CENTER.x}",
                cy: "{props.position.y + layout::PORT_CENTER.y}",
                r: "3",
                fill: PORT_CIRCLE_INDICATOR_FILL,
                pointer_events: "none",
            }
        }
    }
}
