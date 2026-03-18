// A lot of false positives when linting Dioxus macros
#![allow(unused_qualifications)]

//! Dioxus-based frontend for the Beetry editor.
//!
//! This crate contains the desktop UI used to author behavior tree projects.
//! Use [`launch`] to start the desktop editor application.

mod backend;
mod components;
mod definitions;
mod signals;
mod specs;
mod ui;

pub(crate) use backend::Backend;
use beetry_editor_types::output::ui::Point;
use dioxus::{desktop::WindowBuilder, logger::tracing::Level};
pub use specs::{SharedSpecs, Specs};

#[expect(
    clippy::missing_panics_doc,
    reason = "logger should be always initialized"
)]
/// Launches the desktop Beetry editor application.
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
