//! Plugin registration and macros for custom nodes and channels.
//!
//! To get an overview of how the plugin system works in Beetry, the
//! `Plugins` chapter in the book is a good place to start.

pub use beetry_editor_types::spec::node::FieldName;
pub use beetry_plugin::{
    FieldDefinition, FieldMetadata, FieldTypeSpec, ParamsDeserializer, ParamsSpec,
    ProvideParamSpec, action, channel, condition, control, decorator,
};
