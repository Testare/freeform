use std::collections::HashMap;

use bevy_reflect::Reflect;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typed_key::Key;

#[cfg(feature = "json")]
use crate::Json;
use crate::SerdeScheme;
use crate::{Sord, SordError};

#[cfg(feature = "json")]
#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[serde(
    try_from = "HashMap<String, S::Value>",
    into = "HashMap<String, S::Value>"
)]
pub struct Freeform<S: SerdeScheme = Json>(
    #[serde(bound(serialize = "", deserialize = ""))] HashMap<String, Sord<S>>,
);

#[cfg(not(feature = "json"))]
#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[serde(
    try_from = "HashMap<String, S::Value>",
    into = "HashMap<String, S::Value>"
)]
pub struct Freeform<S: SerdeScheme>(
    #[serde(bound(serialize = "", deserialize = ""))] HashMap<String, Sord<S>>,
);

/// Trait for data that is generally compatible with being stored in a Freeform
pub trait FreeformData: Serialize + DeserializeOwned + 'static + Send + Sync {}

impl<T: Serialize + DeserializeOwned + 'static + Send + Sync> FreeformData for T {}

#[derive(Clone, Debug, Error)]
pub enum FreeformErr<S: SerdeScheme> {
    #[error("error from serde_json in metadata: {0}")]
    SerdeError(S::Error),
    #[error("required metadata key not found [{0}]")]
    RequiredKeyNotFound(String),
    #[error("The key type doesn't match what was stored")]
    KeyTypeDoesNotMatch,
}

impl<S: SerdeScheme> From<&SordError<S>> for FreeformErr<S> {
    fn from(value: &SordError<S>) -> Self {
        match value {
            SordError::WrongTypeError => FreeformErr::KeyTypeDoesNotMatch,
            SordError::SeDeError(e) => FreeformErr::<S>::SerdeError(e.clone()),
        }
    }
}

impl<S: SerdeScheme> From<SordError<S>> for FreeformErr<S> {
    fn from(value: SordError<S>) -> Self {
        match value {
            SordError::WrongTypeError => FreeformErr::KeyTypeDoesNotMatch,
            SordError::SeDeError(e) => FreeformErr::<S>::SerdeError(e),
        }
    }
}

