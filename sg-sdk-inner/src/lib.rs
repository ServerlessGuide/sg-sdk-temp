use std::collections::HashMap;
use std::env;

use tokio::sync::RwLock;
use tonic::Status;

use crate::{
    model::{DaprConfig, ExtraParamMap, FunctionContextV1beta1, FunctionContextV1beta2},
    util::{BizResult, URI},
};

pub mod config;
pub mod daprs;
pub mod log;
pub mod macros;
pub mod model;
pub mod nullable_to_vec;
pub mod start;
pub mod stringify_on_num;
pub mod util;

pub mod body {
    use http_body_util::{Either, Empty, Full};
    use hyper::body::Bytes;

    pub type Body = Either<Empty<Bytes>, Full<Bytes>>;

    pub fn empty() -> Body {
        Either::Left(Empty::new())
    }

    pub fn bytes<B: Into<Bytes>>(chunk: B) -> Body {
        Either::Right(Full::from(chunk.into()))
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
    pub static ref DAPR_CONFIG: HashMap<String, DaprConfig> = {
        match env::var("DAPR_CONFIG") {
            Ok(val) => match serde_json::from_str::<HashMap<String, DaprConfig>>(&val) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("env DAPR_CONFIG format error: {}", err);
                    panic!("init DAPR_CONFIG error!")
                }
            },
            Err(_) => {
                eprintln!("env DAPR_CONFIG not found");
                panic!("init DAPR_CONFIG error!")
            }
        }
    };
    pub static ref FUNC_CONTEXT: FunctionContextV1beta1 = {
        match env::var("FUNC_CONTEXT") {
            Ok(val) => match serde_json::from_str::<FunctionContextV1beta1>(&val) {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("env FUNC_CONTEXT format error");
                    panic!("init FUNC_CONTEXT error!")
                }
            },
            Err(_) => {
                eprintln!("env FUNC_CONTEXT not found");
                panic!("init FUNC_CONTEXT error!")
            }
        }
    };
    pub static ref FUNC_CONTEXT_V1BETA2: FunctionContextV1beta2 = {
        match env::var("FUNC_CONTEXT_V1BETA2") {
            Ok(val) => match serde_json::from_str::<FunctionContextV1beta2>(&val) {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("env FUNC_CONTEXT_V1BETA2 format error");
                    panic!("init FUNC_CONTEXT_V1BETA2 error!")
                }
            },
            Err(_) => {
                eprintln!("env FUNC_CONTEXT_V1BETA2 not found");
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
            eprintln!("Important!!! env {} not set!", "FUNC_CONTEXT");
            panic!("init envs error!")
        }
        if let None = envs.get("DAPR_HOST") {
            println!("Important!!! env {} not set! will use localhost default.", "DAPR_HOST");
            envs.insert(String::from("DAPR_HOST"), String::from("localhost"));
        }
        if let None = envs.get("POD_NAMESPACE") {
            eprintln!("Important!!! env {} not set!", "POD_NAMESPACE");
            panic!("init envs error!")
        }

        println!("envs:{:?}", envs);

        envs
    };
}

pub async fn init() -> HttpResult<()> {
    // 设置接口入参，示例
    // income_param! {
    //     (QUERY_BY_APP_ID, [(app_id, 2, Path, Number, true)]);
    //     (INSERT, [(app_id, app_id, Body, Number, true), (version, version, Body, String, true)]);
    //     (ENV_PREPARE, [(app_id, 2, Path, Number, true)]);
    // }

    // 设置内部校验tag，不区分大小写，不能为空串
    // internal_auth_tag!("test-tag");

    // 设置跳过校验的URI
    // skip_auth_uri!(INSERT);

    Ok(())
}

// 初始化URI，示例：
// uri! {
//     (QUERY_BY_APP_ID, GET, "^/app-version/\\d{19}$", Query, false, true);
//     (INSERT, POST, "^/app-version$", Insert, false, false);
//     (ENV_PREPARE, GET, "^/app-version/\\d{19}/env-prepare$", Function, false, false);
// }

// 初始化BizResult，这些是固定的BizResult，范围：999900-999999，业务场景不能使用，示例：
biz_result! {
    InnerConfigForSelfUse,
    (OK, 200, 999900, "success");
    (URI_NOT_MATCH, 404, 999901, "uri match nothing");
    (BODY_PARAMETER_ILLEGAL, 400, 999902, "body parameter illegal");
    (CONVERT_TO_MODEL_ERROR, 500, 999903, "convert to model error");
    (PARAMETER_ILLEGAL, 400, 999904, "parameter illegal");
    (HEADER_NOT_FOUND, 400, 999905, "header not found");
    (PARAM_MAP_PARSE_ERROR, 500, 999906, "param map parse error");
    (PATH_PARAM_NOT_EXIST, 500, 999907, "path param not exist");
    (BODY_PARAM_NOT_EXIST, 500, 999908, "body param not exist");
    (QUERY_PARAM_NOT_EXIST, 500, 999909, "query param not exist");
    (URL_PARSE_ERROR, 500, 999910, "url parse error");
    (DAPR_HTTP_REQ_BUILD_ERROR, 500, 999911, "dapr request build error");
    (DAPR_REQUEST_FAIL, 500, 999912, "dapr request fail");
    (REQUEST_METHOD_NOT_ALLOWED, 500, 999913, "request method not allowed");
    (ENV_PARAMETER_ERROR, 500, 999914, "env parameter error");
    (DAPR_DATA_ILLEGAL, 500, 999915, "dapr data illegal");
    (ENUM_NOT_FOUND, 500, 999916, "enum not found");
    (IMPLICIT_RESPONSE_ERROR, 500, 999917, "implicit response error");
    (BIZ_RESULT_NOT_FOUND, 500, 999918, "biz result not found");
    (DAPR_CONFIG_NOT_EXIST, 500, 999919, "dapr config not exist");
    (EXEC_NAME_NOT_EXIST, 500, 999920, "execute name not exist");
    (DAPR_EXECUTE_NOT_EXIST, 500, 999921, "dapr execute not exist");
    (QUERY_SQL_IS_NOT_UNIQUE, 500, 999922, "query sql is not unique");
    (SQL_NOT_VALID, 500, 999923, "sql not valid");
    (SQL_NOT_SUPPORT, 500, 999924, "sql not support");
    (DATA_NOT_FOUND, 400, 999925, "data not found");
    (SQL_OUT_COLUMNS_IS_EMPTY, 500, 999926, "sql out_columns is empty");
    (DATA_ERROR, 500, 999927, "data error");
    (AUTH_ERROR, 401, 999928, "auth error");
    (INTERNAL_AUTH_TAG_NOT_SET, 500, 999929, "internal auth tag not set");
}

struct InnerConfigForSelfUse();
