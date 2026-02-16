mod backend;
mod components;
mod definitions;
mod sidebar;
mod signals;
mod specs;
mod toolbar;
mod ui;

use beetry_editor_types::output::ui::Point;
use dioxus::{desktop::WindowBuilder, logger::tracing::Level};

pub(crate) use backend::Backend;
pub use specs::{SharedSpecs, Specs};

#[cfg(target_family = "wasm")]
unsafe extern "C" {
    fn __wasm_call_ctors();
}

pub fn launch() {
    #[cfg(target_family = "wasm")]
    unsafe {
        use dioxus::logger::tracing::info;
        info!("running wasm ctor");
        __wasm_call_ctors();
    }
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");
    let cfg = dioxus::desktop::Config::default().with_window(
        WindowBuilder::new()
            .with_always_on_top(false)
            .with_title("Beetry Editor 🌳"),
    );
    dioxus::LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(components::app::App);
}
