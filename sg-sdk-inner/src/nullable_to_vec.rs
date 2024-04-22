use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<T, S>(value: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    if value.is_empty() {
        serializer.serialize_none()
    } else {
        value.serialize(serializer)
    }
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    let j = Option::<Vec<T>>::deserialize(deserializer)?;
    match j {
        None => Ok(Vec::new()),
        Some(j_some) => Ok(j_some),
    }
}
