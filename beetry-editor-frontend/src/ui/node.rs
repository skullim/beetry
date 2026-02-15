mod base;
pub mod menu;
pub mod parameter_dialog;
pub mod port;
mod renderer;

pub(super) mod control;
pub(super) mod leaf;
pub(super) mod root;

pub use base::Handlers;
pub use menu::Menu;
pub use port::ConnectionOrigin;
pub use renderer::Renderer;

use crate::Point;
use dioxus::prelude::*;

pub const PARAM_DIALOG_POSITION: Point = Point { x: 300.0, y: 200.0 };

pub fn style_defs() -> Element {
    rsx! {
        {root::style_defs()}
        {control::style_defs()}
        {leaf::style_defs()}
        {port::style_defs()}
    }
}
