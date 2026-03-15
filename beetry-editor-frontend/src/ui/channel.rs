pub mod dialog;
pub mod edge;
pub mod edge_menu;
pub mod menu;
pub mod renderer;

use beetry_editor_backend::{api, api::ChannelQueryView};
use beetry_editor_types::id::ChannelId;
use beetry_plugin::Named;
use dioxus::prelude::*;
pub use menu::Menu;
pub use renderer::{ConnectionRenderer, Renderer};

use crate::{
    Backend, Point,
    ui::{
        error::ErrorQueueState,
        handler::define_handlers,
        shadow,
        text::{self, text_width_from},
        tooltip::TooltipCard,
    },
};

pub mod layout {
    use crate::Point;

    pub const PORT_WIDTH: f64 = 40.0;
    pub const HEIGHT: f64 = 25.0;
    pub const TOOLTIP_GAP: Point = Point { x: 8.0, y: 0.0 };
    pub const PORT_CENTER: Point = Point {
        x: PORT_WIDTH / 2.0,
        y: HEIGHT / 2.0,
    };
}

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
    let name = spec.name();
    let (kind_label, capacity_label, sender_connected, receiver_connected) =
        match channel_query_api.config(id) {
            Ok(config) => (
                config.kind().to_string(),
                config.capacity().to_string(),
                config.count().sender() > 0,
                config.count().receiver() > 0,
            ),
            Err(e) => {
                errors.push(e);
                ("unknown".to_string(), "unknown".to_string(), false, false)
            }
        };

    let font_size = text::FONT_SIZE_SMALL;
    let body_width = text_width_from(name, font_size);

    let mut sender_hovered = use_signal(|| false);
    let mut body_hovered = use_signal(|| false);
    let mut receiver_hovered = use_signal(|| false);

    let handlers = use_context::<Handlers>();
    let on_menu = move |evt| handlers.on_menu.call((id, evt));

    rsx! {
        g {
            onmousedown: move |evt| {
                handlers.on_mouse_down.call((id, position, evt));
            },

            // Sender port (left side)
            rect {
                onmouseup: move |_| { handlers.sender_on_mouse_up.call(id) },
                oncontextmenu: on_menu,
                onmouseenter: move |_| sender_hovered.set(true),
                onmouseleave: move |_| sender_hovered.set(false),
                x: "{position.x}",
                y: "{position.y}",
                width: "{layout::PORT_WIDTH}",
                height: "{layout::HEIGHT}",
                rx: "10",
                ry: "10",
                fill: if sender_connected { if *sender_hovered.read() { GradientHoverUrl::SENDER } else { GradientUrl::SENDER } } else if *sender_hovered.read() { GradientHoverUrl::DISCONNECTED_SENDER } else { GradientUrl::DISCONNECTED_SENDER },
                filter: if *sender_hovered.read() { shadow::FilterUrl::SHADOW_HOVER } else { shadow::FilterUrl::SHADOW },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                style: "cursor: grab;",
            }

            // Main body
            rect {
                oncontextmenu: on_menu,
                onmouseenter: move |_| {
                    body_hovered.set(true);
                },
                onmouseleave: move |_| {
                    body_hovered.set(false);
                },
                x: "{position.x + layout::PORT_WIDTH}",
                y: "{position.y}",
                width: "{body_width}",
                height: "{layout::HEIGHT}",
                rx: "10",
                ry: "10",
                fill: if *body_hovered.read() { GradientHoverUrl::BODY } else { GradientUrl::BODY },
                filter: if *body_hovered.read() { shadow::FilterUrl::SHADOW_HOVER } else { shadow::FilterUrl::SHADOW },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                style: "cursor: grab;",
            }

            // Receiver port (right side)
            rect {
                onmouseup: move |_| { handlers.receiver_on_mouse_up.call(id) },
                oncontextmenu: on_menu,
                onmouseenter: move |_| receiver_hovered.set(true),
                onmouseleave: move |_| receiver_hovered.set(false),
                x: "{position.x + layout::PORT_WIDTH + body_width}",
                y: "{position.y}",
                width: "{layout::PORT_WIDTH}",
                height: "{layout::HEIGHT}",
                rx: "10",
                ry: "10",
                fill: if receiver_connected { if *receiver_hovered.read() {
                    GradientHoverUrl::RECEIVER
                } else {
                    GradientUrl::RECEIVER
                } } else if *receiver_hovered.read() { GradientHoverUrl::DISCONNECTED_RECEIVER } else { GradientUrl::DISCONNECTED_RECEIVER },
                filter: if *receiver_hovered.read() { shadow::FilterUrl::SHADOW_HOVER } else { shadow::FilterUrl::SHADOW },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                style: "cursor: grab;",
            }

            text {
                class: "bt-text-sm",
                x: "{position.x + layout::PORT_WIDTH + (body_width / 2.0)}",
                y: "{position.y + 16.0}",
                fill: "white",
                font_weight: "medium",
                text_anchor: "middle",
                pointer_events: "none",
                "{name}"
            }

            if *body_hovered.read() {
                TooltipCard {
                    anchor: Point {
                        x: position.x + (layout::PORT_WIDTH * 2.0) + body_width + layout::TOOLTIP_GAP.x,
                        y: position.y + layout::TOOLTIP_GAP.y,
                    },
                    lines: vec![
                        format!("ID : {id}"),
                        format!("Type : {kind_label}"),
                        format!("Capacity : {capacity_label}"),
                    ],
                }
            }

            // Port indicators (small dots)
            // Sender
            circle {
                cx: "{position.x + layout::PORT_CENTER.x}",
                cy: "{position.y + layout::PORT_CENTER.y}",
                r: "3",
                fill: "rgba(255,255,255,0.8)",
                pointer_events: "none",
            }
            // Receiver
            circle {
                cx: "{position.x + layout::PORT_WIDTH + body_width + layout::PORT_CENTER.x}",
                cy: "{position.y + layout::PORT_CENTER.y}",
                r: "3",
                fill: "rgba(255,255,255,0.8)",
                pointer_events: "none",
            }
        }
    }
}

