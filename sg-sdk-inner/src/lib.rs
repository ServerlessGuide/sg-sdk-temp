use std::collections::HashMap;
use std::env;

use tokio::sync::RwLock;
use tonic::Status;
use tracing::{error, info, warn};

use crate::{
    model::{DaprConfig, ExtraParamMap, FunctionContextV1beta1, FunctionContextV1beta2},
    util::{BizResult, URI},
};

pub mod config;
pub mod context_extension;
pub mod dapr_resp_resolve;
pub mod daprs;
pub mod inner_biz_result;
pub mod log;
pub mod macros;
pub mod model;
pub mod nullable_to_vec;
pub mod sql_builder;
pub mod start;
pub mod stringify_on_num;
pub mod traits;
pub mod util;

pub mod body {
    use std::convert::Infallible;

    use http_body_util::{combinators::BoxBody, Either, Empty, Full};
    use hyper::body::Bytes;

    pub type Body = Either<Empty<Bytes>, Full<Bytes>>;

    pub type BodySt = Either<Empty<Bytes>, BoxBody<Bytes, Infallible>>;

    pub fn empty() -> Body {
        Either::Left(Empty::new())
    }

    pub fn bytes<B: Into<Bytes>>(chunk: B) -> Body {
        Either::Right(Full::from(chunk.into()))
    }

    pub fn stream_body(chunk_stream: BoxBody<Bytes, Infallible>) -> BodySt {
        Either::Right(chunk_stream)
    }
}

pub type HttpResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type GrpcResult<T> = std::result::Result<T, Status>;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref SERVICE_ID: i64 = 6999453093112840195;
    pub static ref SKIP_AUTH_IFS: RwLock<Vec<String>> = RwLock::new(vec![]);
    pub static ref INTERNAL_AUTH_TAG: RwLock<Option<String>> = RwLock::new(None);
    pub static ref URI_REGEX_MAP: RwLock<HashMap<URI, regex::Regex>> = RwLock::new(HashMap::<URI, regex::Regex>::new());
    pub static ref URIS: RwLock<HashMap<String, URI>> = RwLock::new(HashMap::<String, URI>::new());
    pub static ref URI_HANDLERS: RwLock<Vec<(String, String)>> = RwLock::new(Vec::<(String, String)>::new());
    pub static ref BIZ_RESULT_MAP: RwLock<HashMap<String, BizResult<'static>>> = RwLock::new(HashMap::<String, BizResult>::new());
    pub static ref INCOME_PARAM_MAP: RwLock<HashMap<String, ExtraParamMap>> = RwLock::new(HashMap::<String, ExtraParamMap>::new());
    pub static ref DAPR_CONFIG: DaprConfig = {
        match env::var("DAPR_CONFIG") {
            Ok(val) => match serde_json::from_str::<DaprConfig>(&val) {
                Ok(v) => v,
                Err(err) => {
                    error!("env DAPR_CONFIG format error: {}", err);
                    panic!("init DAPR_CONFIG error!")
                }
            },
            Err(_) => {
                error!("env DAPR_CONFIG not found");
                panic!("init DAPR_CONFIG error!")
            }
        }
    };
    pub static ref FUNC_CONTEXT: FunctionContextV1beta1 = {
        match env::var("FUNC_CONTEXT") {
            Ok(val) => match serde_json::from_str::<FunctionContextV1beta1>(&val) {
                Ok(v) => v,
                Err(_) => {
                    warn!("env FUNC_CONTEXT format error");
                    panic!("init FUNC_CONTEXT error!")
                }
            },
            Err(_) => {
                warn!("env FUNC_CONTEXT not found");
                panic!("init FUNC_CONTEXT error!")
            }
        }
    };
    pub static ref FUNC_CONTEXT_V1BETA2: FunctionContextV1beta2 = {
        match env::var("FUNC_CONTEXT_V1BETA2") {
            Ok(val) => match serde_json::from_str::<FunctionContextV1beta2>(&val) {
                Ok(v) => v,
                Err(_) => {
                    warn!("env FUNC_CONTEXT_V1BETA2 format error");
                    panic!("init FUNC_CONTEXT_V1BETA2 error!")
                }
            },
            Err(_) => {
                warn!("env FUNC_CONTEXT_V1BETA2 not found");
                panic!("init FUNC_CONTEXT_V1BETA2 error!")
            }
        }
    };
    pub static ref ENVS: HashMap<String, String> = {
        let mut envs = HashMap::<String, String>::new();

        for (k, v) in env::vars() {
            envs.insert(k, v);
        }

        if let None = envs.get("FUNC_CONTEXT") {
            error!("Important!!! env {} not set!", "FUNC_CONTEXT");
            panic!("init envs error!")
        }
        if let None = envs.get("DAPR_HOST") {
            error!("Important!!! env {} not set! will use localhost default.", "DAPR_HOST");
            envs.insert(String::from("DAPR_HOST"), String::from("localhost"));
        }
        if let None = envs.get("POD_NAMESPACE") {
            error!("Important!!! env {} not set!", "POD_NAMESPACE");
            panic!("init envs error!")
        }

        info!("envs:{:?}", envs);

        envs
    };
}
