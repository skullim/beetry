//! Backend services for the Beetry editor.
//!
//! This crate owns the editor-facing application layer operations. The layered
//! architecture is displayed below:
#![doc = embed_doc_image::embed_image!(
    "backend_layers",
    "docs/layers.excalidraw.png"
)]
//! Consult [`api`] for offered API.

//! ![Beetry editor backend layers][backend_layers]
//! # FAQ
//!
//! ## Why is importing a serialized project/tree not the only prerequisite i.e. why must used plugins be loaded?
//! It is because of how editor and tree reconstruction work.
//!
//! Longer answer:
//! Advanced parameter-value validation requires objects that
//! cannot be easily serialized. Serializing all project/tree dependencies would
//! also increase the exported file size and is less flexible than the
//! alternative solution.
//! However, if this is a major concern, one can consider implementing a
//! "limited" editor view that allows only basic operations.
//!
//! Tree reconstruction requires a constructor
//! that instantiates a given node. This object is not trivially serializable.
//! The chosen approach keeps tree export as small as needed and avoids some
//! types of breaking changes.

pub mod api;
mod id;
mod repository;
mod service;

pub use service::{channel, edge, node, ui};
pub type EditorService = api::contract::EditorService;
