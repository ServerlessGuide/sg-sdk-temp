use crate::*;

lazy_static! {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct DBStorageModel {
    #[serde(with = "stringify_on_num", default)]
    #[prost(int64, optional, tag = "1")]
    pub id: Option<i64>,

    #[serde(with = "stringify_on_num", default)]
    #[prost(int64, optional, tag = "2")]
    pub db_user_id: Option<i64>,

    #[serde(with = "stringify_on_num", default)]
    #[prost(int64, optional, tag = "3")]
    pub db_database_id: Option<i64>,

    #[prost(string, optional, tag = "4")]
    #[validate(length(max = 64))]
    pub name: Option<String>,

    #[prost(string, optional, tag = "5")]
    #[validate(length(max = 256))]
    pub comment: Option<String>,

    #[prost(string, optional, tag = "6")]
    pub create_time: Option<String>,

    #[prost(string, optional, tag = "7")]
    pub update_time: Option<String>,

    #[prost(bool, optional, tag = "8")]
    pub active: Option<bool>,

    #[prost(string, optional, tag = "9")]
    #[validate(length(max = 128))]
    pub descr: Option<String>,
}

impl DBStorageModel {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        info!("1111");
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for DBStorageModel {}

crud!(DBStorageModel {}, "db_sm");

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct UserWithIdSid {
    #[prost(string, optional, tag = "1")]
    pub id: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub sid: Option<String>,

    #[prost(string, optional, tag = "3")]
    pub app_code: Option<String>,

    #[prost(string, optional, tag = "4")]
    pub app_version: Option<String>,

    #[prost(int64, optional, tag = "5")]
    pub snow_id: Option<i64>,

    #[prost(int64, optional, tag = "6")]
    pub db_database_id: Option<i64>,

    #[prost(string, optional, tag = "7")]
    pub sm_name: Option<String>,

    #[prost(enumeration = "RequireType", optional, tag = "8")]
    #[serde(with = "stringify_enum_requiretype_option", default)]
    pub require_type: Option<i32>,

    #[prost(string, optional, tag = "9")]
    pub require_name: Option<String>,

    #[prost(string, optional, tag = "10")]
    pub require_instance_name: Option<String>,

    #[prost(string, optional, tag = "11")]
    pub db_database_name: Option<String>,

    #[prost(string, optional, tag = "12")]
    pub db_user_name: Option<String>,

    #[prost(string, optional, tag = "13")]
    pub db_user_password: Option<String>,
}

impl UserWithIdSid {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for UserWithIdSid {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct Require {
    #[prost(enumeration = "RequireType", optional, tag = "1")]
    #[serde(with = "stringify_enum_requiretype_option", default)]
    pub require_type: Option<i32>,

    #[prost(string, optional, tag = "2")]
    #[validate(length(max = 64))]
    pub require_name: Option<String>,

    #[prost(string, optional, tag = "3")]
    pub instance_name: Option<String>,

    #[prost(string, optional, tag = "4")]
    pub app_version: Option<String>,

    #[prost(string, optional, tag = "5")]
    pub require_version: Option<String>,
}

impl Require {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = ["require_type"].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                "require_type" => Ok((true, RequireType::lit_val_to_i32(f_value))),
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for Require {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Reflect, prost::Enumeration)]
#[repr(i32)]
pub enum RequireType {
    SQL = 1,
    NoSql = 2,
    Graph = 3,
    KV = 4,
    OSS = 5,
    TimeSeries = 6,
    MQ = 7,
    SearchEngine = 8,
    Other = 9,
}

impl FromStr for RequireType {
    type Err = ResponseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SQL" => Ok(RequireType::SQL),
            "NoSql" => Ok(RequireType::NoSql),
            "Graph" => Ok(RequireType::Graph),
            "KV" => Ok(RequireType::KV),
            "OSS" => Ok(RequireType::OSS),
            "TimeSeries" => Ok(RequireType::TimeSeries),
            "MQ" => Ok(RequireType::MQ),
            "SearchEngine" => Ok(RequireType::SearchEngine),
            "Other" => Ok(RequireType::Other),
            _ => Err(gen_resp_err(ENUM_NOT_FOUND, None)),
        }
    }
}

impl ToString for RequireType {
    fn to_string(&self) -> String {
        match self {
            RequireType::SQL => String::from("SQL"),
            RequireType::NoSql => String::from("NoSql"),
            RequireType::Graph => String::from("Graph"),
            RequireType::KV => String::from("KV"),
            RequireType::OSS => String::from("OSS"),
            RequireType::TimeSeries => String::from("TimeSeries"),
            RequireType::MQ => String::from("MQ"),
            RequireType::SearchEngine => String::from("SearchEngine"),
            RequireType::Other => String::from("Other"),
        }
    }
}

impl RequireType {
    pub fn lit_val_to_i32(value: &str) -> Option<i32> {
        match value {
            "SQL" => Some(RequireType::SQL.to_i32()),
            "NoSql" => Some(RequireType::NoSql.to_i32()),
            "Graph" => Some(RequireType::Graph.to_i32()),
            "KV" => Some(RequireType::KV.to_i32()),
            "OSS" => Some(RequireType::OSS.to_i32()),
            "TimeSeries" => Some(RequireType::TimeSeries.to_i32()),
            "MQ" => Some(RequireType::MQ.to_i32()),
            "SearchEngine" => Some(RequireType::SearchEngine.to_i32()),
            "Other" => Some(RequireType::Other.to_i32()),
            _ => None,
        }
    }
}

impl RequireType {
    pub fn to_i32(&self) -> i32 {
        match self {
            RequireType::SQL => 1,
            RequireType::NoSql => 2,
            RequireType::Graph => 3,
            RequireType::KV => 4,
            RequireType::OSS => 5,
            RequireType::TimeSeries => 6,
            RequireType::MQ => 7,
            RequireType::SearchEngine => 8,
            RequireType::Other => 9,
        }
    }
}

mod stringify_enum_requiretype_option {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::RequireType;

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(value) => {
                let enum_i32 = value.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
                let enum_type = RequireType::from_i32(enum_i32)
                    .ok_or("enum RequireType i32 tag not valid")
                    .map_err(|err| ser::Error::custom(err.to_string()))?;
                serializer.collect_str(&enum_type.to_string())
            }
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
            Some(value) => {
                let enum_i32_string = RequireType::from_str(&value)
                    .map_err(|err| de::Error::custom(err.to_string()))?
                    .to_i32()
                    .to_string();
                match enum_i32_string.parse::<T>() {
                    Ok(t) => Ok(Some(t)),
                    Err(err) => Err(de::Error::custom(err.to_string())),
                }
            }
        }
    }
}

