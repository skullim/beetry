//! Plugin registration and macros for custom nodes and channels.
//!
//! To get an overview of how the plugin system works in Beetry, the
//! `Plugins` chapter in the book is a good place to start.

pub use beetry_plugin::{action, channel, condition, control, decorator};

/// Parameter-related types.
///
/// This module groups the public API related to parameter.
pub mod parameter {
    pub use beetry_plugin::{
        FieldDefinition, FieldMetadata, FieldTypeSpec, ParamsDeserializer as Deserializer,
        ParamsSpec as Spec, ProvideParamSpec,
    };
}
