//! # Beetry Plugin
//!
//! This crate provides the plugin registration and factory infrastructure
//! needed when custom nodes or channels must be available in the editor or
//! reconstructible from serialized trees.
//!
//! At a high level, a plugin contributes two things:
//!
//! - a spec, so the editor and reconstruction pipeline know what exists
//! - a factory, so runtime objects can be created from serialized tree data
//!
//! This keeps the core crates generic while allowing each application to
//! define its own behavior tree vocabulary.
//!
//! The macros provided by this crate generate the corresponding plugin type
//! and register it automatically.
//!
//! ## Channel plugins
//!
//! Channel plugins start with a message type. The message must provide the
//! metadata Beetry uses for typing and editor integration.
//!
//! See [`channel!`] for the full example and usage details.
//!
//! ## Node plugins
//!
//! Beetry provides node plugin macros for the main behavior tree categories:
//!
//! - [`action!`] for leaf nodes that perform work
//! - [`condition!`] for leaf nodes that evaluate success or failure
//! - [`control!`] for nodes that manage multiple children
//! - [`decorator!`] for nodes that wrap a single child
//!
//! Each macro-generated plugin:
//!
//! - publishes a `NodeSpec`
//! - publishes port metadata and parameter metadata
//! - provides a factory that reconstructs the runtime behavior from stored
//!   parameters and resolved channel endpoints
//!
//! ## Parameters
//!
//! For nodes with customizable parameters, implement [`ProvideParamSpec`] for
//! a params struct to expose editable parameter metadata to the editor.
//!
//! See [`action!`], [`condition!`], [`control!`], and [`decorator!`] for the
//! macro-level examples that show how to attach a params schema with
//! `params(...)` and reconstruct the typed value at runtime.

pub mod channel;
mod channel_macro;
pub mod node;
mod node_macro;

pub use beetry_editor_types::spec::node::{
    FieldDefinition, FieldMetadata, FieldTypeSpec, ParamsSpec, ProvideParamSpec,
};
pub use beetry_reconstruction_types::params::ParamsReconstructor;

pub trait Plugin {
    type Spec;
    type Factory;

    fn new() -> Self
    where
        Self: Sized;

    fn spec(&self) -> &Self::Spec;

    fn factory(&self) -> &Self::Factory;

    fn into_parts(self: Box<Self>) -> (Self::Spec, Self::Factory);
}

pub type BoxPlugin<S, F> = Box<dyn Plugin<Spec = S, Factory = F>>;

impl<S, F> fmt::Debug for dyn Plugin<Spec = S, Factory = F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Plugin<Spec = {}, Factory = {}>",
            std::any::type_name::<S>(),
            std::any::type_name::<F>(),
        )
    }
}

pub trait Named {
    fn name(&self) -> &str;
}

/// Internal helper trait to define unique plugins filtering
trait ConstructPlugin {
    type Spec: Named;
    type Factory;
    fn construct(&self) -> BoxPlugin<Self::Spec, Self::Factory>;
}

pub struct PluginConstructor<S, F>(pub fn() -> BoxPlugin<S, F>);

impl<S, F> PluginConstructor<S, F>
where
    S: 'static,
    F: 'static,
{
    #[must_use]
    pub const fn new<P: Plugin<Spec = S, Factory = F> + 'static>() -> Self {
        Self(|| Box::new(P::new()))
    }
}

impl<S, F> ConstructPlugin for PluginConstructor<S, F>
where
    S: Named,
{
    type Factory = F;
    type Spec = S;
    fn construct(&self) -> BoxPlugin<Self::Spec, Self::Factory> {
        (self.0)()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PluginError {
    #[error("duplicate plugin name: '{0}'. Each plugin must have a unique name.")]
    DuplicateName(String),
}

pub(crate) fn unique_plugins<C, S, F>() -> Result<Vec<BoxPlugin<S, F>>, PluginError>
where
    S: Named,
    C: inventory::Collect + ConstructPlugin<Spec = S, Factory = F>,
{
    let mut seen_names = HashSet::new();

    inventory::iter::<C>().try_fold(Vec::new(), |mut plugins, constructor| {
        let plugin = constructor.construct();
        let spec = plugin.spec();
        let name = spec.name();

        if seen_names.contains(name) {
            return Err(PluginError::DuplicateName(name.into()));
        }
        seen_names.insert(name.to_string());

        plugins.push(plugin);
        Ok(plugins)
    })
}

use std::{collections::HashSet, fmt};

pub use inventory;

#[macro_export]
macro_rules! submit {
    ($plugin:expr) => {
        $crate::inventory::submit!($plugin);
    };
}

#[doc(hidden)]
pub mod __macro_support {
    pub use anyhow;
    pub use beetry_channel;
    pub use beetry_core::{BoxActionBehavior, BoxConditionBehavior};
    pub use beetry_editor_types::spec::channel::ChannelSpec;
    pub use beetry_editor_types::spec::message::MessageSpec;
    pub use beetry_editor_types::spec::node::{
        NodeKind, NodeName, NodePortKind, NodePortSpec, NodeSpec, NodeSpecKey, PortsSpec,
    };
    pub use beetry_reconstruction_types::node::{
        ActionReconstructionData, ConditionReconstructionData,
    };
    pub use mitsein::iter1::FromIterator1;
}
