pub mod channel;
pub mod node;

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
