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
    pub static ref BIZ_RESULT_PREFIX: RwLock<i16> = RwLock::new(-1);
    pub static ref SKIP_AUTH_IFS: RwLock<Vec<String>> = RwLock::new(vec![]);
    pub static ref INTERNAL_AUTH_TAG: RwLock<Option<String>> = RwLock::new(None);
    pub static ref URI_REGEX_MAP: RwLock<HashMap<URI, regex::Regex>> = RwLock::new(HashMap::<URI, regex::Regex>::new());
    pub static ref URIS: RwLock<HashMap<String, URI>> = RwLock::new(HashMap::<String, URI>::new());
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

pub async fn init() {
    // 初始化业务码prefix，示例：
    // biz_code_prefix!(9801);

    // 这些是固定的BizResult，范围：0-999999，业务场景不能使用
    biz_result! {
        (OK, 200, 0, "success");
        (URI_NOT_MATCH, 404, 11, "uri match nothing");
        (BODY_PARAMETER_ILLEGAL, 400, 2, "body parameter illegal");
        (CONVERT_TO_MODEL_ERROR, 500, 3, "convert to model error");
        (PARAMETER_ILLEGAL, 400, 4, "parameter illegal");
        (HEADER_NOT_FOUND, 400, 5, "header not found");
        (PARAM_MAP_PARSE_ERROR, 500, 6, "param map parse error");
        (PATH_PARAM_NOT_EXIST, 500, 7, "path param not exist");
        (BODY_PARAM_NOT_EXIST, 500, 8, "body param not exist");
        (QUERY_PARAM_NOT_EXIST, 500, 9, "query param not exist");
        (URL_PARSE_ERROR, 500, 10, "url parse error");
        (DAPR_HTTP_REQ_BUILD_ERROR, 500, 11, "dapr request build error");
        (DAPR_REQUEST_FAIL, 500, 12, "dapr request fail");
        (REQUEST_METHOD_NOT_ALLOWED, 500, 13, "request method not allowed");
        (ENV_PARAMETER_ERROR, 500, 14, "env parameter error");
        (DAPR_DATA_ILLEGAL, 500, 15, "dapr data illegal");
        (ENUM_NOT_FOUND, 500, 16, "enum not found");
        (IMPLICIT_RESPONSE_ERROR, 500, 17, "implicit response error");
        (BIZ_RESULT_NOT_FOUND, 500, 18, "biz result not found");
        (DAPR_CONFIG_NOT_EXIST, 500, 19, "dapr config not exist");
        (EXEC_NAME_NOT_EXIST, 500, 20, "execute name not exist");
        (DAPR_EXECUTE_NOT_EXIST, 500, 21, "dapr execute not exist");
        (QUERY_SQL_IS_NOT_UNIQUE, 500, 22, "query sql is not unique");
        (SQL_NOT_VALID, 500, 23, "sql not valid");
        (SQL_NOT_SUPPORT, 500, 24, "sql not support");
        (DATA_NOT_FOUND, 400, 25, "data not found");
        (SQL_OUT_COLUMNS_IS_EMPTY, 500, 26, "sql out_columns is empty");
        (DATA_ERROR, 500, 27, "data error");
        (AUTH_ERROR, 401, 28, "auth error");
        (INTERNAL_AUTH_TAG_NOT_SET, 500, 29, "internal auth tag not set");
    };

    // 初始化URI，示例：
    // uri! {
    //     (QUERY_BY_APP_ID, GET, "^/app-version/\\d{19}$", Query, false, true);
    //     (INSERT, POST, "^/app-version$", Insert, false, false);
    //     (ENV_PREPARE, GET, "^/app-version/\\d{19}/env-prepare$", Function, false, false);
    // }

    // 初始化URI参数，示例：
    // income_param! {
    //     (INSERT, [(app_id, app_id, Body, Number, true)]);
    // }

    // 设置内部校验tag，不区分大小写，不能为空串
    // internal_auth_tag!("test-tag");

    // skip_auth_uri!(INSERT);
}
