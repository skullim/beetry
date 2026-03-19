//! Opens the Beetry editor with the example plugin set registered.
//!
//! To work with the parking example, import the corresponding project `.json`
//! file into the editor.
//!
//! Example:
//! `beetry-example/src/domain/parking/project.json`

// Import all crates that register plugins so they are available in the editor.
use beetry_example as _;

fn main() {
    beetry::editor::launch();
}
