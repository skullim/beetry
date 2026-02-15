pub mod base;
pub mod menu;
pub mod parameter;
pub mod port;
pub mod renderer;

pub(super) mod control;
pub(super) mod leaf;
pub(super) mod root;

pub use menu::Menu;
pub use renderer::Renderer;

use crate::Point;
use dioxus::prelude::*;

pub const PARAMETER_POSITION: Point = Point { x: 300.0, y: 200.0 };

pub fn style_defs() -> Element {
    rsx! {
        {root::style_defs()}
        {control::style_defs()}
        {leaf::style_defs()}
        {port::style_defs()}
    }
}
