use std::any::Any;
use std::borrow::Borrow;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

pub trait SeDe: Clone + std::fmt::Debug + Default {
    type Error: std::fmt::Debug + std::fmt::Display + Clone;
    type Value: DeserializeOwned + Serialize;

    fn deserialize<T: DeserializeOwned>(input: &str) -> Result<T, Self::Error>;
    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error>;

    /// # Safety
    /// Should only be called in sitatuions where we KNOW de is type T
    unsafe fn serialize_as_any<T: Serialize + 'static + Send + Sync>(input: &Arc<dyn Any + Send + Sync + 'static>) -> Result<String, Self::Error> {
        Self::serialize::<T>(input.clone().downcast::<T>().expect("this method should not be called unless we are sure the downcast will be successful").borrow())
    }
}

pub trait SeDeAny {}

impl<T: Serialize + DeserializeOwned> SeDeAny for T {}

#[derive(Clone, Debug, Default)]
pub struct Json;

impl SeDe for Json {
    type Error = Arc<serde_json::Error>;
    type Value = serde_json::Value;
    fn deserialize<'a, T: DeserializeOwned>(input: &str) -> Result<T, Self::Error> {
        serde_json::from_str(input).map_err( Arc::new)
    }

    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error> {
        serde_json::to_string(input).map_err(Arc::new)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Ron;

impl SeDe for Ron {
    type Error = ron::Error;
    type Value = ron::Value;

    fn deserialize<T: DeserializeOwned>(input: &str) -> Result<T, Self::Error> {
        Ok(ron::de::from_str(input)?)
    }

    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error> {
        ron::to_string(input)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Toml;

#[derive(Clone, Debug, Error)]
pub enum TomlError {
    #[error(transparent)]
    De(#[from] toml::de::Error),
    #[error(transparent)]
    Ser(#[from] toml::ser::Error),
}

impl SeDe for Toml {
    type Error = TomlError;
    type Value = toml::Value;
    fn deserialize<T: DeserializeOwned>(input: &str) -> Result<T, Self::Error> {
        Ok(toml::de::from_str(input)?)
    }
    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error> {
        Ok(toml::ser::to_string(input)?)
    }
}
