use anyhow::Result;
use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use tracing::{Level, debug, instrument};

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
    fn provide() -> Schema {
        Schema::default()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializedParameters {
    params: serde_json::Value,
}

impl SerializedParameters {
    pub fn from_value(value: serde_json::Value) -> Self {
        Self { params: value }
    }

    pub fn try_into<T>(self) -> Result<T>
    where
        T: TryFromSerializedParameters,
    {
        T::try_from(self)
    }
}

pub trait SerializedParametersMarker {}

pub trait TryFromSerializedParameters: Sized {
    fn try_from(params: SerializedParameters) -> Result<Self>;
}

impl<T> TryFromSerializedParameters for T
where
    T: for<'de> Deserialize<'de> + Default + SerializedParametersMarker,
{
    #[instrument(level = Level::DEBUG)]
    fn try_from(params: SerializedParameters) -> Result<Self> {
        if params.params.is_null() {
            debug!("using default parameter values");
            return Ok(T::default());
        }
        Ok(serde_json::from_value(params.params)?)
    }
}
