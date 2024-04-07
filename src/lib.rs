mod sede;
mod sord;
mod typed_sord;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typed_key::Key;

pub use sede::{Json, Ron, SeDe, SeDeAny, Toml};
pub use sord::{Sord, SordError};
pub use typed_sord::TypedSord;

pub use typed_key::typed_key;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(
    try_from = "HashMap<String, S::Value>",
    into = "HashMap<String, S::Value>"
)]
pub struct Freeform<S: SeDe = Json>(#[serde(skip)] PhantomData<S>, HashMap<String, String>);

#[derive(Clone, Debug, Error)]
pub enum FreeformErr {
    #[error("error from serde_json in metadata: {0}")]
    SerdeError(#[from] Arc<serde_json::error::Error>),
    #[error("required metadata key not found [{0}]")]
    RequiredKeyNotFound(String),
}

impl From<serde_json::error::Error> for FreeformErr {
    fn from(value: serde_json::error::Error) -> Self {
        Self::SerdeError(Arc::new(value))
    }
}

type Result<T> = std::result::Result<T, FreeformErr>;

impl<S: SeDe> Freeform<S> {
    pub fn is_empty(&self) -> bool {
        self.1.is_empty()
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_optional<'a, T: Deserialize<'a>>(&'a self, key: Key<T>) -> Result<Option<T>> {
        if let Some(value_str) = self.1.get(&key.name().to_string()) {
            Ok(Some(serde_json::from_str(value_str)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_or_default<'a, T: Deserialize<'a> + Default>(&'a self, key: Key<T>) -> Result<T> {
        self.get_optional(key).map(|opt| opt.unwrap_or_default())
    }

    pub fn get_required<'a, T: Deserialize<'a>>(&'a self, key: Key<T>) -> Result<T> {
        if let Some(value_str) = self.1.get(&key.name().to_string()) {
            Ok(serde_json::from_str(value_str)?)
        } else {
            Err(FreeformErr::RequiredKeyNotFound(key.name().to_owned()))
        }
    }

    pub fn get_field_optional<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<Option<T>> {
        if let Some(value_str) = self.1.get(&field.to_string()) {
            Ok(Some(serde_json::from_str(value_str)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_field_or_default<'a, T: Deserialize<'a> + Default>(
        &'a self,
        field: &str,
    ) -> Result<T> {
        if let Some(value_str) = self.1.get(&field.to_string()) {
            Ok(serde_json::from_str(value_str)?)
        } else {
            Ok(Default::default())
        }
    }

    pub fn get_field_required<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<T> {
        if let Some(value_str) = self.1.get(field) {
            Ok(serde_json::from_str(value_str)?)
        } else {
            Err(FreeformErr::RequiredKeyNotFound(field.to_owned()))
        }
    }

    pub fn put<T: Serialize, B: Borrow<T>>(&mut self, key: Key<T>, data: B) -> Result<()> {
        let data_str = serde_json::to_string(data.borrow())?;
        self.1.insert(key.name().to_string(), data_str);
        Ok(())
    }

    pub fn put_field<T: Serialize, B: Borrow<T>>(&mut self, field: &str, data: B) -> Result<()> {
        let data_str = serde_json::to_string(data.borrow())?;
        self.1.insert(field.to_string(), data_str);
        Ok(())
    }

    /// Puts the data if the option is Some, else it does nothing
    pub fn put_optional<T: Serialize, O: Borrow<Option<T>>>(
        &mut self,
        key: Key<T>,
        data: O,
    ) -> Result<()> {
        if let Some(data_unwrapped) = data.borrow().as_ref() {
            self.put(key, data_unwrapped)
        } else {
            Ok(())
        }
    }

    // Possible future improvement: Trait object IsEmpty, implemented for metadata, hashmap, and Vec?
    pub fn put_nonempty<T: Serialize, V: Borrow<Vec<T>>>(
        &mut self,
        key: Key<Vec<T>>,
        data: V,
    ) -> Result<()> {
        if data.borrow().is_empty() {
            Ok(())
        } else {
            self.put(key, data.borrow())
        }
    }

    pub fn aggregate<F: IntoIterator<Item = Self>>(freeform: F) -> Option<Self> {
        freeform.into_iter().reduce(|mut acm, effects| {
            acm.extend(effects);
            acm
        })
    }
}

impl<S: SeDe> IntoIterator for Freeform<S> {
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;
    type Item = (String, String);
    fn into_iter(self) -> Self::IntoIter {
        self.1.into_iter()
    }
}

impl<S: SeDe> Extend<(String, String)> for Freeform<S> {
    fn extend<T: IntoIterator<Item = (String, String)>>(&mut self, iter: T) {
        self.1.extend(iter)
    }
}

// NOCOMMIT flip to TryFrom
impl<S: SeDe> TryFrom<HashMap<String, S::Value>> for Freeform<S> {
    type Error = S::Error;
    fn try_from(map: HashMap<String, S::Value>) -> std::result::Result<Self, S::Error> {
        let converted_map = map
            .into_iter()
            .map(|(key, val)| Ok((key, S::serialize(&val)?)))
            .collect::<std::result::Result<_, _>>()?;
        Ok(Freeform(PhantomData::<S>, converted_map))
    }
}

impl<S: SeDe> From<Freeform<S>> for HashMap<String, S::Value> {
    fn from(metadata: Freeform<S>) -> Self {
        metadata
            .1
            .into_iter()
            .map(|(key, val)| {
                (
                    key,
                    S::deserialize(val.as_str())
                        .expect("expect serialized types to be able to convert to value"),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use serde_json::{Map, Number, Value};
    use std::collections::HashMap;

    use typed_key::{typed_key, Key};

    use crate::sede::Ron;

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
    pub fn basic_test() {
        let mut freeform = <Freeform>::new();
        let ff_key: Key<Freeform> = typed_key!("ff");
        freeform.put(NUM_KEY, 343).unwrap();
        freeform.put(MAP_KEY, &test_map()).unwrap();
        freeform
            .put(ff_key, &{
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
    }
}
