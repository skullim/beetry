//! Plugin registration and macros for custom nodes and channels.

pub use beetry_editor_types::spec::node::FieldName;
pub use beetry_plugin::{
    FieldDefinition, FieldMetadata, FieldTypeSpec, ParamsDeserializer, ParamsSpec,
    ProvideParamSpec, action, channel, condition, control, decorator,
};
