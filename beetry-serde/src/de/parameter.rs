use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{Level, debug, instrument};

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
