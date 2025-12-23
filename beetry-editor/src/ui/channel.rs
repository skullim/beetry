pub mod config_dialog;
mod renderer;
pub mod temporary;
mod tracker;

use beetry_core::MessageHash;
use beetry_editor_types::ChannelPosition;
use beetry_plugin::Named;
use beetry_plugin_types::channel::ChannelSpec;
use beetry_reconstruction_types::channel::{ChannelId, ChannelSnapshot};
pub use config_dialog::Dialog as ConfigDialog;
pub use config_dialog::Dialog2 as ConfigDialog2;
pub use renderer::{Renderer, Renderer2};
use serde::{Deserialize, Serialize};
pub use temporary::Temporary;
pub use tracker::Tracker;

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::definitions::{EdgePos, Point};
use crate::editor::ServiceContext;
use crate::ui::curve::Curve;
use crate::ui::text::{self, text_width_from};
use crate::ui::viewport::{ViewportContext, ZoomLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub(crate) tracker: Signal<tracker::Tracker>,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            tracker: Signal::new(tracker::Tracker::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelElement {
    pub(crate) pos: Point,
    pub(crate) snapshot: ChannelSnapshot,
}

impl ChannelElement {
    pub(crate) fn new(snapshot: ChannelSnapshot) -> Self {
        Self {
            pos: Point::default(),
            snapshot,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Handlers {
    on_drag_start: EventHandler<(ChannelId, Point)>,
    on_receiver: EventHandler<(ChannelId, MessageHash)>,
    on_sender: EventHandler<(ChannelId, MessageHash)>,
}

impl Handlers {
    pub(crate) fn new(
        on_drag_start: impl FnMut((ChannelId, Point)) + 'static,
        on_receiver: impl FnMut((ChannelId, MessageHash)) + 'static,
        on_sender: impl FnMut((ChannelId, MessageHash)) + 'static,
    ) -> Self {
        Self {
            on_drag_start: EventHandler::new(on_drag_start),
            on_receiver: EventHandler::new(on_receiver),
            on_sender: EventHandler::new(on_sender),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Handlers2 {
    on_drag_start: EventHandler<(ChannelId, Point)>,
    on_receiver: EventHandler<ChannelId>,
    on_sender: EventHandler<ChannelId>,
}

impl Handlers2 {
    pub(crate) fn new(
        on_drag_start: impl FnMut((ChannelId, Point)) + 'static,
        on_receiver: impl FnMut(ChannelId) + 'static,
        on_sender: impl FnMut(ChannelId) + 'static,
    ) -> Self {
        Self {
            on_drag_start: EventHandler::new(on_drag_start),
            on_receiver: EventHandler::new(on_receiver),
            on_sender: EventHandler::new(on_sender),
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ChannelProps {
    pos: Point,
    spec: ChannelSpec,
    id: ChannelId,
}

#[component]
pub(crate) fn Channel(props: ChannelProps) -> Element {
    let position = props.pos;
    let spec = props.spec.clone();
    let id = props.id;

    let zoom_level = use_context::<ViewportContext>().zoom_level;
    let handlers = use_context::<Handlers>();

    let font_size = 10;
    let body_width = text_width_from(spec.as_str(), font_size);

    let mut sender_hovered = use_signal(|| false);
    let mut body_hovered = use_signal(|| false);
    let mut receiver_hovered = use_signal(|| false);
    let mut show_tooltip = use_signal(|| false);

    rsx! {
        g {
            onmousedown: move |evt| {
                on_mouse_down(evt, position, id, zoom_level.into(), &handlers.on_drag_start);
            },

            // Sender port (left side)
            rect {
                onmouseup: move |_| { handlers.on_sender.call((id, spec.msg_hash())) },
                onmouseenter: move |_| sender_hovered.set(true),
                onmouseleave: move |_| sender_hovered.set(false),
                x: "{position.x}",
                y: "{position.y}",
                width: "40",
                height: "25",
                rx: "10",
                ry: "10",
                fill: if *sender_hovered.read() { "url(#channel-sender-gradient-hover)" } else { "url(#channel-sender-gradient)" },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                filter: if *sender_hovered.read() { "url(#shadow-hover)" } else { "url(#shadow)" },
                style: "cursor: grab;",
            }

            // Main body
            rect {
                onmouseenter: move |_| {
                    body_hovered.set(true);
                    show_tooltip.set(true);
                },
                onmouseleave: move |_| {
                    body_hovered.set(false);
                    show_tooltip.set(false);
                },
                x: "{position.x + 40.0}",
                y: "{position.y}",
                width: "{body_width}",
                height: "25",
                rx: "10",
                ry: "10",
                fill: if *body_hovered.peek() { "url(#channel-body-gradient-hover)" } else { "url(#channel-body-gradient)" },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                filter: "url(#shadow)",
                style: "cursor: grab;",
            }

            // Receiver port (right side)
            rect {
                onmouseup: move |_| { handlers.on_receiver.call((id, props.spec.clone().msg_hash())) },
                onmouseenter: move |_| receiver_hovered.set(true),
                onmouseleave: move |_| receiver_hovered.set(false),
                x: "{position.x + 40.0 + body_width}",
                y: "{position.y}",
                width: "40",
                height: "25",
                rx: "10",
                ry: "10",
                fill: if *receiver_hovered.peek() { "url(#channel-receiver-gradient-hover)" } else { "url(#channel-receiver-gradient)" },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                filter: if *receiver_hovered.peek() { "url(#shadow-hover)" } else { "url(#shadow)" },
                style: "cursor: grab;",
            }

            text {
                x: "{position.x + 40.0 + (body_width / 2.0)}",
                y: "{position.y + 16.0}",
                fill: "white",
                font_family: text::font_family(),
                font_size: "{font_size}",
                font_weight: "medium",
                text_anchor: "middle",
                pointer_events: "none",
                "{spec.as_str()}"
            }

            // Tooltip for channel ID (only show on hover)
            if *show_tooltip.read() {
                g {
                    rect {
                        x: "{position.x + 38.0 + (body_width / 2.0) - 15.0}",
                        y: "{position.y - 35.0}",
                        width: "30",
                        height: "18",
                        rx: "4",
                        ry: "4",
                        fill: "rgba(0, 0, 0, 0.8)",
                        stroke: "rgba(255, 255, 255, 0.2)",
                        stroke_width: "1",
                    }

                    text {
                        x: "{position.x + 38.0 + (body_width / 2.0)}",
                        y: "{position.y - 23.0}",
                        fill: "white",
                        font_family: text::font_family(),
                        font_size: "{font_size}",
                        font_weight: "400",
                        text_anchor: "middle",
                        pointer_events: "none",
                        "ID: {id}"
                    }
                }
            }

            // Port indicators (small dots)
            // Sender
            circle {
                cx: "{position.x + 20.0}",
                cy: "{position.y + 12.0}",
                r: "3",
                fill: "rgba(255,255,255,0.8)",
                pointer_events: "none",
            }
            // Receiver
            circle {
                cx: "{position.x + 38.0 + body_width + 20.0}",
                cy: "{position.y + 12.0}",
                r: "3",
                fill: "rgba(255,255,255,0.8)",
                pointer_events: "none",
            }
        }
    }
}

fn on_mouse_down(
    evt: Event<MouseData>,
    position: Point,
    id: ChannelId,
    zoom_level: ReadSignal<ZoomLevel>,
    drag_start_cb: &Callback<(ChannelId, Point)>,
) {
    if evt.held_buttons().contains(MouseButton::Primary) {
        let mouse_coords = evt.client_coordinates();
        let zoom_level = zoom_level.peek().get();

        let svg_mouse_coords = Point {
            x: mouse_coords.x / zoom_level,
            y: mouse_coords.y / zoom_level,
        };

        let drag_offset = Point {
            x: svg_mouse_coords.x - position.x,
            y: svg_mouse_coords.y - position.y,
        };
        drag_start_cb.call((id, drag_offset));
    }
}

pub fn style_defs() -> Element {
    rsx! {
        defs {
            linearGradient { id: "channel-sender-gradient",
                stop { offset: "5%", stop_color: "#10B981" }
                stop { offset: "95%", stop_color: "#059669" }
            }

            linearGradient { id: "channel-body-gradient",
                stop { offset: "5%", stop_color: "#3B82F6" }
                stop { offset: "95%", stop_color: "#1D4ED8" }
            }

            linearGradient { id: "channel-receiver-gradient",
                stop { offset: "5%", stop_color: "#6B7280" }
                stop { offset: "95%", stop_color: "#374151" }
            }

            linearGradient { id: "channel-sender-gradient-hover",
                stop { offset: "5%", stop_color: "#34D399" }
                stop { offset: "95%", stop_color: "#10B981" }
            }

            linearGradient { id: "channel-body-gradient-hover",
                stop { offset: "5%", stop_color: "#60A5FA" }
                stop { offset: "95%", stop_color: "#3B82F6" }
            }

            linearGradient { id: "channel-receiver-gradient-hover",
                stop { offset: "5%", stop_color: "#9CA3AF" }
                stop { offset: "95%", stop_color: "#6B7280" }
            }
        }
    }
}

#[component]
pub(crate) fn SenderConnection(edge: EdgePos) -> Element {
    rsx! {
        path {
            d: "{Curve::calculate_horizontal(&edge.start, &edge.end)}",
            stroke: "#10B981", // color matching channel sender gradient
            stroke_width: "3",
            fill: "none",
        }
    }
}

#[component]
pub(crate) fn ReceiverConnection(edge: EdgePos) -> Element {
    rsx! {
        path {
            d: "{Curve::calculate_horizontal(&edge.start, &edge.end)}",
            stroke: "#6B7280", // color matching channel receiver gradient
            stroke_width: "3",
            fill: "none",
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ChannelProps2 {
    id: ChannelId,
    position: ChannelPosition,
}

#[component]
pub(crate) fn Channel2(props: ChannelProps2) -> Element {
    let id = props.id;
    debug!("rendering channel {id}");
    let position = props.position.origin;

    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let channel_api = read.channel_api();

    let name = channel_api.spec(id).unwrap().name();

    let zoom_level = use_context::<ViewportContext>().zoom_level;
    let handlers = use_context::<Handlers2>();

    let font_size = 10;
    let body_width = text_width_from(name, font_size);

    let mut sender_hovered = use_signal(|| false);
    let mut body_hovered = use_signal(|| false);
    let mut receiver_hovered = use_signal(|| false);
    let mut show_tooltip = use_signal(|| false);

    rsx! {
        g {
            onmousedown: move |evt| {
                on_mouse_down(
                    evt,
                    Point {
                        x: position.x,
                        y: position.y,
                    },
                    id,
                    zoom_level.into(),
                    &handlers.on_drag_start,
                );
            },

            // Sender port (left side)
            rect {
                onmouseup: move |_| { handlers.on_sender.call(id) },
                onmouseenter: move |_| sender_hovered.set(true),
                onmouseleave: move |_| sender_hovered.set(false),
                x: "{position.x}",
                y: "{position.y}",
                width: "40",
                height: "25",
                rx: "10",
                ry: "10",
                fill: if *sender_hovered.read() { "url(#channel-sender-gradient-hover)" } else { "url(#channel-sender-gradient)" },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                filter: if *sender_hovered.read() { "url(#shadow-hover)" } else { "url(#shadow)" },
                style: "cursor: grab;",
            }

            // Main body
            rect {
                onmouseenter: move |_| {
                    body_hovered.set(true);
                    show_tooltip.set(true);
                },
                onmouseleave: move |_| {
                    body_hovered.set(false);
                    show_tooltip.set(false);
                },
                x: "{position.x + 40.0}",
                y: "{position.y}",
                width: "{body_width}",
                height: "25",
                rx: "10",
                ry: "10",
                fill: if *body_hovered.peek() { "url(#channel-body-gradient-hover)" } else { "url(#channel-body-gradient)" },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                filter: "url(#shadow)",
                style: "cursor: grab;",
            }

            // Receiver port (right side)
            rect {
                onmouseup: move |_| { handlers.on_receiver.call(id) },
                onmouseenter: move |_| receiver_hovered.set(true),
                onmouseleave: move |_| receiver_hovered.set(false),
                x: "{position.x + 40.0 + body_width}",
                y: "{position.y}",
                width: "40",
                height: "25",
                rx: "10",
                ry: "10",
                fill: if *receiver_hovered.peek() { "url(#channel-receiver-gradient-hover)" } else { "url(#channel-receiver-gradient)" },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                filter: if *receiver_hovered.peek() { "url(#shadow-hover)" } else { "url(#shadow)" },
                style: "cursor: grab;",
            }

            text {
                x: "{position.x + 40.0 + (body_width / 2.0)}",
                y: "{position.y + 16.0}",
                fill: "white",
                font_family: text::font_family(),
                font_size: "{font_size}",
                font_weight: "medium",
                text_anchor: "middle",
                pointer_events: "none",
                "{name}"
            }

            // Tooltip for channel ID (only show on hover)
            if *show_tooltip.read() {
                g {
                    rect {
                        x: "{position.x + 38.0 + (body_width / 2.0) - 15.0}",
                        y: "{position.y - 35.0}",
                        width: "30",
                        height: "18",
                        rx: "4",
                        ry: "4",
                        fill: "rgba(0, 0, 0, 0.8)",
                        stroke: "rgba(255, 255, 255, 0.2)",
                        stroke_width: "1",
                    }

                    text {
                        x: "{position.x + 38.0 + (body_width / 2.0)}",
                        y: "{position.y - 23.0}",
                        fill: "white",
                        font_family: text::font_family(),
                        font_size: "{font_size}",
                        font_weight: "400",
                        text_anchor: "middle",
                        pointer_events: "none",
                        "ID: {id}"
                    }
                }
            }

            // Port indicators (small dots)
            // Sender
            circle {
                cx: "{position.x + 20.0}",
                cy: "{position.y + 12.0}",
                r: "3",
                fill: "rgba(255,255,255,0.8)",
                pointer_events: "none",
            }
            // Receiver
            circle {
                cx: "{position.x + 38.0 + body_width + 20.0}",
                cy: "{position.y + 12.0}",
                r: "3",
                fill: "rgba(255,255,255,0.8)",
                pointer_events: "none",
            }
        }
    }
}
