use dioxus::prelude::*;

use crate::{definitions::EdgePos, ui::style::curve::Curve};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Dragged {
        pos: EdgePos,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveOrientation {
    Horizontal,
    Vertical,
}

#[component]
pub fn Temporary(
    state: ReadSignal<State>,
    orientation: CurveOrientation,
    stroke: &'static str,
) -> Element {
    debug!("rendering (data: {state:?})");
    let state = state.read();

    let pos = match &*state {
        State::Idle => return rsx!(),
        State::Dragged { pos } => pos,
    };

    let d = match orientation {
        CurveOrientation::Horizontal => Curve::calculate_horizontal(&pos.start, &pos.end),
        CurveOrientation::Vertical => Curve::calculate_vertical(&pos.start, &pos.end),
    };

    rsx! {
        path {
            d: "{d}",
            stroke,
            stroke_width: "2",
            fill: "none",
            stroke_dasharray: "5,5",
            opacity: "0.7",
            style: "pointer-events: none",
        }
    }
}
