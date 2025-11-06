use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{Level, debug, instrument};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameters {
    params: serde_json::Value,
}

impl Parameters {
    pub fn from_value(value: serde_json::Value) -> Self {
        Self { params: value }
    }

    pub fn try_into<T>(self) -> Result<T>
    where
        T: TryFromParameters,
    {
        T::try_from(self)
    }
}

pub trait ParametersMarker {}

pub trait TryFromParameters: Sized {
    fn try_from(params: Parameters) -> Result<Self>;
}

impl<T> TryFromParameters for T
where
    T: for<'de> Deserialize<'de> + Default + ParametersMarker,
{
    #[instrument(level = Level::DEBUG)]
    fn try_from(params: Parameters) -> Result<Self> {
        if params.params.is_null() {
            debug!("using default parameter values");
            return Ok(T::default());
        }
        Ok(serde_json::from_value(params.params)?)
    }
}