impl<S: SerdeScheme> Freeform<S> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn deserialize(input: &str) -> Result<Self, FreeformErr<S>> {
        S::deserialize(input).map_err(FreeformErr::SerdeError)
    }

    pub fn serialize(&self) -> Result<String, FreeformErr<S>> {
        S::serialize(self).map_err(FreeformErr::SerdeError)
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_optional<T: FreeformData>(&self, key: Key<T>) -> Result<Option<&T>, FreeformErr<S>> {
        if let Some(value_sord) = self.0.get(&key.name().to_string()) {
            Ok(Some(value_sord.de::<T>()?))
        } else {
            Ok(None)
        }
    }

    pub fn get_owned_or_default<T: FreeformData + ToOwned + Default>(
        &self,
        key: Key<T>,
    ) -> Result<T::Owned, FreeformErr<S>>
    where
        T::Owned: Default,
    {
        self.get_optional(key)
            .map(|opt| opt.map(|t| t.to_owned()).unwrap_or_default())
    }

    pub fn get_cloned_or_default<T: FreeformData + Clone + Default>(
        &self,
        key: Key<T>,
    ) -> Result<T, FreeformErr<S>> {
        self.get_optional(key)
            .map(|opt| opt.cloned().unwrap_or_default())
    }

    pub fn get_required<T: FreeformData>(&self, key: Key<T>) -> Result<&T, FreeformErr<S>> {
        if let Some(value_sord) = self.0.get(&key.name().to_string()) {
            Ok(value_sord.de::<T>()?)
        } else {
            Err(FreeformErr::RequiredKeyNotFound(key.name().to_owned()))
        }
    }

    pub fn put<T: FreeformData>(&mut self, key: Key<T>, data: T) -> Result<(), FreeformErr<S>> {
        let sord_data = Sord::from_de::<T>(data);
        self.0.insert(key.name().to_string(), sord_data);
        Ok(())
    }

    /// Puts a value by ref by serializing and storing that way
    pub fn put_ref<T: FreeformData>(
        &mut self,
        key: Key<T>,
        data: &T,
    ) -> Result<(), FreeformErr<S>> {
        let sord_data = Sord::from_de_ref::<T>(data)?;
        self.0.insert(key.name().to_string(), sord_data);
        Ok(())
    }

    /// Puts the data if the option is Some, else it does nothing
    pub fn put_optional<T: FreeformData>(
        &mut self,
        key: Key<T>,
        data: Option<T>,
    ) -> Result<(), FreeformErr<S>> {
        if let Some(data_unwrapped) = data {
            self.put(key, data_unwrapped)
        } else {
            Ok(())
        }
    }

    pub fn put_optional_ref<T: FreeformData>(
        &mut self,
        key: Key<T>,
        data: Option<&T>,
    ) -> Result<(), FreeformErr<S>> {
        if let Some(data_unwrapped) = data {
            self.put_ref(key, data_unwrapped)
        } else {
            Ok(())
        }
    }

    // TODO Possible future improvement: Trait object IsEmpty, implemented for metadata, hashmap, and Vec?
    pub fn put_nonempty<T: FreeformData>(
        &mut self,
        key: Key<Vec<T>>,
        data: Vec<T>,
    ) -> Result<(), FreeformErr<S>> {
        if data.is_empty() {
            Ok(())
        } else {
            self.put(key, data)
        }
    }

    pub fn put_nonempty_ref<T: FreeformData>(
        &mut self,
        key: Key<Vec<T>>,
        data: &Vec<T>,
    ) -> Result<(), FreeformErr<S>> {
        if data.is_empty() {
            Ok(())
        } else {
            self.put_ref(key, data)
        }
    }

    pub fn aggregate<F: IntoIterator<Item = Self>>(freeform: F) -> Option<Self> {
        freeform.into_iter().reduce(|mut acm, effects| {
            acm.extend(effects);
            acm
        })
    }
}

