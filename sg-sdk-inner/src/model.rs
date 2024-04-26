use bevy_reflect::{GetField, Reflect};

use dapr::dapr::dapr::proto::{common::v1::*, runtime::v1::*};
use downcast_rs::{impl_downcast, Downcast};
use hyper::Method;
use hyper::Response;
use prost::Message;
use serde::{Deserialize, Serialize};
use sg_sdk_macro::Model;
use sg_sdk_macro::ModelValidate;
use std::{collections::HashMap, fmt::Debug, str::FromStr};
use validator::Validate;
use validator_derive::Validate;

use crate::{inner_biz_result::*, traits::*, util::*, HttpResult, *};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect)]
pub struct Res<T> {
    pub code: u32,
    pub message: String,
    pub result: Option<T>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Validate, Default, Clone)]
pub struct PageInfo {
    pub total_no: u32,
    pub page_size: u32,
    pub current_page_no: u32,
    pub total_page_no: u32,
}

impl PageInfo {
    pub fn to_message_page(self) -> PageInfoMessage {
        PageInfoMessage {
            total_data: self.total_no,
            current_page_no: self.current_page_no,
            total_pages: self.total_page_no,
            page_size: self.page_size,
        }
    }

    // pub fn to_message(self) -> T {
    //     PageWrapperMessage {
    //         total_data: self.total_no,
    //         current_page_no: self.current_page_no,
    //         total_pages: self.total_page_no,
    //         data: self
    //             .data
    //             .iter()
    //             .map(|it| prost_types::Any {
    //                 type_url: "".to_string(),
    //                 value: T::encode_to_vec(it),
    //             })
    //             .collect(),
    //     }
    // }
}

#[derive(PartialEq, Eq, Debug, Validate, Clone, Default)]
pub struct Params {
    pub header: HashMap<String, String>,
    pub path_param: HashMap<u8, String>,
    pub query_param: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub uri: String,
    pub if_info: IfInfo,
}

#[derive(PartialEq, Eq, Debug, Validate, Clone, Default)]
pub struct IfInfo {
    pub action: Action,
    pub bulk_input: bool,
    pub bulk_output: bool,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Reflect)]
pub enum ParamFrom {
    Header,
    Path,
    Query,
    Body,
}

impl FromStr for ParamFrom {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Header" => Ok(ParamFrom::Header),
            "Path" => Ok(ParamFrom::Path),
            "Query" => Ok(ParamFrom::Query),
            "Body" => Ok(ParamFrom::Body),
            _ => Err(err(ENUM_NOT_FOUND)),
        }
    }
}

impl ToString for ParamFrom {
    fn to_string(&self) -> String {
        match self {
            ParamFrom::Header => String::from("Header"),
            ParamFrom::Path => String::from("Path"),
            ParamFrom::Query => String::from("Query"),
            ParamFrom::Body => String::from("Body"),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Reflect)]
pub enum ParamType {
    Bool,
    String,
    Number,
    HashMap,
    Vec,
}

impl FromStr for ParamType {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bool" => Ok(ParamType::Bool),
            "String" => Ok(ParamType::String),
            "Number" => Ok(ParamType::Number),
            "HashMap" => Ok(ParamType::HashMap),
            "Vec" => Ok(ParamType::Vec),
            _ => Err(err(ENUM_NOT_FOUND)),
        }
    }
}

impl ToString for ParamType {
    fn to_string(&self) -> String {
        match self {
            ParamType::Bool => String::from("Bool"),
            ParamType::String => String::from("String"),
            ParamType::Number => String::from("Number"),
            ParamType::HashMap => String::from("HashMap"),
            ParamType::Vec => String::from("Vec"),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Reflect, Hash, prost::Enumeration)]
#[repr(i32)]
pub enum Action {
    Query = 1,
    Update = 2,
    Delete = 3,
    Insert = 4,
    TX = 5,
    Function = 6,
}

mod stringify_enum_action_option {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::Action;

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(value) => {
                let enum_i32 = value.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
                let enum_type = Action::from_i32(enum_i32)
                    .ok_or("enum Action i32 tag not valid")
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
                let enum_i32_string = Action::from_str(&value).map_err(|err| de::Error::custom(err.to_string()))?.to_i32().to_string();
                match enum_i32_string.parse::<T>() {
                    Ok(t) => Ok(Some(t)),
                    Err(err) => Err(de::Error::custom(err.to_string())),
                }
            }
        }
    }
}

