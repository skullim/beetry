pub mod config_dialog;
mod context_menu;
mod renderer;
pub mod temporary;

use crate::Point;
use crate::definitions::EdgePos;
use crate::editor::ServiceContext;
use crate::ui::curve::Curve;
use crate::ui::shadow;
use crate::ui::text::{self, text_width_from};
use crate::ui::viewport::{ViewportContext, ZoomLevel};
use beetry_editor_backend::channel::ChannelQueryApi;
use beetry_editor_types::id::ChannelId;
use beetry_plugin::Named;
pub use config_dialog::Dialog as ConfigDialog;
pub use context_menu::{ContextMenu, Handlers as ContextMenuHandlers, State as ContextMenuState};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
pub use renderer::Renderer;
pub use temporary::Temporary;

#[derive(Debug, Clone)]
pub struct Handlers {
    on_drag_start: EventHandler<(ChannelId, Point)>,
    on_receiver: EventHandler<ChannelId>,
    on_sender: EventHandler<ChannelId>,
    on_context_menu: EventHandler<(ChannelId, Point)>,
}

impl Handlers {
    pub(crate) fn new(
        on_drag_start: impl FnMut((ChannelId, Point)) + 'static,
        on_receiver: impl FnMut(ChannelId) -> Result<()> + 'static,
        on_sender: impl FnMut(ChannelId) -> Result<()> + 'static,
        on_context_menu: impl FnMut((ChannelId, Point)) + 'static,
    ) -> Self {
        Self {
            on_drag_start: EventHandler::new(on_drag_start),
            on_receiver: EventHandler::new(on_receiver),
            on_sender: EventHandler::new(on_sender),
            on_context_menu: EventHandler::new(on_context_menu),
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
}

pub struct GradientHoverUrl;

impl GradientHoverUrl {
    pub const SENDER: &'static str = "url(#channel-sender-gradient-hover)";
    pub const SENDER_EXTERNAL: &'static str = "url(#channel-external-sender-gradient-hover)";
    pub const BODY: &'static str = "url(#channel-body-gradient-hover)";
    pub const RECEIVER: &'static str = "url(#channel-receiver-gradient-hover)";
    pub const RECEIVER_EXTERNAL: &'static str = "url(#channel-external-receiver-gradient-hover)";
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
pub struct ChannelProps {
    id: ChannelId,
    position: Point,
}

#[component]
pub(crate) fn Channel(props: ChannelProps) -> Element {
    let id = props.id;
    debug!("rendering channel {id}");
    let position = props.position;

    let service = use_context::<ServiceContext>();
    let read = service.read();
    let channel_query_api = beetry_editor_backend::api::channel::borrow(&(*read));

    let name = channel_query_api.spec(id).unwrap().name();

    let zoom_level = use_context::<ViewportContext>().zoom_level;
    let handlers = use_context::<Handlers>();

    let font_size = 10;
    let body_width = text_width_from(name, font_size);

    let mut sender_hovered = use_signal(|| false);
    let mut body_hovered = use_signal(|| false);
    let mut receiver_hovered = use_signal(|| false);
    let mut show_tooltip = use_signal(|| false);

    let on_context_menu = move |evt: Event<MouseData>| {
        evt.prevent_default();
        evt.stop_propagation();
        let click_point = Point {
            x: evt.element_coordinates().x,
            y: evt.element_coordinates().y,
        };
        handlers.on_context_menu.call((id, click_point));
    };

    rsx! {
        g {
            onmousedown: move |evt| {
                on_mouse_down(evt, position, id, zoom_level.into(), &handlers.on_drag_start);
            },

            // Sender port (left side)
            rect {
                onmouseup: move |_| { handlers.on_sender.call(id) },
                oncontextmenu: on_context_menu,
                onmouseenter: move |_| sender_hovered.set(true),
                onmouseleave: move |_| sender_hovered.set(false),
                x: "{position.x}",
                y: "{position.y}",
                width: "40",
                height: "25",
                rx: "10",
                ry: "10",
                fill: if *sender_hovered.read() { GradientHoverUrl::SENDER } else { GradientUrl::SENDER },
                filter: if *sender_hovered.read() { shadow::FilterUrl::SHADOW_HOVER } else { shadow::FilterUrl::SHADOW },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                style: "cursor: grab;",
            }

            // Main body
            rect {
                oncontextmenu: on_context_menu,
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
                fill: if *body_hovered.read() { GradientHoverUrl::BODY } else { GradientUrl::BODY },
                filter: if *body_hovered.read() { shadow::FilterUrl::SHADOW_HOVER } else { shadow::FilterUrl::SHADOW },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
                style: "cursor: grab;",
            }

            // Receiver port (right side)
            rect {
                onmouseup: move |_| { handlers.on_receiver.call(id) },
                oncontextmenu: on_context_menu,
                onmouseenter: move |_| receiver_hovered.set(true),
                onmouseleave: move |_| receiver_hovered.set(false),
                x: "{position.x + 40.0 + body_width}",
                y: "{position.y}",
                width: "40",
                height: "25",
                rx: "10",
                ry: "10",
                fill: if *receiver_hovered.read() { GradientHoverUrl::RECEIVER } else { GradientUrl::RECEIVER },
                filter: if *receiver_hovered.read() { shadow::FilterUrl::SHADOW_HOVER } else { shadow::FilterUrl::SHADOW },
                stroke: "rgba(255,255,255,0.3)",
                stroke_width: "1",
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
