pub mod base;
pub mod menu;
pub mod parameter;
pub mod pin;
pub mod port;
pub mod renderer;
pub(crate) mod tooltip;

pub(super) mod control;
pub(super) mod decorator;
pub(super) mod leaf;
pub(super) mod root;

use dioxus::prelude::*;
pub use menu::Menu;
pub use renderer::Renderer;

pub fn style_defs() -> Element {
    rsx! {
        {root::style_defs()}
        {control::style_defs()}
        {decorator::style_defs()}
        {leaf::style_defs()}
        {pin::style_defs()}
    }
}
