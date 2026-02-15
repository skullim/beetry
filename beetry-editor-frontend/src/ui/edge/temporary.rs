use dioxus::prelude::*;
use dioxus_logger::tracing::debug;

use crate::definitions::EdgePos;
use crate::ui::curve::Curve;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Dragged {
        pos: EdgePos,
    },
}

#[component]
pub fn Temporary(state: ReadSignal<State>) -> Element {
    debug!("rendering (data: {state:?})");
    let state = state.read();

    let pos = match &*state {
        State::Idle => return rsx!(),
        State::Dragged { pos } => pos,
    };

    rsx! {
        path {
            d: "{Curve::calculate_vertical(&pos.start, &pos.end)}",
            stroke: "#A78BFA", // Light purple to match output port hover
            stroke_width: "2",
            fill: "none",
            stroke_dasharray: "5,5",
            opacity: "0.7",
            style: "pointer-events: none",
        }
    }
}
