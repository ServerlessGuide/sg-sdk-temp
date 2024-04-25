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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr)]
pub struct RelId {
    #[prost(int64, optional, tag = "1")]
    pub rel_id: Option<i64>,
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
