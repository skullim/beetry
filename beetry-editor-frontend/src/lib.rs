// A lot of false positives when linting Dioxus macros
#![allow(unused_qualifications)]

//! Dioxus-based frontend editor called Beehive.
//! This crate is an internal Beetry implementation crate and is not considered
//! part of the public API. For public APIs, use the `beetry` crate.

mod backend;
mod components;
mod definitions;
mod signals;
mod specs;
mod ui;

pub(crate) use backend::Backend;
use beetry_editor_types::output::ui::Point;
use dioxus::desktop::WindowBuilder;
pub use specs::{SharedSpecs, Specs};

/// Launches the Beehive editor.
pub fn launch() {
    dioxus_logger::initialize_default();
    let cfg = dioxus::desktop::Config::default().with_window(
        WindowBuilder::new()
            .with_always_on_top(false)
            .with_title("Beehive 🐝🌳"),
    );
    dioxus::LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(components::app::App);
}
