use std::any::Any;
use std::borrow::Borrow;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
#[cfg(feature = "toml")]
use thiserror::Error;

/// A trait indicating a scheme for seralizing and deserializing data using Serde
///
/// Can be implmented for other serialization schemes, then create a Freeform<Scheme> to
/// use that scheme for serializing data
pub trait SerdeScheme: Clone + std::fmt::Debug + Default {
    /// Errors that can be returned from serializing/deserializing.
    type Error: std::fmt::Debug + std::fmt::Display + Clone;
    /// The native "value" representation for the scheme. Used when
    /// serializing/deserializing the freeform as a whole so that
    /// the values aren't stored as strings
    type Value: DeserializeOwned + Serialize;

    /// Deserialize a string into a T
    fn deserialize<T: DeserializeOwned>(input: &str) -> Result<T, Self::Error>;
    /// Serialize a T into a string
    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error>;

    /// Used for Freeform internals, default implementation should be sufficient
    ///
    /// # Safety
    /// Should only be called in sitatuions where we KNOW de is type T
    ///
    unsafe fn serialize_as_any<T: Serialize + 'static + Send + Sync>(
        input: &Arc<dyn Any + Send + Sync + 'static>,
    ) -> Result<String, Self::Error> {
        Self::serialize::<T>(input.clone().downcast::<T>().expect("this method should not be called unless we are sure the downcast will be successful").borrow())
    }
}

#[cfg(feature = "json")]
#[derive(Clone, Debug, Default)]
pub struct Json;

#[cfg(feature = "json")]
impl SerdeScheme for Json {
    // Using an Arc because serde_json doesn't implement Clone
    type Error = Arc<serde_json::Error>;
    type Value = serde_json::Value;
    fn deserialize<'a, T: DeserializeOwned>(input: &str) -> Result<T, Self::Error> {
        serde_json::from_str(input).map_err(Arc::new)
    }

    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error> {
        serde_json::to_string(input).map_err(Arc::new)
    }
}

#[cfg(feature = "ron")]
#[derive(Clone, Debug, Default)]
pub struct Ron;

#[cfg(feature = "ron")]
impl SerdeScheme for Ron {
    type Error = ron::Error;
    type Value = ron::Value;

    fn deserialize<T: DeserializeOwned>(input: &str) -> Result<T, Self::Error> {
        Ok(ron::de::from_str(input)?)
    }

    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error> {
        ron::to_string(input)
    }
}

#[cfg(feature = "toml")]
#[derive(Clone, Debug, Default)]
pub struct Toml;

/// Toml has different error types for serializing and deserializing, this wraps both of them
#[cfg(feature = "toml")]
#[derive(Clone, Debug, Error)]
pub enum TomlError {
    #[error(transparent)]
    De(#[from] toml::de::Error),
    #[error(transparent)]
    Ser(#[from] toml::ser::Error),
}

#[cfg(feature = "toml")]
impl SerdeScheme for Toml {
    type Error = TomlError;
    type Value = toml::Value;
    fn deserialize<T: DeserializeOwned>(input: &str) -> Result<T, Self::Error> {
        Ok(toml::de::from_str(input)?)
    }
    fn serialize<T: Serialize>(input: &T) -> Result<String, Self::Error> {
        Ok(toml::ser::to_string(input)?)
    }
}
