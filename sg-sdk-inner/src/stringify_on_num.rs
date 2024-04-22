use std::fmt::Display;
use std::str::FromStr;

use serde::{
    de::{self},
    Deserialize, Deserializer, Serializer,
};

pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(value) => serializer.collect_str(value),
    }
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    match Option::<String>::deserialize(deserializer)? {
        None => Ok(None),
        Some(value) => match value.parse::<T>() {
            Ok(t) => Ok(Some(t)),
            Err(err) => Err(de::Error::custom(err.to_string())),
        },
    }
}