mod stringify_enum_requiretype_prim {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::RequireType;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        let enum_i32 = value.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
        let enum_type = RequireType::from_i32(enum_i32)
            .ok_or("enum RequireType i32 tag not valid")
            .map_err(|err| ser::Error::custom(err.to_string()))?;
        serializer.collect_str(&enum_type.to_string())
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(enum_str) => match RequireType::from_str(&enum_str) {
                Ok(enum_type) => Ok(enum_type.to_i32().to_string().parse::<T>().map_err(|err| de::Error::custom(err.to_string()))?),
                Err(err) => Err(de::Error::custom(err.to_string())),
            },
            Err(err) => Err(err),
        }
    }
}

mod stringify_enum_requiretype_vec {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::RequireType;

    pub fn serialize<T, S>(value: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        let mut seq = Vec::<String>::new();
        for t in value {
            let enum_i32 = t.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
            let enum_type = RequireType::from_i32(enum_i32)
                .ok_or("enum RequireType i32 tag not valid")
                .map_err(|err| ser::Error::custom(err.to_string()))?;
            seq.push(enum_type.to_string())
        }
        serializer.collect_seq(seq)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match Vec::<String>::deserialize(deserializer) {
            Ok(enum_strs) => {
                let mut seq = Vec::<T>::new();
                for enum_str in enum_strs {
                    match RequireType::from_str(&enum_str) {
                        Ok(enum_type) => {
                            let act = enum_type.to_i32().to_string().parse::<T>().map_err(|err| de::Error::custom(err.to_string()))?;
                            seq.push(act);
                        }
                        Err(err) => return Err(de::Error::custom(err.to_string())),
                    }
                }
                Ok(seq)
            }
            Err(err) => Err(err),
        }
    }
}

mod stringify_enum_requiretype_map {
    use std::hash::Hash;
    use std::str::FromStr;
    use std::{collections::HashMap, fmt::Display};

    use serde::{
        de::{self},
        ser, Deserializer, Serializer,
    };
    use serde::{Deserialize, Serialize};

    use super::RequireType;

    pub fn serialize<K, V, S>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: Eq + PartialEq + Hash + Clone + Serialize,
        V: Display,
        S: Serializer,
    {
        let mut map = HashMap::<K, String>::new();
        for (k, v) in value {
            let enum_i32 = v.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
            let enum_type = RequireType::from_i32(enum_i32)
                .ok_or("enum RequireType i32 tag not valid")
                .map_err(|err| ser::Error::custom(err.to_string()))?;
            map.insert(k.clone(), enum_type.to_string());
        }
        serializer.collect_map(map)
    }

    pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
    where
        V: FromStr,
        V::Err: Display,
        D: Deserializer<'de>,
        K: Deserialize<'de> + Eq + Hash,
    {
        match HashMap::<K, String>::deserialize(deserializer) {
            Ok(enum_strs) => {
                let mut map = HashMap::<K, V>::new();
                for (k, v) in enum_strs {
                    match RequireType::from_str(&v) {
                        Ok(enum_type) => {
                            let act = enum_type.to_i32().to_string().parse::<V>().map_err(|err| de::Error::custom(err.to_string()))?;
                            map.insert(k, act);
                        }
                        Err(err) => return Err(de::Error::custom(err.to_string())),
                    }
                }
                Ok(map)
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct IdRes {
    #[prost(int64, optional, tag = "1")]
    #[validate(range(min = 101000, max = 101000))]
    pub code: Option<i64>,

    #[prost(string, optional, tag = "2")]
    #[validate(contains = "success")]
    #[validate(length(min = 7, max = 7))]
    pub message: Option<String>,

    #[prost(int64, optional, tag = "3")]
    pub result: Option<i64>,
}

impl IdRes {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for IdRes {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct BulkIdRes {
    #[prost(int64, optional, tag = "1")]
    #[validate(range(min = 101000, max = 101000))]
    pub code: Option<i64>,

    #[prost(string, optional, tag = "2")]
    #[validate(contains = "success")]
    #[validate(length(min = 7, max = 7))]
    pub message: Option<String>,

    #[prost(int64, repeated, tag = "3")]
    pub result: Vec<i64>,
}

impl BulkIdRes {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for BulkIdRes {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct RelId {
    #[prost(int64, optional, tag = "1")]
    pub rel_id: Option<i64>,

    #[prost(string, optional, tag = "2")]
    pub app_code: Option<String>,

    #[prost(string, optional, tag = "3")]
    pub app_version: Option<String>,

    #[prost(string, optional, tag = "4")]
    pub sm_name: Option<String>,

    #[prost(int64, optional, tag = "5")]
    pub db_database_id: Option<i64>,

    #[prost(string, optional, tag = "6")]
    pub require_instance_name: Option<String>,

    #[prost(string, optional, tag = "7")]
    pub db_database_name: Option<String>,
}

impl RelId {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for RelId {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct StorageModelInfo {
    #[prost(int64, optional, tag = "1")]
    #[serde(with = "stringify_on_num", default)]
    pub id: Option<i64>,

    #[prost(string, optional, tag = "9")]
    pub storage_model_name: Option<String>,

    #[prost(bool, optional, tag = "11")]
    pub primary_key_enable: Option<bool>,

    #[prost(message, optional, tag = "12")]
    pub primary_key_info: Option<PrimaryKeyInfo>,

    #[prost(bool, optional, tag = "13")]
    pub foreign_key_enable: Option<bool>,

    #[prost(bool, optional, tag = "15")]
    pub unique_key_enable: Option<bool>,

    #[prost(string, optional, tag = "17")]
    pub comment: Option<String>,
}

impl StorageModelInfo {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for StorageModelInfo {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct SMDSL {
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub namespace: Option<String>,

    #[prost(string, optional, tag = "3")]
    #[serde(rename = "appName")]
    pub app_name: Option<String>,

    #[prost(string, optional, tag = "5")]
    #[serde(rename = "dataBaseName")]
    pub database_name: Option<String>,

    #[prost(string, optional, tag = "8")]
    pub schema: Option<String>,

    #[serde(rename = "storageModelName")]
    #[prost(string, optional, tag = "9")]
    pub storage_model_name: Option<String>,

    #[serde(rename = "initDataBase")]
    #[prost(string, optional, tag = "10")]
    pub init_database: Option<String>,

    #[serde(rename = "primaryKeyEnable")]
    #[prost(bool, optional, tag = "11")]
    pub primary_key_enable: Option<bool>,

    #[serde(rename = "primaryKeyEnable")]
    #[prost(message, optional, tag = "12")]
    pub primary_key_info: Option<PrimaryKeyInfo>,

    #[serde(rename = "foreignKeyEnable")]
    #[prost(bool, optional, tag = "13")]
    pub foreign_key_enable: Option<bool>,

    #[serde(rename = "uniqueKeyEnable")]
    #[prost(bool, optional, tag = "15")]
    pub unique_key_enable: Option<bool>,

    #[prost(string, optional, tag = "17")]
    pub comment: Option<String>,
}

impl SMDSL {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for SMDSL {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct PrimaryKeyInfo {
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,

    #[serde(rename = "fieldNames")]
    #[prost(string, repeated, tag = "2")]
    #[serde(with = "nullable_to_vec", default)]
    pub field_names: Vec<String>,

    #[prost(string, optional, tag = "3")]
    pub comment: Option<String>,
}

impl PrimaryKeyInfo {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for PrimaryKeyInfo {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct ObjectList {
    #[prost(message, repeated, tag = "1")]
    #[serde(with = "nullable_to_vec")]
    pub Contents: Vec<ObjectListContent>,

    #[prost(string, optional, tag = "2")]
    pub Prefix: Option<String>,
}

impl ObjectList {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for ObjectList {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct ObjectListContent {
    #[prost(string, optional, tag = "1")]
    pub Key: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub ETag: Option<String>,
}

impl ObjectListContent {
    pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
        let enum_flds = [].to_vec();
        if enum_flds.contains(&f_name) {
            match f_name {
                _ => Err(Box::new(gen_resp_err(ENUM_NOT_FOUND, Some(format!("enum field {} not found", f_name))))),
            }
        } else {
            Ok((false, None))
        }
    }
}

impl DaprBody for ObjectListContent {}
