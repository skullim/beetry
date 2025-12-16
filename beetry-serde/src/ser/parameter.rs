use anyhow::Result;
use bon::Builder;
use getset::CopyGetters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

pub trait ProvideSchema {
    fn provide() -> Schema;
}

#[derive(Debug, Clone, Builder, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Definition {
    #[builder(into)]
    pub name: String,
    pub ty: Type,
    #[builder(into)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Builder, PartialEq, Eq, Hash, Serialize, Deserialize, CopyGetters)]
#[get_copy = "pub"]
pub struct Bounds {
    min: i64,
    max: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Boolean,
    Integer { bounds: Option<Bounds> },
    Float { bounds: Option<Bounds> },
    String { max_length: Option<usize> },
}

//@todo: Switch to Schema2

pub struct Schema2 {
    pub defs: Vec<Definition2>,
}

impl Schema2 {
    pub fn new(defs: impl IntoIterator<Item = Definition2>) -> Self {
        Self {
            defs: defs.into_iter().collect(),
        }
    }
}

type BoxValidationFn<T> = Box<dyn Fn(&T) -> Result<()>>;
type BoolValidationFn = BoxValidationFn<bool>;
type IntegerValidationFn = BoxValidationFn<i32>;
type FloatValidationFn = BoxValidationFn<f32>;
type StringValidationFn = BoxValidationFn<String>;

pub struct ValidationFns<T> {
    fns: Vec<BoxValidationFn<T>>,
}

impl<T> ValidationFns<T> {
    pub fn validate(&self, value: T) -> bool {
        self.fns.iter().all(|func| (func)(&value).is_err())
    }
}

pub enum Type2 {
    Boolean(BoolValidationFn),
    Integer(IntegerValidationFn),
    Float(FloatValidationFn),
    String(StringValidationFn),
}

#[derive(Builder)]
pub struct Definition2 {
    #[builder(into)]
    pub name: String,
    pub ty: Type2,
    #[builder(into)]
    pub description: Option<String>,
}
