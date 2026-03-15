//! Opens the Beetry editor with the example plugin set registered.
//!
//! Run with:
//! `cargo run -p beetry-example --bin tree_authoring`
//!
//! To work with the parking example, import the corresponding project `.json`
//! file into the editor.
//!
//! Example:
//! `beetry-example/src/domain/parking/project.json`

// Import all crates that register plugins so they are available in the editor.
use beetry_example as _;
use beetry_node as _;

fn main() {
    beetry_editor_frontend::launch();
}
