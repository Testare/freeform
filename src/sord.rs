use super::{SerdeScheme, TypedSord};

use std::any::Any;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::{Arc, OnceLock};

use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone, Debug)]
#[allow(clippy::type_complexity)]
pub struct Sord<S: SerdeScheme> {
    se: OnceLock<Result<String, SordError<S>>>,
    de: OnceLock<Result<Arc<dyn Any + 'static + Send + Sync>, SordError<S>>>,
    se_fn: Option<unsafe fn(&Arc<dyn Any + 'static + Send + Sync>) -> Result<String, S::Error>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SordError<S: SerdeScheme> {
    SeDeError(S::Error),
    WrongTypeError,
}

impl<S: SerdeScheme> Sord<S> {
    pub fn from_de_ref<T: 'static + Send + Sync + Serialize>(de: &T) -> Result<Self, SordError<S>> {
        let se = S::serialize::<T>(de).map_err(SordError::SeDeError)?;
        Ok(Sord {
            se: OnceLock::from(Ok(se)),
            de: OnceLock::new(),
            se_fn: None,
        })
    }

    pub fn from_de<T: Serialize + 'static + Send + Sync>(de: T) -> Self {
        Sord {
            se: OnceLock::new(),
            de: OnceLock::from(Ok(Arc::new(de) as Arc<dyn Any + 'static + Send + Sync>)),
            se_fn: Some(S::serialize_as_any::<T>),
        }
    }

    pub fn from_se<T: ToString>(se: T) -> Self {
        Sord {
            se: OnceLock::from(Ok(se.to_string())),
            de: OnceLock::new(),
            se_fn: None,
        }
    }

    pub fn from_value(value: &S::Value) -> Result<Self, SordError<S>> {
        let se = S::serialize::<S::Value>(value).map_err(SordError::SeDeError)?;
        Ok(Sord {
            se: OnceLock::from(Ok(se.to_string())),
            de: OnceLock::new(),
            se_fn: None,
        })
    }

    pub fn de<T: DeserializeOwned + 'static + Send + Sync>(&self) -> Result<&T, &SordError<S>> {
        let se = &self.se;
        self.de
            .get_or_init(|| {
                let se = se
                    .get()
                    .expect("should not be possible for both se and de to be uninitialized")
                    .as_ref()
                    .expect("should not be possible to initialize se as an error");
                let deserialize: T = S::deserialize(se).map_err(SordError::SeDeError)?;
                Ok(Arc::new(deserialize))
            })
            .as_ref()
            .and_then(|de| de.downcast_ref::<T>().ok_or(&SordError::WrongTypeError))
    }

    pub fn se<T: Serialize + 'static>(&self) -> Result<&str, &SordError<S>> {
        let de = &self.de;
        self.se
            .get_or_init(|| {
                let de = de
                    .get()
                    .expect("should not be possible for both de and se to be uninitialized")
                    .as_ref()
                    .expect("should not be possible to initialize de as an error")
                    .downcast_ref::<T>()
                    .ok_or(SordError::<S>::WrongTypeError)?;
                S::serialize(de).map_err(SordError::SeDeError)
            })
            .as_ref()
            .map(|cow| cow.borrow())
    }

    pub fn value(&self) -> Result<S::Value, SordError<S>> {
        if let Some(Ok(se)) = self.se.get() {
            S::deserialize(se.as_str()).map_err(SordError::SeDeError)
        } else if let Some(Ok(de)) = self.de.get() {
            let se_str = unsafe {
                // SAEFTY: de is only initialized without se being initialized with de,
                // and this function is only populated in that case
                self.se_fn
                    .expect("se_fn should be created initialized with de")(de)
                .map_err(SordError::SeDeError)?
            };
            S::deserialize(se_str.as_str()).map_err(SordError::SeDeError)
        } else {
            unreachable!("Se or De should be the initial value")
        }
    }

    pub fn typed<T: Clone + Serialize + DeserializeOwned + 'static + Send + Sync>(
        self,
    ) -> Option<TypedSord<T, S>> {
        let Sord { se, de, se_fn: _ } = self;

        let se = if let Some(se) = se.into_inner() {
            match se {
                Ok(se) => OnceLock::from(Ok(se)),
                Err(SordError::WrongTypeError) => return None,
                Err(SordError::SeDeError(err)) => OnceLock::from(Err(err)),
            }
        } else {
            OnceLock::new()
        };

        let de = if let Some(de) = de.into_inner() {
            match de {
                Ok(de) => {
                    if let Ok(de) = de.downcast::<T>() {
                        OnceLock::from(Ok(Arc::<T>::unwrap_or_clone(de)))
                    } else {
                        return None;
                    }
                }
                Err(SordError::WrongTypeError) => return None,
                Err(SordError::SeDeError(err)) => OnceLock::from(Err(err)),
            }
        } else {
            OnceLock::new()
        };
        Some(TypedSord { se, de })
    }
}

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use super::*;
    use crate::Json;

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct TestySeDe {
        name: String,
        count: u8,
    }

    const SERIALIZED: &str = "{\"name\":\"Test\",\"count\":8}";

    fn test_obj() -> TestySeDe {
        TestySeDe {
            name: "Test".to_string(),
            count: 8,
        }
    }

    #[test]
    fn sord_testing_from_de() {
        let sord = Sord::<Json>::from_de(test_obj());
        assert_eq!(
            &test_obj(),
            sord.de::<TestySeDe>().expect("deserialized should exist")
        );
        assert_eq!(
            SERIALIZED,
            sord.se::<TestySeDe>()
                .expect("should serialize successfully")
        );
        assert!(matches!(
            sord.de::<String>(),
            Err(&SordError::WrongTypeError)
        ));
    }

    #[test]
    fn sord_testing_from_se() {
        let sord = Sord::<Json>::from_se(SERIALIZED);
        assert_eq!(
            SERIALIZED,
            sord.se::<TestySeDe>().expect("serialized should exist")
        );
        assert_eq!(
            &test_obj(),
            sord.de::<TestySeDe>()
                .expect("should deserialize succcessfully")
        );
        assert!(matches!(
            sord.de::<String>(),
            Err(&SordError::WrongTypeError)
        ));
    }
}