pub fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "channel-sender-gradient",
                stop { offset: "5%", stop_color: "#10B981" }
                stop { offset: "95%", stop_color: "#059669" }
            }

            linearGradient { id: "channel-external-sender-gradient",
                stop { offset: "5%", stop_color: "#095038ff" }
                stop { offset: "95%", stop_color: "#033d2bff" }
            }

            linearGradient { id: "channel-body-gradient",
                stop { offset: "5%", stop_color: "#3B82F6" }
                stop { offset: "95%", stop_color: "#1D4ED8" }
            }

            linearGradient { id: "channel-receiver-gradient",
                stop { offset: "5%", stop_color: "#6B7280" }
                stop { offset: "95%", stop_color: "#374151" }
            }

            linearGradient { id: "channel-disconnected-sender-gradient",
                stop { offset: "5%", stop_color: "#b45309" }
                stop { offset: "95%", stop_color: "#92400e" }
            }

            linearGradient { id: "channel-disconnected-receiver-gradient",
                stop { offset: "5%", stop_color: "#f43f5e" }
                stop { offset: "95%", stop_color: "#e11d48" }
            }

            linearGradient { id: "channel-external-receiver-gradient",
                stop { offset: "5%", stop_color: "#24272cff" }
                stop { offset: "95%", stop_color: "#0a0c0fff" }
            }

            linearGradient { id: "channel-sender-gradient-hover",
                stop { offset: "5%", stop_color: "#34D399" }
                stop { offset: "95%", stop_color: "#10B981" }
            }

            linearGradient { id: "channel-external-sender-gradient-hover",
                stop { offset: "5%", stop_color: "#21966bff" }
                stop { offset: "95%", stop_color: "#058d60ff" }
            }

            linearGradient { id: "channel-body-gradient-hover",
                stop { offset: "5%", stop_color: "#60A5FA" }
                stop { offset: "95%", stop_color: "#3B82F6" }
            }

            linearGradient { id: "channel-receiver-gradient-hover",
                stop { offset: "5%", stop_color: "#9CA3AF" }
                stop { offset: "95%", stop_color: "#6B7280" }
            }

            linearGradient { id: "channel-disconnected-sender-gradient-hover",
                stop { offset: "5%", stop_color: "#d97706" }
                stop { offset: "95%", stop_color: "#b45309" }
            }

            linearGradient { id: "channel-disconnected-receiver-gradient-hover",
                stop { offset: "5%", stop_color: "#fb7185" }
                stop { offset: "95%", stop_color: "#f43f5e" }
            }

            linearGradient { id: "channel-external-receiver-gradient-hover",
                stop { offset: "5%", stop_color: "#393d44ff" }
                stop { offset: "95%", stop_color: "#14171bff" }
            }
        }
    }
}

pub struct GradientUrl;

impl GradientUrl {
    pub const SENDER: &'static str = "url(#channel-sender-gradient)";
    pub const SENDER_EXTERNAL: &'static str = "url(#channel-external-sender-gradient)";
    pub const BODY: &'static str = "url(#channel-body-gradient)";
    pub const RECEIVER: &'static str = "url(#channel-receiver-gradient)";
    pub const RECEIVER_EXTERNAL: &'static str = "url(#channel-external-receiver-gradient)";
    pub const DISCONNECTED_SENDER: &'static str = "url(#channel-disconnected-sender-gradient)";
    pub const DISCONNECTED_RECEIVER: &'static str = "url(#channel-disconnected-receiver-gradient)";
}

pub struct GradientHoverUrl;

impl GradientHoverUrl {
    pub const SENDER: &'static str = "url(#channel-sender-gradient-hover)";
    pub const SENDER_EXTERNAL: &'static str = "url(#channel-external-sender-gradient-hover)";
    pub const BODY: &'static str = "url(#channel-body-gradient-hover)";
    pub const RECEIVER: &'static str = "url(#channel-receiver-gradient-hover)";
    pub const RECEIVER_EXTERNAL: &'static str = "url(#channel-external-receiver-gradient-hover)";
    pub const DISCONNECTED_SENDER: &'static str =
        "url(#channel-disconnected-sender-gradient-hover)";
    pub const DISCONNECTED_RECEIVER: &'static str =
        "url(#channel-disconnected-receiver-gradient-hover)";
}
