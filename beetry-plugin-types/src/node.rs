use std::cmp::Ordering;
use std::collections::BTreeSet;

use bon::Builder;
use derive_more::{Display, From};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::node::leaf_schema_builder::SetKind;

use super::channel::MessageSpec;
use super::parameter;

pub type RootSpec = NodeSpec<RootSchema>;
pub type LeafSpec = NodeSpec<LeafSchema>;
pub type ActionSpec = LeafSpec;
pub type ConditionSpec = LeafSpec;
pub type ControlSpec = NodeSpec<ControlSchema>;
pub type DecoratorSpec = NodeSpec<DecoratorSchema>;

#[derive(Debug, Builder, Clone, Eq, Getters, Serialize, Deserialize)]
#[get = "pub"]
pub struct NodeSpec<S> {
    #[builder(into)]
    pub name: NodeName,
    pub schema: S,
    #[builder(default)]
    pub params_schema: parameter::Schema,
}

impl<S> PartialEq for NodeSpec<S> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<S> PartialOrd for NodeSpec<S>
where
    S: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S> Ord for NodeSpec<S>
where
    S: Eq + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, From,
)]
pub struct NodeName(pub String);

impl NodeName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl From<&'static str> for NodeName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RootSchema;

#[derive(Debug, Clone, Default)]
pub struct ControlSchema;

#[derive(Debug, Clone, Default)]
pub struct DecoratorSchema;

//@todo use NodePortSpec here
#[derive(Debug, Clone, PartialEq, Eq, Builder, CopyGetters, Getters, Serialize, Deserialize)]
pub struct LeafSchema {
    #[get_copy = "pub"]
    pub kind: LeafKind,
    #[get = "pub"]
    #[builder(default, with = <_>::from_iter)]
    pub receivers: BTreeSet<MessageSpec>,
    #[get = "pub"]
    #[builder(default, with = <_>::from_iter)]
    pub senders: BTreeSet<MessageSpec>,
}

pub struct ActionLeafSchema;
impl ActionLeafSchema {
    // cannot implement Default here as that would need to return ZST instead of LeafSchema
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> LeafSchema {
        Self::builder().build()
    }

    pub fn builder() -> LeafSchemaBuilder<SetKind> {
        LeafSchema::builder().kind(LeafKind::Action)
    }
}

pub struct ConditionLeafSchema;
impl ConditionLeafSchema {
    // cannot implement Default here as that would need to return ZST instead of LeafSchema
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> LeafSchema {
        Self::builder().build()
    }

    pub fn builder() -> LeafSchemaBuilder<SetKind> {
        LeafSchema::builder().kind(LeafKind::Condition)
    }
}

//@todo harmonize with NodeKind
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LeafKind {
    Action,
    Condition,
}

#[macro_export]
macro_rules! spec {
    ( type = action,
      name = $name: literal
      $(, params = $params_schema:expr)?
      $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),* ])?
      $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
    ) =>
    {
        spec! {
            spec_builder: $crate::node::ActionSpec::builder(),
            schema_kind = action,
            name = $name
            $(, params = $params_schema)?
            $(, senders = [$($snd_ty, desc = $snd_desc),*])?
            $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
        }

    };

    ( type = condition,
      name = $name: literal
      $(, params = $params_schema:expr )?
      $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
      $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
    ) =>
    {
        spec! {
            spec_builder: $crate::node::ConditionSpec::builder(),
            schema_kind = condition,
            name = $name
            $(, params = $params_schema)?
            $(, senders = [$($snd_ty, desc = $snd_desc),*])?
            $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
        }
    };

    (
        spec_builder: $builder: expr,
        schema_kind = $schema_kind: ident,
        name = $name: literal
        $(, params = $params_schema:expr )?
        $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
        $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
    ) =>
     {
        {
            let builder = $builder.name($name);
            $(
                let builder = builder.params_schema($params_schema);
            )?
            builder.schema($crate::schema! {
                kind = $schema_kind
                $(, senders = [$($snd_ty, desc = $snd_desc),*])?
                $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
            }).build()
        }
     }
}

#[macro_export]
macro_rules! schema {
    (   kind = action
        $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
        $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
    ) =>
    {
        $crate::schema! { schema_builder = $crate::node::ActionLeafSchema::builder()
                          $(, senders = [$($snd_ty, desc = $snd_desc),*])?
                          $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
        }
    };


    (   kind = condition
        $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
        $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
    ) =>
    {
        $crate::schema! { schema_builder = $crate::node::ConditionLeafSchema::builder()
                          $(, senders = [$($snd_ty, desc = $snd_desc),*])?
                          $(, receivers = [$($rcv_ty, desc = $rcv_desc),*])?
        }
    };

    (   schema_builder = $builder: expr
        $(, senders = [$($snd_ty:ty, desc = $snd_desc:literal),*])?
        $(, receivers = [$($rcv_ty:ty, desc = $rcv_desc:literal),*])?
    ) =>
    {
        {
            let builder = $builder;
            $(
                let builder = builder.senders([
                    $(
                        $crate::channel::MessageSpec::new::<$snd_ty>($snd_desc),
                    )*
                ]);
            )?
            $(
                let builder = builder.receivers([
                    $(
                        $crate::channel::MessageSpec::new::<$rcv_ty>($rcv_desc),
                    )*
                ]);
            )?
            builder.build()
        }
    }
}
