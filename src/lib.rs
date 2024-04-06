use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use typed_key::Key;

pub use typed_key::typed_key;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(from = "HashMap<String, Value>", into = "HashMap<String, Value>")]
pub struct Freeform(HashMap<String, String>);

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

impl Freeform {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn new() -> Self {
        Freeform::default()
    }

    pub fn get_optional<'a, T: Deserialize<'a>>(&'a self, key: Key<T>) -> Result<Option<T>> {
        if let Some(value_str) = self.0.get(&key.name().to_string()) {
            Ok(Some(serde_json::from_str(value_str)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_or_default<'a, T: Deserialize<'a> + Default>(&'a self, key: Key<T>) -> Result<T> {
        self.get_optional(key).map(|opt| opt.unwrap_or_default())
    }

    pub fn get_required<'a, T: Deserialize<'a>>(&'a self, key: Key<T>) -> Result<T> {
        if let Some(value_str) = self.0.get(&key.name().to_string()) {
            Ok(serde_json::from_str(value_str)?)
        } else {
            Err(FreeformErr::RequiredKeyNotFound(key.name().to_owned()))
        }
    }

    pub fn get_field_optional<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<Option<T>> {
        if let Some(value_str) = self.0.get(&field.to_string()) {
            Ok(Some(serde_json::from_str(value_str)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_field_or_default<'a, T: Deserialize<'a> + Default>(
        &'a self,
        field: &str,
    ) -> Result<T> {
        if let Some(value_str) = self.0.get(&field.to_string()) {
            Ok(serde_json::from_str(value_str)?)
        } else {
            Ok(Default::default())
        }
    }

    pub fn get_field_required<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<T> {
        if let Some(value_str) = self.0.get(field) {
            Ok(serde_json::from_str(value_str)?)
        } else {
            Err(FreeformErr::RequiredKeyNotFound(field.to_owned()))
        }
    }

    pub fn put<T: Serialize, B: Borrow<T>>(&mut self, key: Key<T>, data: B) -> Result<()> {
        let data_str = serde_json::to_string(data.borrow())?;
        self.0.insert(key.name().to_string(), data_str);
        Ok(())
    }

    pub fn put_field<T: Serialize, B: Borrow<T>>(&mut self, field: &str, data: B) -> Result<()> {
        let data_str = serde_json::to_string(data.borrow())?;
        self.0.insert(field.to_string(), data_str);
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

    pub fn aggregate<M: IntoIterator<Item = Freeform>>(metadata: M) -> Option<Self> {
        metadata.into_iter().reduce(|mut acm, effects| {
            acm.extend(effects);
            acm
        })
    }
}

impl IntoIterator for Freeform {
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;
    type Item = (String, String);
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<(String, String)> for Freeform {
    fn extend<T: IntoIterator<Item = (String, String)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl From<HashMap<String, Value>> for Freeform {
    fn from(map: HashMap<String, Value>) -> Self {
        Freeform(
            map.into_iter()
                .map(|(key, val)| (key, val.to_string()))
                .collect(),
        )
    }
}

impl From<Freeform> for HashMap<String, Value> {
    fn from(metadata: Freeform) -> Self {
        metadata
            .0
            .into_iter()
            .map(|(key, val)| {
                (
                    key,
                    serde_json::from_str(&val).expect("Freeform should not store escaped strings"),
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

    use super::Freeform;

    #[test]
    pub fn basic_test() {
        let mut freeform = Freeform::new();
        let u_key: Key<usize> = typed_key!("u_key");
        let inner_hashmap_key: Key<HashMap<String, String>> = typed_key!("inner_hashmap");
        let inner_freeform_key: Key<Freeform> = typed_key!("inner_freeform");
        let mut hashmap: HashMap<String, String> = HashMap::new();
        hashmap.insert("what".to_string(), "hey".to_string());
        hashmap.insert("whata".to_string(), "hey".to_string());
        hashmap.insert("foo".to_string(), "bar".to_string());
        freeform.put(u_key, 343).unwrap();
        freeform.put(inner_hashmap_key, &hashmap).unwrap();
        freeform
            .put(inner_freeform_key, &{
                let mut metadata = Freeform::new();
                metadata.put(u_key, 143).unwrap();
                metadata
            })
            .unwrap();

        let mut inner_freeform_map = Map::new();
        inner_freeform_map.insert(u_key.name().to_string(), Value::Number(Number::from(143)));

        let mut hashmap_value = Map::new();
        hashmap_value.insert("what".to_string(), Value::String("hey".to_string()));
        hashmap_value.insert("whata".to_string(), Value::String("hey".to_string()));
        hashmap_value.insert("foo".to_string(), Value::String("bar".to_string()));

        let mut expected_map = Map::new();
        expected_map.insert(u_key.name().to_string(), Value::Number(Number::from(343)));
        expected_map.insert(
            inner_hashmap_key.name().to_string(),
            Value::Object(hashmap_value),
        );
        expected_map.insert(
            inner_freeform_key.name().to_string(),
            Value::Object(inner_freeform_map),
        );

        let result = serde_json::to_value(&freeform).unwrap();

        assert_eq!(Value::Object(expected_map), result);
    }
}
