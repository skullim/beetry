use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schema {
    pub defs: Vec<Definition>,
}

impl Schema {
    pub fn new(defs: impl IntoIterator<Item = Definition>) -> Self {
        Self {
            defs: defs.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Builder, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    #[builder(into)]
    pub name: String,
    pub ty: Type,
    #[builder(into)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Builder, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct Bounds {
    #[getter(copy)]
    min: i64,
    #[getter(copy)]
    max: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    Boolean,
    Integer { bounds: Option<Bounds> },
    Float { bounds: Option<Bounds> },
    String { max_length: Option<usize> },
}

pub trait ProvideSchema {
    fn provide() -> Schema;
}