impl<S: SerdeScheme> IntoIterator for Freeform<S> {
    type IntoIter = std::collections::hash_map::IntoIter<String, Sord<S>>;
    type Item = (String, Sord<S>);
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<S: SerdeScheme> Extend<(String, Sord<S>)> for Freeform<S> {
    fn extend<T: IntoIterator<Item = (String, Sord<S>)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl<S: SerdeScheme> TryFrom<HashMap<String, S::Value>> for Freeform<S> {
    type Error = FreeformErr<S>;
    fn try_from(map: HashMap<String, S::Value>) -> std::result::Result<Self, Self::Error> {
        let converted_map = map
            .into_iter()
            .map(|(key, val)| Ok((key, Sord::<S>::from_value(&val)?)))
            .collect::<std::result::Result<_, Self::Error>>()?;

        Ok(Freeform(converted_map))
    }
}

impl<S: SerdeScheme> From<Freeform<S>> for HashMap<String, S::Value> {
    fn from(metadata: Freeform<S>) -> Self {
        metadata
            .0
            .into_iter()
            .map(|(key, val)| (key, val.value().expect("Should be able to serialize")))
            .collect()
    }
}

#[cfg(all(test, any(feature = "json", feature = "toml", feature = "ron")))]
mod test {
    #[cfg(feature = "json")]
    use serde_json::{Map, Number, Value};
    use std::collections::HashMap;

    use typed_key::{typed_key, Key};

    #[cfg(feature = "ron")]
    use crate::scheme::Ron;

    use super::Freeform;

    const NUM_KEY: Key<usize> = typed_key!("num");
    const MAP_KEY: Key<HashMap<String, String>> = typed_key!("map");

    fn test_map() -> HashMap<String, String> {
        [("foo", "FOO"), ("bar", "BAR"), ("hello", "bonjour")]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    #[cfg(feature = "json")]
    pub fn basic_test() {
        let mut freeform = <Freeform>::new();
        let ff_key: Key<Freeform> = typed_key!("ff");
        freeform.put(NUM_KEY, 343).unwrap();
        freeform.put(MAP_KEY, test_map()).unwrap();
        freeform
            .put(ff_key, {
                let mut metadata = Freeform::new();
                metadata.put(NUM_KEY, 143).unwrap();
                metadata
            })
            .unwrap();

        let mut inner_freeform_map = Map::new();
        inner_freeform_map.insert(NUM_KEY.name().to_string(), Value::Number(Number::from(143)));

        let mut hashmap_value = Map::new();
        hashmap_value.insert("foo".to_string(), Value::String("FOO".to_string()));
        hashmap_value.insert("bar".to_string(), Value::String("BAR".to_string()));
        hashmap_value.insert("hello".to_string(), Value::String("bonjour".to_string()));

        let mut expected_map = Map::new();
        expected_map.insert(NUM_KEY.name().to_string(), Value::Number(Number::from(343)));
        expected_map.insert(MAP_KEY.name().to_string(), Value::Object(hashmap_value));
        expected_map.insert(ff_key.name().to_string(), Value::Object(inner_freeform_map));

        let result = serde_json::to_value(&freeform).unwrap();

        assert_eq!(Value::Object(expected_map), result);
    }

    #[test]
    #[cfg(feature = "ron")]
    pub fn ron_test() {
        let mut freeform = <Freeform<Ron>>::new();
        let ff_key: Key<Freeform<Ron>> = typed_key!("ff");

        freeform.put(NUM_KEY, 62).unwrap();
        freeform.put(MAP_KEY, test_map()).unwrap();
        let mut ff: Freeform<Ron> = Freeform::new();
        ff.put(NUM_KEY, 143).unwrap();
        freeform.put(ff_key, ff).unwrap();

        let mut expected_map = ron::Map::new();
        expected_map.insert(
            ron::Value::String("foo".to_string()),
            ron::Value::String("FOO".to_string()),
        );
        expected_map.insert(
            ron::Value::String("bar".to_string()),
            ron::Value::String("BAR".to_string()),
        );
        expected_map.insert(
            ron::Value::String("hello".to_string()),
            ron::Value::String("bonjour".to_string()),
        );

        let mut expected_ff = ron::Map::new();
        expected_ff.insert(
            ron::Value::String(NUM_KEY.name().to_string()),
            ron::Value::Number(ron::Number::Integer(143)),
        );

        let mut expected_ron = ron::Map::new();
        expected_ron.insert(
            ron::Value::String(NUM_KEY.name().to_string()),
            ron::Value::Number(ron::Number::Integer(62)),
        );
        expected_ron.insert(
            ron::Value::String(MAP_KEY.name().to_string()),
            ron::Value::Map(expected_map),
        );
        expected_ron.insert(
            ron::Value::String(ff_key.name().to_string()),
            ron::Value::Map(expected_ff),
        );

        let serialized = ron::to_string(&freeform).unwrap();
        let ron_value: ron::Value = ron::from_str(serialized.as_str()).unwrap();
        assert_eq!(ron::Value::Map(expected_ron), ron_value);

        assert_eq!(&62, freeform.get_required(NUM_KEY).unwrap());
        assert_eq!(&test_map(), freeform.get_required(MAP_KEY).unwrap());
        let inner_freeform = freeform.get_required(ff_key).unwrap();
        assert_eq!(Some(&143), inner_freeform.get_optional(NUM_KEY).unwrap());
        assert_eq!(None, inner_freeform.get_optional(MAP_KEY).unwrap());
    }
}
