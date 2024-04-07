use super::SerdeScheme;

use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::OnceLock;

use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug)]
pub struct TypedSord<T, S: SerdeScheme> {
    pub(crate) se: OnceLock<Result<String, S::Error>>,
    pub(crate) de: OnceLock<Result<T, S::Error>>,
}

impl<T: DeserializeOwned + Serialize, S: SerdeScheme> TypedSord<T, S> {
    pub fn from_se<K>(se: K) -> Self
    where
        K: ToString,
    {
        let se: String = se.to_string();
        TypedSord {
            se: OnceLock::from(Ok(se)),
            de: OnceLock::new(),
        }
    }

    pub fn from_de(de: T) -> Self {
        TypedSord {
            se: OnceLock::new(),
            de: OnceLock::from(Ok(de)),
        }
    }

    pub fn de(&self) -> Result<&T, &S::Error> {
        let se = &self.se;
        self.de
            .get_or_init(|| {
                let se = se
                    .get()
                    .expect("should not be possible for both se and de to be uninitialized")
                    .as_ref()
                    .expect("should not be possible to initialize se as an error");
                S::deserialize(se)
            })
            .as_ref()
    }

    pub fn se(&self) -> Result<&str, &S::Error> {
        let de = &self.de;
        let m = self
            .se
            .get_or_init(|| {
                let de = de
                    .get()
                    .expect("should not be possible for both de and se to be uninitialized")
                    .as_ref()
                    .expect("should not be possible to initialize de as an error");
                S::serialize(de)
            })
            .as_ref()
            .map(|cow| cow.borrow());
        m
    }
}
