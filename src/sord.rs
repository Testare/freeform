use super::{SeDe, SeDeAny, TypedSord};
// Experimental Mod

use std::any::Any;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::OnceLock;

use serde::de::DeserializeOwned;
use serde::Serialize;

impl<T: Serialize + DeserializeOwned> SeDeAny for T {}

#[derive(Debug)]
#[allow(clippy::type_complexity)]
pub struct Sord<S: SeDe> {
    se: OnceLock<Result<String, SordError<S::Error>>>,
    de: OnceLock<Result<Box<dyn Any>, SordError<S::Error>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SordError<E> {
    SeDeError(E),
    WrongTypeError,
}

impl<S: SeDe> Sord<S> {
    pub fn from_de<T: 'static>(de: T) -> Self {
        Sord {
            se: OnceLock::new(),
            de: OnceLock::from(Ok(Box::new(de) as Box<dyn Any>)),
        }
    }

    pub fn from_se<T: ToString>(se: T) -> Self {
        Sord {
            se: OnceLock::from(Ok(se.to_string())),
            de: OnceLock::new(),
        }
    }

    pub fn de<T: DeserializeOwned + 'static>(&self) -> Result<&T, &SordError<S::Error>> {
        let se = &self.se;
        self.de
            .get_or_init(|| {
                let se = se
                    .get()
                    .expect("should not be possible for both se and de to be uninitialized")
                    .as_ref()
                    .expect("should not be possible to initialize se as an error");
                let deserialize: T = S::deserialize(se).map_err(SordError::SeDeError)?;
                Ok(Box::new(deserialize))
            })
            .as_ref()
            .and_then(|de| de.downcast_ref::<T>().ok_or(&SordError::WrongTypeError))
    }

    pub fn se<T: Serialize + 'static>(&self) -> Result<&str, &SordError<S::Error>> {
        let de = &self.de;
        self.se
            .get_or_init(|| {
                let de = de
                    .get()
                    .expect("should not be possible for both de and se to be uninitialized")
                    .as_ref()
                    .expect("should not be possible to initialize de as an error")
                    .downcast_ref::<T>()
                    .ok_or(SordError::<S::Error>::WrongTypeError)?;
                S::serialize(de).map_err(SordError::SeDeError)
            })
            .as_ref()
            .map(|cow| cow.borrow())
    }

    pub fn typed<T: Serialize + DeserializeOwned + 'static>(self) -> Option<TypedSord<T, S>> {
        let Sord { se, de } = self;

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
                        OnceLock::from(Ok(*de))
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
