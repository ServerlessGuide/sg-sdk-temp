use crate::*;

lazy_static! {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr, EnumFieldsConvert)]
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

crud!(AppVersion {});

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr, EnumFieldsConvert)]
pub struct AppCodeAndVersion {
    #[prost(string, optional, tag = "1")]
    pub version: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub code: Option<String>,

    #[prost(string, optional, tag = "3")]
    pub domain: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr, EnumFieldsConvert)]
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr, EnumFieldsConvert)]
pub struct QueryAppVersions {
    #[serde(with = "stringify_on_num", default)]
    #[prost(int64, optional, tag = "1")]
    pub app_id: Option<i64>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr, EnumFieldsConvert)]
pub struct UserWithIdSid {
    #[prost(string, optional, tag = "1")]
    pub id: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub sid: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr, EnumFieldsConvert)]
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message, Dapr, EnumFieldsConvert)]
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

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Reflect, prost::Enumeration, EnumGenerate)]
#[repr(i32)]
pub enum RoleCode {
    Admin = 1,
    Member = 2,
    StandBy = 3,
}
