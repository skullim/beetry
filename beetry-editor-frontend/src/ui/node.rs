pub mod base;
pub mod menu;
pub mod parameter;
pub mod port;
pub mod renderer;
pub(crate) mod tooltip;

pub(super) mod control;
pub(super) mod leaf;
pub(super) mod root;

pub use menu::Menu;
pub use renderer::Renderer;
use dioxus::prelude::*;

pub fn style_defs() -> Element {
    rsx! {
        {root::style_defs()}
        {control::style_defs()}
        {leaf::style_defs()}
        {port::style_defs()}
    }
}
