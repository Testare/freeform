use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

pub trait SeDe: Clone + Default {
    type Error: std::fmt::Debug + std::fmt::Display;
    type Value: DeserializeOwned + Serialize;

    fn deserialize<T: DeserializeOwned>(input: &str) -> Result<T, Self::Error>;
    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error>;
}

#[derive(Clone, Default)]
pub struct Json;

impl SeDe for Json {
    type Error = serde_json::Error;
    type Value = serde_json::Value;
    fn deserialize<'a, T: DeserializeOwned>(input: &str) -> Result<T, Self::Error> {
        serde_json::from_str(input)
    }

    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error> {
        serde_json::to_string(input)
    }
}

#[derive(Clone, Default)]
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

#[derive(Clone, Default)]
pub struct Toml;

#[derive(Debug, Error)]
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