mod stringify_enum_action_prim {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::Action;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        let action_i32 = value.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
        let action = Action::from_i32(action_i32)
            .ok_or("action i32 tag not valid")
            .map_err(|err| ser::Error::custom(err.to_string()))?;
        serializer.collect_str(&action.to_string())
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(action_str) => match Action::from_str(&action_str) {
                Ok(action) => Ok(action.to_i32().to_string().parse::<T>().map_err(|err| de::Error::custom(err.to_string()))?),
                Err(err) => Err(de::Error::custom(err.to_string())),
            },
            Err(err) => Err(err),
        }
    }
}

mod stringify_enum_action_vec {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::Action;

    pub fn serialize<T, S>(value: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        let mut seq = Vec::<String>::new();
        for t in value {
            let action_i32 = t.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
            let action = Action::from_i32(action_i32)
                .ok_or("action i32 tag not valid")
                .map_err(|err| ser::Error::custom(err.to_string()))?;
            seq.push(action.to_string())
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
            Ok(action_strs) => {
                let mut seq = Vec::<T>::new();
                for action_str in action_strs {
                    match Action::from_str(&action_str) {
                        Ok(action) => {
                            let act = action.to_i32().to_string().parse::<T>().map_err(|err| de::Error::custom(err.to_string()))?;
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

mod stringify_enum_action_map {
    use std::hash::Hash;
    use std::str::FromStr;
    use std::{collections::HashMap, fmt::Display};

    use serde::{
        de::{self},
        ser, Deserializer, Serializer,
    };
    use serde::{Deserialize, Serialize};

    use super::Action;

    pub fn serialize<K, V, S>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: Eq + PartialEq + Hash + Clone + Serialize,
        V: Display,
        S: Serializer,
    {
        let mut map = HashMap::<K, String>::new();
        for (k, v) in value {
            let action_i32 = v.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
            let action = Action::from_i32(action_i32)
                .ok_or("action i32 tag not valid")
                .map_err(|err| ser::Error::custom(err.to_string()))?;
            map.insert(k.clone(), action.to_string());
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
            Ok(action_strs) => {
                let mut map = HashMap::<K, V>::new();
                for (k, v) in action_strs {
                    match Action::from_str(&v) {
                        Ok(action) => {
                            let act = action.to_i32().to_string().parse::<V>().map_err(|err| de::Error::custom(err.to_string()))?;
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

impl Action {
    pub fn to_i32(&self) -> i32 {
        match self {
            Action::Query => 1,
            Action::Update => 2,
            Action::Delete => 3,
            Action::Insert => 4,
            Action::TX => 5,
            Action::Function => 6,
        }
    }
}

impl FromStr for Action {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Query" => Ok(Action::Query),
            "Update" => Ok(Action::Update),
            "Delete" => Ok(Action::Delete),
            "Insert" => Ok(Action::Insert),
            "TX" => Ok(Action::TX),
            "Function" => Ok(Action::Function),
            _ => Err(err(ENUM_NOT_FOUND)),
        }
    }
}

impl ToString for Action {
    fn to_string(&self) -> String {
        match self {
            Action::Query => String::from("Query"),
            Action::Update => String::from("Update"),
            Action::Delete => String::from("Delete"),
            Action::Insert => String::from("Insert"),
            Action::TX => String::from("TX"),
            Action::Function => String::from("Function"),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Validate, Clone)]
pub struct IncomeParamDef {
    pub name: String,
    pub required: bool,
    pub from: ParamFrom,
    pub param_type: ParamType,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct ExtraParamMap {
    pub params: HashMap<String, IncomeParamDef>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct DaprConfig {
    #[serde(with = "nullable_to_vec")]
    pub binding: Vec<DaprComponentInfo>,

    #[serde(with = "nullable_to_vec")]
    pub state: Vec<DaprComponentInfo>,

    #[serde(with = "nullable_to_vec")]
    pub pubsub: Vec<DaprComponentInfo>,

    #[serde(with = "nullable_to_vec")]
    pub secret: Vec<DaprComponentInfo>,

    #[serde(with = "nullable_to_vec")]
    pub conf: Vec<DaprComponentInfo>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct DaprComponentInfo {
    pub bb_type: DaprBuildBlockType,
    pub name: String,
    pub component_type: String,
    pub namespace: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub topic: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct DaprInvokeServiceInfo {
    pub bb_type: DaprBuildBlockType,
    pub name: String,
    pub component_type: String,
    pub namespace: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub topic: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub enum DaprBuildBlockType {
    #[default]
    Binding,
    State,
    Pubsub,
    Secret,
    Conf,
}

#[derive(Debug, Default)]
pub struct ContextWrapper<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C> {
    pub saga_id: Option<String>,
    pub uri_name: String,
    pub if_info: IfInfo,
    pub header: HashMap<String, String>,
    pub path_param: HashMap<u8, String>,
    pub query_param: HashMap<String, String>,
    pub input: I,
    pub inputs: Vec<I>,
    pub exec_name: Option<String>,
    pub exec: HashMap<String, (DaprRequest, DaprResponse, Option<Vec<Box<dyn DaprBody>>>)>,
    pub output: O,
    pub outputs: Vec<O>,
    pub page_info: Option<PageInfo>,
    pub inner_context: C,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct EmptyInnerContext {}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct DaprRequest {
    pub _dapr_config: Option<DaprComponentInfo>,
    pub invoke_service: Option<InvokeServiceRequest>,
    pub get_state: Option<GetStateRequest>,
    pub get_bulk_state: Option<GetBulkStateRequest>,
    pub query_state: Option<QueryStateRequest>,
    pub save_state: Option<SaveStateRequest>,
    pub transaction_state: Option<ExecuteStateTransactionRequest>,
    pub delete_state: Option<DeleteStateRequest>,
    pub delete_bulk_state: Option<DeleteBulkStateRequest>,
    pub invoke_binding: Option<InvokeBindingRequest>,
    pub invoke_binding_sql: Option<InvokeBindingSqlRequest>,
    pub publish_event: Option<PublishEventRequest>,
    pub publish_bulk_event: Option<BulkPublishRequest>,
    pub get_secret: Option<GetSecretRequest>,
    pub get_bluk_secret: Option<GetBulkSecretRequest>,
    pub get_configuration: Option<GetConfigurationRequest>,
}

impl DaprRequest {
    pub fn make_invoke_service(id: String, method: String, content_type: String, http_method: MethodEnum, querystring: String) -> HttpResult<Self> {
        let mut s: Self = Default::default();

        s.invoke_service = Some(InvokeServiceRequest {
            id,
            message: Some(InvokeRequest {
                method,
                data: None,
                content_type,
                http_extension: Some(HttpExtension {
                    verb: http_method.to_i32(),
                    querystring,
                }),
                headers: HashMap::<String, String>::new(),
            }),
        });

        Ok(s)
    }
}

pub trait DaprBody: Debug + Send + Downcast {
    fn as_dapr_body(self) -> Box<dyn DaprBody>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl_downcast!(DaprBody);

#[derive(PartialEq, Eq, Clone, Deserialize, Serialize, Message)]
pub struct EmptyDaprBody {}

impl DaprBody for EmptyDaprBody {}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct DaprResponse {
    pub invoke_service: Option<InvokeResponse>,
    pub get_state: Option<GetStateResponse>,
    pub get_bulk_state: Option<GetBulkStateResponse>,
    pub query_state: Option<QueryStateResponse>,
    pub invoke_binding: Option<InvokeBindingResponse>,
    pub invoke_binding_sql: Option<InvokeBindingSqlResponse>,
    pub publish_bulk_event: Option<BulkPublishResponse>,
    pub get_secret: Option<GetSecretResponse>,
    pub get_bluk_secret: Option<GetBulkSecretResponse>,
    pub get_configuration: Option<GetConfigurationResponse>,
}

#[derive(PartialEq, Eq, Reflect, Serialize, Deserialize, Clone, prost::Message)]
pub struct IfRes<T: ModelTrait + Message + Default> {
    #[prost(string, optional, tag = "1")]
    pub saga_id: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub uri_name: Option<String>,

    #[prost(enumeration = "Action", optional, tag = "3")]
    #[serde(with = "stringify_enum_action_option", default)]
    pub action: Option<i32>,

    #[prost(bool, tag = "4")]
    pub bulk_output: bool,

    #[prost(message, optional, tag = "5")]
    pub output: Option<T>,

    #[prost(message, repeated, tag = "6")]
    pub outputs: Vec<T>,

    #[prost(message, optional, tag = "7")]
    pub page_info: Option<PageInfoMessage>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct EmptyOutPut {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct EmptyOutPuts {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct EmptyInPut {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct EmptyInPuts {}

impl<T: ModelTrait + Message + Default + Serialize> IfRes<T> {
    pub fn to_message(self) -> IfResMessage {
        let mut any;
        let mut anys;
        if self.bulk_output {
            anys = if self.outputs.is_empty() {
                None
            } else {
                let json_value = serde_json::to_value(self.outputs).expect("to json error");
                let prost_value = serde_json_to_prost(json_value);
                Some(prost_types::Any {
                    type_url: "".to_string(),
                    value: prost_value.encode_to_vec(),
                })
            };
            any = None;
        } else {
            anys = None;
            any = match self.output {
                None => None,
                Some(pw) => Some(prost_types::Any {
                    type_url: "".to_string(),
                    value: pw.encode_to_vec(),
                }),
            };
        };

        IfResMessage {
            saga_id: self.saga_id,
            uri_name: self.uri_name,
            action: match self.action {
                None => None,
                Some(action) => Some(action),
            },
            bulk_output: self.bulk_output,
            output: any,
            outputs: anys,
            page_info: self.page_info,
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(PartialEq, Serialize, Deserialize, Clone, prost::Message)]
pub struct IfResMessage {
    #[prost(string, optional, tag = "1")]
    pub saga_id: Option<String>,
    #[prost(string, optional, tag = "2")]
    pub uri_name: Option<String>,
    #[prost(int32, optional, tag = "3")]
    pub action: Option<i32>,
    #[prost(bool, tag = "4")]
    pub bulk_output: bool,
    #[prost(message, optional, tag = "5")]
    pub output: Option<prost_types::Any>,
    #[prost(message, optional, tag = "6")]
    pub outputs: Option<prost_types::Any>,
    #[prost(message, optional, tag = "7")]
    pub page_info: Option<PageInfoMessage>,
}

#[derive(PartialEq, Eq, Reflect, Serialize, Deserialize, Validate, Clone, prost::Message)]
pub struct PageInfoMessage {
    #[prost(uint32, tag = "1")]
    pub total_data: u32,
    #[prost(uint32, tag = "2")]
    pub current_page_no: u32,
    #[prost(uint32, tag = "3")]
    pub total_pages: u32,
    #[prost(uint32, tag = "4")]
    pub page_size: u32,
}

#[derive(PartialEq, Eq, Default, Serialize, Deserialize, Debug, Clone, Reflect)]
pub enum MethodEnum {
    #[default]
    GET,
    POST,
    PUT,
    DELETE,
    CONNECT,
    HEAD,
    OPTIONS,
    PATCH,
    TRACE,
}

impl MethodEnum {
    pub fn to_i32(&self) -> i32 {
        match self {
            MethodEnum::GET => 1,
            MethodEnum::POST => 3,
            MethodEnum::PUT => 4,
            MethodEnum::DELETE => 5,
            MethodEnum::CONNECT => 6,
            MethodEnum::HEAD => 2,
            MethodEnum::OPTIONS => 7,
            MethodEnum::PATCH => 9,
            MethodEnum::TRACE => 8,
        }
    }
}

impl Into<Method> for MethodEnum {
    fn into(self) -> Method {
        match self {
            MethodEnum::GET => Method::GET,
            MethodEnum::POST => Method::POST,
            MethodEnum::PUT => Method::PUT,
            MethodEnum::DELETE => Method::DELETE,
            MethodEnum::CONNECT => Method::CONNECT,
            MethodEnum::HEAD => Method::HEAD,
            MethodEnum::OPTIONS => Method::OPTIONS,
            MethodEnum::PATCH => Method::PATCH,
            MethodEnum::TRACE => Method::TRACE,
        }
    }
}

impl FromStr for MethodEnum {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(MethodEnum::GET),
            "POST" => Ok(MethodEnum::POST),
            "PUT" => Ok(MethodEnum::PUT),
            "DELETE" => Ok(MethodEnum::DELETE),
            "CONNECT" => Ok(MethodEnum::CONNECT),
            "HEAD" => Ok(MethodEnum::HEAD),
            "OPTIONS" => Ok(MethodEnum::OPTIONS),
            "PATCH" => Ok(MethodEnum::PATCH),
            "TRACE" => Ok(MethodEnum::TRACE),
            _ => Err(ResponseError {
                biz_res: String::from("ENUM_NOT_FOUND"),
                message: None,
            }),
        }
    }
}

impl ToString for MethodEnum {
    fn to_string(&self) -> String {
        match self {
            MethodEnum::GET => String::from("GET"),
            MethodEnum::POST => String::from("POST"),
            MethodEnum::PUT => String::from("PUT"),
            MethodEnum::DELETE => String::from("DELETE"),
            MethodEnum::CONNECT => String::from("CONNECT"),
            MethodEnum::HEAD => String::from("HEAD"),
            MethodEnum::OPTIONS => String::from("OPTIONS"),
            MethodEnum::PATCH => String::from("PATCH"),
            MethodEnum::TRACE => String::from("TRACE"),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct FunctionContextV1beta1 {
    pub name: Option<String>,
    pub version: Option<String>,
    pub inputs: Option<HashMap<String, FunctionInput>>,
    pub outputs: Option<HashMap<String, FunctionOutput>>,
    pub states: Option<HashMap<String, FunctionOutput>>,
    pub runtime: Option<String>,
    pub port: Option<String>,
    pub prePlugins: Option<Vec<String>>,
    pub postPlugins: Option<Vec<String>>,
    pub pluginsTracing: Option<FunctionPluginsTracing>,
    pub httpPattern: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct FunctionInput {
    pub uri: Option<String>,
    pub componentName: Option<String>,
    pub componentType: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct FunctionOutput {
    pub uri: Option<String>,
    pub componentName: Option<String>,
    pub componentType: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub operation: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct FunctionState {
    pub componentName: Option<String>,
    pub componentType: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct FunctionPluginsTracing {
    pub enabled: Option<bool>,
    pub provider: Option<TracingProvider>,
    pub tags: Option<HashMap<String, String>>,
    pub baggage: Option<HashMap<String, String>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct TracingProvider {
    pub name: Option<String>,
    pub oapServer: Option<String>,
    pub exporter: Option<TracingExporter>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct TracingExporter {
    pub name: Option<String>,
    pub endpoint: Option<String>,
    pub headers: Option<String>,
    pub compression: Option<String>,
    pub timeout: Option<String>,
    pub protocol: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct FunctionContextV1beta2 {
    pub name: Option<String>,
    pub version: Option<String>,
    pub inputs: Option<HashMap<String, FunctionComponent>>,
    pub outputs: Option<HashMap<String, FunctionComponent>>,
    pub states: Option<HashMap<String, FunctionComponent>>,
    pub preHooks: Option<Vec<String>>,
    pub postHooks: Option<Vec<String>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct FunctionComponent {
    pub componentType: Option<String>,
    pub componentName: Option<String>,
    pub topic: Option<String>,
    pub operation: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct QueryFilterCondition {
    pub EQ: Option<(String, String)>,
    pub IN: Option<(String, Vec<String>)>,
    pub AND: Option<Vec<QueryFilterCondition>>,
    pub OR: Option<Vec<QueryFilterCondition>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub enum QuerySort {
    DESC,
    #[default]
    ASC,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct QuerySortCondition {
    pub key: Option<String>,
    pub order: Option<QuerySort>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct QueryPageCondition {
    pub limit: Option<u32>,
    pub token: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct QueryCondition {
    pub filter: Option<QueryFilterCondition>,
    pub sort: Option<Vec<QuerySortCondition>>,
    pub page: Option<QueryPageCondition>,
}

impl DaprBody for GetConfigurationResponse {}
impl DaprBody for GetBulkSecretResponse {}
impl DaprBody for GetSecretResponse {}
impl DaprBody for BulkPublishRequest {}
impl DaprBody for BulkPublishResponse {}
impl DaprBody for PublishEventRequest {}
impl DaprBody for InvokeBindingResponse {}
impl DaprBody for InvokeBindingRequest {}
impl DaprBody for DeleteBulkStateRequest {}
impl DaprBody for ExecuteStateTransactionRequest {}
impl DaprBody for SaveStateRequest {}
impl DaprBody for QueryStateRequest {}
impl DaprBody for QueryStateResponse {}
impl DaprBody for GetBulkStateRequest {}
impl DaprBody for GetBulkStateResponse {}
impl DaprBody for GetStateRequest {}
impl DaprBody for GetStateResponse {}
impl DaprBody for InvokeServiceRequest {}
impl DaprBody for InvokeResponse {}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub struct InvokeBindingSqlRequest {
    pub name: String,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub operation: SqlOperation,
    pub sqls: Vec<SqlWithParams>,
    pub is_select_page: Option<bool>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub struct SqlWithParams {
    pub sql: String,
    pub params: String,
    pub is_page: bool,
    pub output_columns: Vec<String>,
    pub offset: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub struct InvokeBindingSqlResponse {
    pub responses: Vec<SqlResponse>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub struct SqlResponse {
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub is_page: bool,
    pub output_columns: Vec<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug, Default)]
pub enum SqlOperation {
    #[default]
    Query,
    QueryPage,
    Exec,
    ExecTransaction,
}

impl SqlOperation {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SqlOperation::Query => "Query",
            SqlOperation::QueryPage => "QueryPage",
            SqlOperation::Exec => "Exec",
            SqlOperation::ExecTransaction => "ExecTransaction",
        }
    }
}

impl ToString for SqlOperation {
    fn to_string(&self) -> String {
        match self {
            SqlOperation::Query => String::from("query"),
            SqlOperation::QueryPage => String::from("query"),
            SqlOperation::Exec => String::from("exec"),
            SqlOperation::ExecTransaction => String::from("exec"),
        }
    }
}

pub enum AuthHeader {
    XSGAuthJWT,
    XSGAuthBasic,
    XSGAuthOAuth2,
    XSGAuthAksk,
    XSGAuthApiKey,
    XSGAuthDigestAuth,
    XSGAuthOIDC,
    XSGAuthInternal,
}

impl AuthHeader {
    pub fn upper_case_value(&self) -> &str {
        match self {
            AuthHeader::XSGAuthJWT => "X-SG-AUTH-JWT",
            AuthHeader::XSGAuthBasic => "X-SG-AUTH-BASIC",
            AuthHeader::XSGAuthOAuth2 => "X-SG-AUTH-OAUTH2",
            AuthHeader::XSGAuthAksk => "X-SG-AUTH-AKSK",
            AuthHeader::XSGAuthApiKey => "X-SG-AUTH-APIKEY",
            AuthHeader::XSGAuthDigestAuth => "X-SG-AUTH-DIGESTAUTH",
            AuthHeader::XSGAuthOIDC => "X-SG-AUTH-OIDC",
            AuthHeader::XSGAuthInternal => "X-SG-AUTH-INTERNAL",
        }
    }

    pub fn lower_case_value(&self) -> &str {
        match self {
            AuthHeader::XSGAuthJWT => "x-sg-auth-jwt",
            AuthHeader::XSGAuthBasic => "x-sg-auth-basic",
            AuthHeader::XSGAuthOAuth2 => "x-sg-auth-oauth2",
            AuthHeader::XSGAuthAksk => "x-sg-auth-aksk",
            AuthHeader::XSGAuthApiKey => "x-sg-auth-apikey",
            AuthHeader::XSGAuthDigestAuth => "x-sg-auth-digestauth",
            AuthHeader::XSGAuthOIDC => "x-sg-auth-oidc",
            AuthHeader::XSGAuthInternal => "x-sg-auth-internal",
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, Model, Validate, ModelValidate, prost::Message)]
pub struct JwtToken {
    #[prost(string, optional, tag = "1")]
    pub token: Option<String>,
}

impl DaprBody for JwtToken {}

pub trait HttpRequestDispatcherTrait {
    fn do_http_dispatch(params: Params) -> impl std::future::Future<Output = HttpResult<Response<body::Body>>> + Send;
}

pub trait GrpcRequestDispatcherTrait {
    fn do_grpc_dispatch(params: Params) -> impl std::future::Future<Output = GrpcResult<tonic::Response<InvokeResponse>>> + Send;
}
