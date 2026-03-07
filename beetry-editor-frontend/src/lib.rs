mod backend;
mod components;
mod definitions;
mod signals;
mod specs;
mod ui;

use beetry_editor_types::output::ui::Point;
use dioxus::{desktop::WindowBuilder, logger::tracing::Level};

pub(crate) use backend::Backend;
pub use specs::{SharedSpecs, Specs};

#[expect(
    clippy::missing_panics_doc,
    reason = "logger should be always initialized"
)]
pub fn launch() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    let cfg = dioxus::desktop::Config::default().with_window(
        WindowBuilder::new()
            .with_always_on_top(false)
            .with_title("Beetry Editor 🌳"),
    );
    dioxus::LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(components::app::App);
}
