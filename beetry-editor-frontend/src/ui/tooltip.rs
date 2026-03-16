use dioxus::prelude::*;

use crate::{Point, ui::style::text};

#[derive(Props, Clone, PartialEq)]
pub struct TooltipCardProps {
    pub anchor: Point,
    pub lines: Vec<String>,
}

#[component]
pub(crate) fn TooltipCard(props: TooltipCardProps) -> Element {
    const TEXT_X_PADDING: f64 = 10.0;
    const LINE_HEIGHT: f64 = 18.0;
    const HEIGHT_BASE_PADDING: f64 = 12.0;
    const FIRST_LINE_BASELINE_OFFSET: f64 = 20.0;
    const MIN_WIDTH: f64 = 50.0;
    const MAX_WIDTH: f64 = 400.0;

    #[expect(
        clippy::cast_precision_loss,
        reason = "number of lines is reasonably small"
    )]
    let line_count = props.lines.len() as f64;
    let longest_line_len = props
        .lines
        .iter()
        .map(|line| text::text_width_from(line, text::FONT_SIZE_NORMAL))
        .fold(0.0, f64::max);
    let width = (longest_line_len + 2.0 * TEXT_X_PADDING).clamp(MIN_WIDTH, MAX_WIDTH);
    let height = (line_count * LINE_HEIGHT) + HEIGHT_BASE_PADDING;
    let text_start_y = -height + FIRST_LINE_BASELINE_OFFSET;

    rsx! {
        g {
            transform: "translate({props.anchor.x} {props.anchor.y})",
            pointer_events: "none",

            rect {
                x: "0",
                y: "{-height}",
                width: "{width}",
                height: "{height}",
                rx: "10.0",
                ry: "10.0",
                fill: "rgba(17, 24, 39, 0.92)",
                stroke: "rgba(255, 255, 255, 0.25)",
                stroke_width: "1.0",
            }

            for (idx , line) in props.lines.iter().enumerate() {
                text {
                    x: "{TEXT_X_PADDING}",
                    y: "{text_start_y + (idx as f64 * LINE_HEIGHT)}",
                    fill: "white",
                    "{line}"
                }
            }
        }
    }
}
