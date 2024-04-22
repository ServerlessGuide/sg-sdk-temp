use std::collections::HashMap;
use std::env;

use tonic::Status;

use crate::{
    model::{DaprConfig, ExtraParamMap, FunctionContextV1beta1, FunctionContextV1beta2},
    util::{find_regex, BizResult, URI},
};

pub mod config;
pub mod daprs;
pub mod envs;
pub mod model;
pub mod start;
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
    pub static ref BIZ_RESULT_PREFIX: i16 = 1048;
    pub static ref SKIP_AUTH_IFS: Vec<String> = vec![];
    pub static ref INTERNAL_AUTH_TAG: Option<String> = Some(String::from("Serverless-Guide"));
    pub static ref URI_REGEX_MAP: HashMap<URI, regex::Regex> = {
        let uris = URIS.clone().into_values().collect();

        let uris_res = find_regex(uris);
        if let Err(err) = &uris_res {
            eprintln!("generate uri regex error: {}", err);
        }

        uris_res.unwrap()
    };
    pub static ref URIS: HashMap<String, URI> = {
        let mut uris = HashMap::<String, URI>::new();
        uris
    };
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
    pub static ref BIZ_RESULT_MAP: HashMap<String, BizResult<'static>> = {
        let mut biz_res_map = HashMap::<String, BizResult>::new();
        biz_res_map
    };
    pub static ref INCOME_PARAM_MAP: HashMap<String, ExtraParamMap> = {
        let mut params = HashMap::<String, ExtraParamMap>::new();
        params
    };
}

biz_result! {
    (OK, 200, 105300, "success", "OK");
    (URI_NOT_MATCH, 404, 105301, "uri match nothing", "URI_NOT_MATCH");
    (BODY_PARAMETER_ILLEGAL, 400, 105302, "body parameter illegal", "BODY_PARAMETER_ILLEGAL");
    (CONVERT_TO_MODEL_ERROR, 500, 105303, "convert to model error", "CONVERT_TO_MODEL_ERROR");
    (PARAMETER_ILLEGAL, 400, 105304, "parameter illegal", "PARAMETER_ILLEGAL");
    (HEADER_NOT_FOUND, 400, 105305, "header not found", "HEADER_NOT_FOUND");
    (PARAM_MAP_PARSE_ERROR, 500, 105306, "param map parse error", "PARAM_MAP_PARSE_ERROR");
    (PATH_PARAM_NOT_EXIST, 500, 105307, "path param not exist", "PATH_PARAM_NOT_EXIST");
    (BODY_PARAM_NOT_EXIST, 500, 105308, "body param not exist", "BODY_PARAM_NOT_EXIST");
    (QUERY_PARAM_NOT_EXIST, 500, 105309, "query param not exist", "QUERY_PARAM_NOT_EXIST");
    (URL_PARSE_ERROR, 500, 105310, "url parse error", "URL_PARSE_ERROR");
    (DAPR_HTTP_REQ_BUILD_ERROR, 500, 105311, "dapr request build error", "DAPR_HTTP_REQ_BUILD_ERROR");
    (DAPR_REQUEST_FAIL, 500, 105312, "dapr request fail", "DAPR_REQUEST_FAIL");
    (REQUEST_METHOD_NOT_ALLOWED, 500, 105313, "request method not allowed", "REQUEST_METHOD_NOT_ALLOWED");
    (ENV_PARAMETER_ERROR, 500, 105314, "env parameter error", "ENV_PARAMETER_ERROR");
    (DAPR_DATA_ILLEGAL, 500, 105315, "dapr data illegal", "DAPR_DATA_ILLEGAL");
    (ENUM_NOT_FOUND, 500, 105316, "enum not found", "ENUM_NOT_FOUND");
    (IMPLICIT_RESPONSE_ERROR, 500, 105317, "implicit response error", "IMPLICIT_RESPONSE_ERROR");
    (BIZ_RESULT_NOT_FOUND, 500, 105318, "biz result not found", "BIZ_RESULT_NOT_FOUND");
    (DAPR_CONFIG_NOT_EXIST, 500, 105319, "dapr config not exist", "DAPR_CONFIG_NOT_EXIST");
    (EXEC_NAME_NOT_EXIST, 500, 105320, "execute name not exist", "EXEC_NAME_NOT_EXIST");
    (DAPR_EXECUTE_NOT_EXIST, 500, 105321, "dapr execute not exist", "DAPR_EXECUTE_NOT_EXIST");
    (QUERY_SQL_IS_NOT_UNIQUE, 500, 105322, "query sql is not unique", "QUERY_SQL_IS_NOT_UNIQUE");
    (SQL_NOT_VALID, 500, 105323, "sql not valid", "SQL_NOT_VALID");
    (SQL_NOT_SUPPORT, 500, 105324, "sql not support", "SQL_NOT_SUPPORT");
    (DATA_NOT_FOUND, 400, 105325, "data not found", "DATA_NOT_FOUND");
    (SQL_OUT_COLUMNS_IS_EMPTY, 500, 105326, "sql out_columns is empty", "SQL_OUT_COLUMNS_IS_EMPTY");
    (DATA_ERROR, 500, 105327, "data error", "DATA_ERROR");
    (AUTH_ERROR, 401, 105328, "auth error", "AUTH_ERROR");
    (INTERNAL_AUTH_TAG_NOT_SET, 500, 105329, "internal auth tag not set", "INTERNAL_AUTH_TAG_NOT_SET");
}
