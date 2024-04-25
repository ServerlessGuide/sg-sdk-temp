use crate::*;

lazy_static! {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr)]
pub struct AppVersion {
    #[serde(with = "stringify_on_num", default)]
    #[prost(int64, optional, tag = "1")]
    pub id: Option<i64>,

    #[serde(with = "stringify_on_num", default)]
    #[prost(int64, optional, tag = "2")]
    pub app_id: Option<i64>,

    #[prost(string, optional, tag = "3")]
    #[validate(length(min = 5, max = 128))]
    pub version: Option<String>,

    #[prost(string, optional, tag = "4")]
    pub create_time: Option<String>,

    #[prost(string, optional, tag = "5")]
    pub update_time: Option<String>,

    #[prost(bool, optional, tag = "6")]
    pub active: Option<bool>,

    #[prost(string, optional, tag = "7")]
    #[validate(length(max = 128))]
    pub descr: Option<String>,
}

impl AppVersion {
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

crud!(AppVersion {});

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr)]
pub struct AppCodeAndVersion {
    #[prost(string, optional, tag = "1")]
    pub version: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub code: Option<String>,

    #[prost(string, optional, tag = "3")]
    pub domain: Option<String>,
}

impl AppCodeAndVersion {
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr)]
pub struct AppInMapBuilder {
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub version: Option<String>,

    #[prost(string, repeated, tag = "3")]
    pub namespaces: Vec<String>,

    #[serde(rename = "rqNamespaces")]
    #[prost(string, repeated, tag = "4")]
    pub rq_namespaces: Vec<String>,

    #[prost(string, repeated, tag = "5")]
    pub domains: Vec<String>,
}

impl AppInMapBuilder {
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr)]
pub struct QueryAppVersions {
    #[serde(with = "stringify_on_num", default)]
    #[prost(int64, optional, tag = "1")]
    pub app_id: Option<i64>,
}

impl QueryAppVersions {
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr)]
pub struct UserWithIdSid {
    #[prost(string, optional, tag = "1")]
    pub id: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub sid: Option<String>,
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr)]
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr)]
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr, EnumFieldsConvert)]
pub struct RelId {
    #[prost(int64, optional, tag = "1")]
    pub rel_id: Option<i64>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr, EnumFieldsConvert)]
pub struct Role {
    #[serde(with = "stringify_on_num", default)]
    #[prost(int64, optional, tag = "1")]
    pub id: Option<i64>,

    #[prost(string, optional, tag = "2")]
    #[validate(length(min = 2, max = 64))]
    pub name: Option<String>,

    #[prost(enumeration = "RoleCode", optional, tag = "3")]
    #[serde(with = "stringify_enum_rolecode_option", default)]
    pub code: Option<i32>,

    #[prost(string, optional, tag = "4")]
    pub create_time: Option<String>,

    #[prost(string, optional, tag = "5")]
    pub update_time: Option<String>,

    #[prost(bool, optional, tag = "6")]
    pub active: Option<bool>,

    #[prost(string, optional, tag = "7")]
    #[validate(length(max = 128))]
    pub descr: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Reflect, prost::Enumeration)]
#[repr(i32)]
pub enum RoleCode {
    Admin = 1,
    Member = 2,
    StandBy = 3,
}

impl FromStr for RoleCode {
    type Err = ResponseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Admin" => Ok(RoleCode::Admin),
            "Member" => Ok(RoleCode::Member),
            "StandBy" => Ok(RoleCode::StandBy),
            _ => Err(gen_resp_err(ENUM_NOT_FOUND, None)),
        }
    }
}

impl ToString for RoleCode {
    fn to_string(&self) -> String {
        match self {
            RoleCode::Admin => String::from("Admin"),
            RoleCode::Member => String::from("Member"),
            RoleCode::StandBy => String::from("StandBy"),
        }
    }
}

impl RoleCode {
    pub fn lit_val_to_i32(value: &str) -> Option<i32> {
        match value {
            "Admin" => Some(RoleCode::Admin.to_i32()),
            "Member" => Some(RoleCode::Member.to_i32()),
            "StandBy" => Some(RoleCode::StandBy.to_i32()),
            _ => None,
        }
    }
}

impl RoleCode {
    pub fn to_i32(&self) -> i32 {
        match self {
            RoleCode::Admin => 1,
            RoleCode::Member => 2,
            RoleCode::StandBy => 3,
        }
    }
}

mod stringify_enum_rolecode_option {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::RoleCode;

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(value) => {
                let enum_i32 = value.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
                let enum_type = RoleCode::from_i32(enum_i32)
                    .ok_or("enum RoleCode i32 tag not valid")
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
                let enum_i32_string = RoleCode::from_str(&value)
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

mod stringify_enum_rolecode_prim {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::RoleCode;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        let enum_i32 = value.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
        let enum_type = RoleCode::from_i32(enum_i32)
            .ok_or("enum RoleCode i32 tag not valid")
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
            Ok(enum_str) => match RoleCode::from_str(&enum_str) {
                Ok(enum_type) => Ok(enum_type.to_i32().to_string().parse::<T>().map_err(|err| de::Error::custom(err.to_string()))?),
                Err(err) => Err(de::Error::custom(err.to_string())),
            },
            Err(err) => Err(err),
        }
    }
}

mod stringify_enum_rolecode_vec {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::RoleCode;

    pub fn serialize<T, S>(value: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        let mut seq = Vec::<String>::new();
        for t in value {
            let enum_i32 = t.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
            let enum_type = RoleCode::from_i32(enum_i32)
                .ok_or("enum RoleCode i32 tag not valid")
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
                    match RoleCode::from_str(&enum_str) {
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

mod stringify_enum_rolecode_map {
    use std::hash::Hash;
    use std::str::FromStr;
    use std::{collections::HashMap, fmt::Display};

    use serde::{
        de::{self},
        ser, Deserializer, Serializer,
    };
    use serde::{Deserialize, Serialize};

    use super::RoleCode;

    pub fn serialize<K, V, S>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: Eq + PartialEq + Hash + Clone + Serialize,
        V: Display,
        S: Serializer,
    {
        let mut map = HashMap::<K, String>::new();
        for (k, v) in value {
            let enum_i32 = v.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
            let enum_type = RoleCode::from_i32(enum_i32)
                .ok_or("enum RoleCode i32 tag not valid")
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
                    match RoleCode::from_str(&v) {
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
