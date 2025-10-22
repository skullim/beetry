use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schema {
    pub params: Vec<Definition>,
}

impl Schema {
    pub fn new(params: impl IntoIterator<Item = Definition>) -> Self {
        Self {
            params: params.into_iter().collect(),
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
    min: i32,
    #[getter(copy)]
    max: i32,
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
