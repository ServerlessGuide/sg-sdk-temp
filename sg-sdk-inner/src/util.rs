use crate::{
    body, config::*, daprs::*, model::*, GrpcResult, HttpResult, DAPR_CONFIG, INCOME_PARAM_MAP, INTERNAL_AUTH_TAG, SKIP_AUTH_IFS, URIS, URI_REGEX_MAP, *,
};
use chrono::{DateTime, Local};
use dapr::{
    appcallback::InvokeRequest,
    client::*,
    dapr::dapr::proto::runtime::v1::{
        BulkPublishRequest, ExecuteStateTransactionRequest, GetBulkSecretRequest, GetBulkStateRequest, GetConfigurationRequest, QueryStateRequest,
    },
};
use http_body_util::*;
use hyper::{
    body::Incoming,
    header::{self, HeaderName, HeaderValue},
    Method, Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use model_macro::{ModelTrait, Validator};
use prost::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlparser::{
    ast::{Expr, SelectItem, SetExpr, Statement},
    dialect::GenericDialect,
    parser::Parser,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    str::FromStr,
};
use tokio::net::TcpStream;
use tonic::Status;
use tracing::{debug, error, info, trace, warn};
use validator::Validate;

#[derive(Debug, Clone)]
pub struct ResponseError {
    pub biz_res: String,
    pub message: Option<String>,
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}",
            self.biz_res,
            match &self.message {
                None => "None".to_string(),
                Some(m) => m.to_string(),
            }
        )
    }
}

impl std::error::Error for ResponseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl ResponseError {
    pub fn from_str_ref_err(err: &str) -> Self {
        ResponseError {
            biz_res: IMPLICIT_RESPONSE_ERROR.name(),
            message: Some(format!("{}: {}", IMPLICIT_RESPONSE_ERROR.message(), err.to_string())),
        }
    }

    pub fn from_string_err(err: String) -> Self {
        ResponseError {
            biz_res: IMPLICIT_RESPONSE_ERROR.name(),
            message: Some(format!("{}: {}", IMPLICIT_RESPONSE_ERROR.message(), err)),
        }
    }

    pub fn from_box_err(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        if err.is::<ResponseError>() {
            let respnse_err = err.downcast_ref::<ResponseError>().unwrap();
            respnse_err.to_owned()
        } else {
            ResponseError {
                biz_res: IMPLICIT_RESPONSE_ERROR.name(),
                message: Some(format!("{}: {}", IMPLICIT_RESPONSE_ERROR.message(), err.to_string())),
            }
        }
    }
}

pub async fn err_resolve(err: Box<dyn std::error::Error + Send + Sync>) -> Response<body::Body> {
    debug!(
        "============================handle finish with error============================\nend time: {}\n{:#?}",
        utc_timestamp(),
        err
    );
    if err.is::<ResponseError>() || err.is::<Box<ResponseError>>() {
        let respnse_err;
        if err.is::<ResponseError>() {
            respnse_err = err.downcast_ref::<ResponseError>().unwrap();
        } else {
            respnse_err = err.downcast_ref::<Box<ResponseError>>().unwrap();
        }
        let biz_res_name = respnse_err.biz_res.to_owned();
        let biz_res = BizResult::from(biz_res_name).await;

        if let Err(err) = biz_res {
            if err.is::<ResponseError>() {
                return gen_resp(
                    BIZ_RESULT_NOT_FOUND.status_code(),
                    Res::<String> {
                        code: BIZ_RESULT_NOT_FOUND.biz_code(),
                        message: match &respnse_err.message {
                            None => BIZ_RESULT_NOT_FOUND.message(),
                            Some(message) => format!("{}: {}", BIZ_RESULT_NOT_FOUND.message(), message),
                        },
                        result: None,
                    },
                );
            } else {
                return gen_resp(
                    IMPLICIT_RESPONSE_ERROR.status_code(),
                    Res::<String> {
                        code: IMPLICIT_RESPONSE_ERROR.biz_code(),
                        message: match &respnse_err.message {
                            None => IMPLICIT_RESPONSE_ERROR.message(),
                            Some(message) => format!("{}: {}", IMPLICIT_RESPONSE_ERROR.message(), message),
                        },
                        result: None,
                    },
                );
            }
        }
        let biz_res = biz_res.unwrap();
        if let None = respnse_err.message {
            gen_resp(
                biz_res.status_code(),
                Res::<String> {
                    code: biz_res.biz_code(),
                    message: biz_res.message(),
                    result: None,
                },
            )
        } else {
            gen_resp(
                biz_res.status_code(),
                Res::<String> {
                    code: biz_res.biz_code(),
                    message: format!("{}: {}", biz_res.message(), respnse_err.message.clone().unwrap()),
                    result: None,
                },
            )
        }
    } else {
        gen_resp(
            IMPLICIT_RESPONSE_ERROR.status_code(),
            Res::<String> {
                code: IMPLICIT_RESPONSE_ERROR.biz_code(),
                message: format!("{}: {}", IMPLICIT_RESPONSE_ERROR.message(), err.to_string()),
                result: None,
            },
        )
    }
}

pub fn gen_resp<T: Serialize + Display>(status_code: u16, body: Res<T>) -> Response<body::Body> {
    let mut response_builder = Response::builder();

    let code = StatusCode::from_u16(status_code);
    if let Err(_) = code {
        response_builder = response_builder.status(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let code = code.unwrap();
    response_builder = response_builder.status(code);

    let value = serde_json::to_value(body).unwrap().to_string();
    let resp = response_builder.body(body::bytes(value)).unwrap();

    resp
}

pub fn gen_resp_err<'a>(biz_res: BizResult<'static>, message: Option<String>) -> ResponseError {
    ResponseError {
        biz_res: biz_res.name(),
        message,
    }
}

pub async fn gen_resp_ok<T: Serialize>(biz_res: BizResult<'static>, result: T, params: &Params) -> Response<body::Body> {
    let mut response_builder = Response::builder();

    response_builder = response_builder.header(header::CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());

    let token_pair = find_response_auth_header(params).await.unwrap();

    match token_pair.0 {
        None => {}
        Some(key) => match token_pair.1 {
            None => {}
            Some(value) => {
                response_builder = response_builder.header(HeaderName::from_str(&key).unwrap(), HeaderValue::from_str(&value).unwrap());
            }
        },
    }

    let code = StatusCode::from_u16(biz_res.status_code());
    if let Err(_) = code {
        response_builder = response_builder.status(StatusCode::INTERNAL_SERVER_ERROR);
    } else {
        response_builder = response_builder.status(code.unwrap());
    }

    let resp_body = Res::<T> {
        code: biz_res.biz_code(),
        message: biz_res.message(),
        result: Some(result),
    };
    let json = serde_json::to_string(&resp_body).unwrap();
    let resp = response_builder.body(body::bytes(json.as_bytes().to_vec())).unwrap();

    debug!(
        "============================handle finish OK============================\nend time: {}",
        utc_timestamp()
    );
    resp
}

#[derive(PartialEq, Eq, Debug, Hash, Clone, Default)]
pub struct URI(pub Method, pub &'static str, pub &'static str, pub Action, pub bool, pub bool);

impl URI {
    pub fn method(&self) -> &Method {
        &self.0
    }

    pub fn path(&self) -> &str {
        &self.1
    }

    pub fn name(&self) -> &str {
        &self.2
    }

    pub fn action(&self) -> &Action {
        &self.3
    }

    pub fn bulk_input(&self) -> &bool {
        &self.4
    }

    pub fn bulk_output(&self) -> &bool {
        &self.5
    }
}

pub async fn insert_uri(uri: URI) -> HttpResult<()> {
    let mut uris = URIS.write().await;
    match uris.insert(uri.name().to_string(), uri.clone()) {
        None => {}
        Some(_) => {
            return Err(Box::new(ResponseError {
                biz_res: format!("uri is exist: {}", uri.name()),
                message: None,
            }));
        }
    };

    let mut uri_regex_map = URI_REGEX_MAP.write().await;
    uri_regex_map.insert(uri.clone(), regex::Regex::new(uri.path())?);

    Ok(())
}

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub struct BizResult<'a>(pub u16, pub i32, pub &'a str, pub &'a str);

impl BizResult<'static> {
    pub fn status_code(&self) -> u16 {
        self.0
    }

    pub fn biz_code(&self) -> i32 {
        self.1
    }

    pub fn message(&self) -> String {
        self.2.to_string()
    }

    pub fn name(&self) -> String {
        self.3.to_string()
    }

    pub async fn from(item: String) -> HttpResult<Self> {
        let biz_result_map = BIZ_RESULT_MAP.read().await;
        let res = biz_result_map.get(&item);
        let Some(res) = res else {
            return Err(Box::new(gen_resp_err(BIZ_RESULT_NOT_FOUND, None)));
        };
        Ok(*res)
    }
}

pub async fn insert_biz_result(mut biz_res: BizResult<'static>) -> HttpResult<()> {
    let mut biz_result_map = BIZ_RESULT_MAP.write().await;
    match biz_result_map.insert(biz_res.name(), biz_res) {
        None => {}
        Some(_) => {
            return Err(Box::new(ResponseError {
                biz_res: format!("biz result is exist: {}", biz_res.name()),
                message: None,
            }));
        }
    };
    Ok(())
}

pub async fn insert_income_param(uri: URI, params: Vec<(String, String, ParamFrom, ParamType, bool)>) -> HttpResult<()> {
    let mut interface_params = HashMap::<String, IncomeParamDef>::new();

    for (target, name, from, param_type, required) in params {
        interface_params.insert(
            target,
            IncomeParamDef {
                name: name,
                from: from,
                param_type: param_type,
                required: required,
            },
        );
    }

    let mut income_param_map = INCOME_PARAM_MAP.write().await;

    match income_param_map.insert(uri.name().to_string(), ExtraParamMap { params: interface_params }) {
        None => {}
        Some(_) => {
            return Err(Box::new(ResponseError {
                biz_res: format!("income param of uri '{}' is exist", uri.name()),
                message: None,
            }));
        }
    };

    Ok(())
}

pub async fn set_internal_auth_tag(tag: &str) -> HttpResult<()> {
    *INTERNAL_AUTH_TAG.write().await = match tag.is_empty() {
        true => {
            return Err(Box::new(ResponseError {
                biz_res: String::from("internal auth tag can not be empty"),
                message: None,
            }));
        }
        false => Some(tag.to_string()),
    };

    Ok(())
}

pub async fn set_skip_auth_uri(uri: URI) -> HttpResult<()> {
    let mut skip_ifs = SKIP_AUTH_IFS.write().await;

    skip_ifs.push(uri.name().to_string());

    Ok(())
}

pub async fn uri_match(req_path: &str, req_method: Method) -> HttpResult<URI> {
    let uri_regex_map = URI_REGEX_MAP.read().await;
    for (uri, regex) in uri_regex_map.iter() {
        if regex.is_match(req_path) && uri.method() == req_method {
            return Ok(uri.to_owned());
        }
    }
    Err(Box::new(gen_resp_err(
        URI_NOT_MATCH,
        Some(format!("uri: {}, method: {}.", req_path, req_method.as_str())),
    )))
}

pub async fn parse_params_grpc(req: tonic::Request<InvokeRequest>) -> GrpcResult<Params> {
    let metadata = &req.metadata().clone();
    let r = &req.into_inner();

    let Some(http_extension) = &r.http_extension else {
        return Err(Status::failed_precondition("http extension not appointed."));
    };

    let http_method = Method::from_str(http_extension.verb().as_str_name());
    let Ok(http_method) = http_method else {
        return Err(Status::failed_precondition("http extension verb not available."));
    };

    let path = &r.method;

    let uri = uri_match(path, http_method).await;
    let Ok(uri) = uri else {
        return Err(Status::failed_precondition("uri not match."));
    };

    let mut uri_path_params = HashMap::<u8, String>::new();
    let paths: Vec<&str> = path.split("/").collect();
    for i in 0..paths.len() {
        uri_path_params.insert(i as u8, paths[i].to_string());
    }

    let mut headers = HashMap::<String, String>::new();
    for (k, v) in metadata.clone().into_headers().iter() {
        headers.insert(k.to_string(), v.to_str().unwrap().to_string());
    }

    if !headers.contains_key("Content-Type") {
        headers.insert("Content-Type".to_string(), r.content_type.clone().to_string());
    }

    let mut uri_query_params = HashMap::<String, String>::new();
    let query = &http_extension.querystring;
    let querys: Vec<&str> = query.split("&").collect();
    for kv_pair in querys {
        let kvs: Vec<&str> = kv_pair.split("=").collect();
        uri_query_params.insert(kvs[0].to_string(), kvs[1].to_string());
    }

    let mut params: Params = Default::default();
    params.uri = uri.name().to_string();
    params.header = headers;
    params.query_param = uri_query_params;
    params.path_param = uri_path_params;
    params.if_info = IfInfo {
        action: uri.action().clone(),
        bulk_input: uri.bulk_input().clone(),
        bulk_output: uri.bulk_output().clone(),
    };

    match &r.data {
        None => {}
        Some(data) => {
            params.body = Some(data.value.clone());
        }
    }

    Ok(params)
}

pub async fn parse_params(req: Request<Incoming>) -> HttpResult<Params> {
    let uri = uri_match(req.uri().path(), req.method().to_owned()).await?;

    let mut headers = HashMap::<String, String>::new();
    for (k, v) in req.headers().into_iter() {
        headers.insert(k.to_string(), v.to_str().unwrap().to_string());
    }

    let mut uri_query_params = HashMap::<String, String>::new();
    let query = req.uri().query();
    if let Some(query) = query {
        let querys: Vec<&str> = query.split("&").collect();
        for kv_pair in querys {
            let kvs: Vec<&str> = kv_pair.split("=").collect();
            uri_query_params.insert(kvs[0].to_string(), kvs[1].to_string());
        }
    }

    let mut uri_path_params = HashMap::<u8, String>::new();
    let path = req.uri().path();
    let paths: Vec<&str> = path.split("/").collect();
    for i in 0..paths.len() {
        uri_path_params.insert(i as u8, paths[i].to_string());
    }

    let mut params: Params = Default::default();

    params.uri = uri.name().to_string();
    params.if_info = IfInfo {
        action: uri.action().clone(),
        bulk_input: uri.bulk_input().clone(),
        bulk_output: uri.bulk_output().clone(),
    };

    if headers.len() > 0 {
        params.header = headers;
    }

    if uri_query_params.len() > 0 {
        params.query_param = uri_query_params;
    }

    if uri_path_params.len() > 0 {
        params.path_param = uri_path_params;
    }

    let body_bytes = req.collect().await?.to_bytes().to_vec();

    if body_bytes.is_empty() {
        params.body = None;
    } else {
        params.body = Some(body_bytes);
    }

    debug!(
        "============================accept param============================\nstart time: {}\n{:?}",
        utc_timestamp(),
        params
    );

    Ok(params)
}

fn de_bytes_slice<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> HttpResult<T> {
    let model = serde_json::from_slice::<T>(&bytes[..]);
    if let Err(err) = model {
        return Err(Box::new(gen_resp_err(CONVERT_TO_MODEL_ERROR, Some(err.to_string()))));
    };
    Ok(model.unwrap())
}

pub fn set_input_param<I: for<'de> Deserialize<'de> + ModelTrait + prost::Message + Default + Serialize>(
    param_map: &ExtraParamMap,
    params: &Params,
    input_param: &mut I,
) -> HttpResult<()> {
    for (target_name, param_def) in param_map.params.clone().into_iter() {
        let name = &param_def.name;
        match param_def.from {
            ParamFrom::Header => {
                if params.header.is_empty() {
                    return Err(Box::new(gen_resp_err(HEADER_NOT_FOUND, None)));
                }

                if param_def.required {
                    let Some(value) = params.header.get(name) else {
                        return Err(Box::new(gen_resp_err(HEADER_NOT_FOUND, Some(String::from(format!("header {name} not found"))))));
                    };
                    input_param.set_field(value.to_owned(), target_name.as_str())?;
                } else {
                    if let Some(value) = params.header.get(name) {
                        input_param.set_field(value.to_owned(), target_name.as_str())?;
                    };
                }
            }

            ParamFrom::Path => {
                let path_pos = name.parse::<u8>();
                if let Err(err) = path_pos {
                    return Err(Box::new(gen_resp_err(PARAM_MAP_PARSE_ERROR, Some(err.to_string()))));
                }

                if params.path_param.is_empty() {
                    return Err(Box::new(gen_resp_err(PATH_PARAM_NOT_EXIST, None)));
                }

                if param_def.required {
                    let Some(value) = params.path_param.get(&path_pos.unwrap()) else {
                        return Err(Box::new(gen_resp_err(
                            PATH_PARAM_NOT_EXIST,
                            Some(String::from(format!("path param {name} not found"))),
                        )));
                    };
                    input_param.set_field(value.to_owned(), target_name.as_str())?;
                } else {
                    if let Some(value) = params.path_param.get(&path_pos.unwrap()) {
                        input_param.set_field(value.to_owned(), target_name.as_str())?;
                    };
                }
            }

            ParamFrom::Query => {
                if !params.query_param.is_empty() {
                    return Err(Box::new(gen_resp_err(QUERY_PARAM_NOT_EXIST, None)));
                }

                if param_def.required {
                    let Some(value) = params.query_param.get(name) else {
                        return Err(Box::new(gen_resp_err(
                            PATH_PARAM_NOT_EXIST,
                            Some(String::from(format!("query parameter {name} not found"))),
                        )));
                    };
                    input_param.set_field(value.to_owned(), target_name.as_str())?;
                } else {
                    if let Some(value) = params.query_param.get(name) {
                        input_param.set_field(value.to_owned(), target_name.as_str())?;
                    };
                }
            }

            ParamFrom::Body => {
                if param_def.required {
                    match param_def.param_type {
                        ParamType::Vec => {}
                        ParamType::HashMap => {}
                        _ => {
                            input_param.get_field_str(param_def.name.as_str()).ok_or(gen_resp_err(
                                BODY_PARAM_NOT_EXIST,
                                Some(String::from(format!("body parameter {name} not found"))),
                            ))?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn params_to_model<
    I: for<'de> Deserialize<'de> + ModelTrait + prost::Message + Default + Serialize,
    O: for<'de> Deserialize<'de> + ModelTrait + prost::Message,
    C,
>(
    params: &Params,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let mut income_param_exist = true;
    let param_map = INCOME_PARAM_MAP.read().await;
    let param_map = param_map.get(&params.uri);
    if let None = param_map {
        income_param_exist = false;
    }

    let mut input_param;
    let mut input_params;

    if params.if_info.bulk_input {
        input_param = None;
        input_params = if let None = params.body {
            Some(Default::default())
        } else {
            let bytes = params.body.clone().unwrap();
            match params.header.get_key_value("Content-Type") {
                None => Some(de_bytes_slice::<BulkT<I>>(&bytes[..])?),
                Some((_, value)) => {
                    if value == "application/grpc" || value == "application/grpc+proto" {
                        Some(BulkT::<I>::decode(&bytes[..])?)
                    } else {
                        Some(de_bytes_slice::<BulkT<I>>(&bytes[..])?)
                    }
                }
            }
        };
    } else {
        input_params = None;
        input_param = if let None = params.body {
            Some(Default::default())
        } else {
            let bytes = params.body.clone().unwrap();
            match params.header.get_key_value("Content-Type") {
                None => Some(de_bytes_slice::<I>(&bytes[..])?),
                Some((_, value)) => {
                    if value == "application/grpc" || value == "application/grpc+proto" {
                        Some(I::decode(&bytes[..])?)
                    } else {
                        Some(de_bytes_slice::<I>(&bytes[..])?)
                    }
                }
            }
        };
    }

    let saga_id_ori = params.header.get("saga_id");
    let mut saga_id: Option<String> = None;
    if let Some(saga_id_ori) = saga_id_ori {
        saga_id = Some(saga_id_ori.to_owned());
    }

    let exec = HashMap::<String, (DaprRequest, DaprResponse, Option<Vec<Box<dyn DaprBody>>>)>::new();

    if !income_param_exist {
        debug!("input_param model: {:#?}", input_param);
        debug!("input_params model: {:#?}", input_params);

        return Ok(ContextWrapper {
            saga_id,
            uri_name: params.uri.clone(),
            if_info: params.if_info.clone(),
            input: input_param,
            inputs: input_params,
            exec,
            output: None,
            outputs: Vec::<O>::new(),
            exec_name: None,
            header: params.header.clone(),
            path_param: params.path_param.clone(),
            query_param: params.query_param.clone(),
            page_info: None,
            inner_context: None,
        });
    }

    let param_map = param_map.unwrap();

    if params.if_info.bulk_input {
        if let Some(input_params) = input_params.as_mut() {
            for input_param in input_params.bulk_data.iter_mut() {
                set_input_param(param_map, &params, input_param)?;
            }
        }
    } else {
        if let Some(input_param) = input_param.as_mut() {
            set_input_param(param_map, &params, input_param)?;
        }
    }

    debug!("input_param model: {:#?}", input_param);
    debug!("input_params model: {:#?}", input_params);

    Ok(ContextWrapper {
        saga_id,
        uri_name: params.uri.clone(),
        if_info: params.if_info.clone(),
        input: input_param,
        inputs: input_params,
        exec,
        output: None,
        outputs: Vec::<O>::new(),
        exec_name: None,
        header: params.header.clone(),
        path_param: params.path_param.clone(),
        query_param: params.query_param.clone(),
        page_info: None,
        inner_context: None,
    })
}

pub fn validate<I: ModelTrait + prost::Message + Validate + Default, O: ModelTrait + prost::Message, C>(
    context: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    if context.if_info.bulk_input {
        let bulk_data = &context.inputs.as_ref().ok_or("inputs param not found")?.bulk_data;

        for ele in bulk_data.iter() {
            ele.validate()?;
        }
    } else {
        let _ = &context.input.as_ref().ok_or("input param not found")?.validate()?;
    }

    Ok(context)
}

pub fn validate_naked<T: ModelTrait + Validate>(model: T) -> HttpResult<T> {
    let _ = &model.validate()?;
    Ok(model)
}

pub fn utc_timestamp() -> DateTime<Local> {
    Local::now()
}

pub async fn res<I: ModelTrait + Default + prost::Message, O: ModelTrait + Validator + prost::Message + std::default::Default, C>(
    context: ContextWrapper<I, O, C>,
) -> HttpResult<IfRes<O>> {
    let uris = URIS.read().await;
    let uri = uris.get(&context.uri_name).unwrap();

    let mut if_res: IfRes<O> = Default::default();
    if_res.saga_id = context.saga_id;
    if_res.uri_name = Some(uri.name().to_string());
    if_res.action = Some(uri.action().to_i32());
    if_res.bulk_output = context.if_info.bulk_output;
    if context.if_info.bulk_output {
        if_res.outputs = context.outputs;
        if_res.output = None;
    } else {
        if_res.outputs = Vec::<O>::new();
        if_res.output = context.output;
    }

    Ok(if_res)
}

pub async fn hyper_request(
    url: String,
    http_method: Method,
    body: Option<Vec<u8>>,
    headers: Option<HashMap<String, String>>,
) -> HttpResult<Response<Incoming>> {
    let hyper_url = match url.parse::<hyper::Uri>() {
        Err(err) => {
            return Err(Box::new(gen_resp_err(URL_PARSE_ERROR, Some(err.to_string()))));
        }
        Ok(url) => url,
    };

    match http_method {
        Method::GET | Method::POST | Method::PUT | Method::DELETE | Method::PATCH => {}

        _ => return Err(Box::new(gen_resp_err(REQUEST_METHOD_NOT_ALLOWED, None))),
    }

    debug!("[invoke] request to dapr: {:?}", &hyper_url);

    let mut builder = Request::builder()
        .method(http_method)
        .uri(hyper_url.clone())
        .header("Content-Type", "application/json");

    if let Some(headers) = headers {
        for (key, value) in headers {
            builder = builder.header(&key, &value);
        }
    }

    let req = if let None = body {
        builder.body(body::empty())
    } else {
        builder.body(body::bytes(body.unwrap()))
    };

    if let Err(err) = req {
        return Err(Box::new(gen_resp_err(DAPR_HTTP_REQ_BUILD_ERROR, Some(err.to_string()))));
    }

    let host = hyper_url.host().ok_or("uri has no host")?;
    let port = hyper_url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

    let stream = TcpStream::connect(addr).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            error!("Connection failed: {:?}", err);
        }
    });

    let res = sender.send_request(req.unwrap()).await?;

    debug!("response from dapr: {:?}", res);

    Ok(res)
}

pub fn de_any_json<T: for<'de> Deserialize<'de> + DaprBody + ModelTrait + Validate>(data: &prost_types::Any) -> HttpResult<Box<dyn DaprBody>> {
    let t = serde_json::from_slice::<T>(&data.value[..])?;
    Ok(Box::new(validate_naked(t)?))
}

pub fn de_any_prost<T: for<'de> Deserialize<'de> + prost::Message + Default + DaprBody + ModelTrait + Validate>(
    data: &prost_types::Any,
) -> HttpResult<Box<dyn DaprBody>> {
    let t = T::decode(&data.value[..])?;
    Ok(Box::new(validate_naked(t)?))
}

pub fn de_sql_result<T: Default + ModelTrait + Debug + DaprBody>(
    result_set: &[u8],
    columns: &Vec<String>,
    enum_flds: fn(&str, &str) -> HttpResult<(bool, Option<i32>)>,
) -> HttpResult<Vec<Box<dyn DaprBody>>> {
    if columns.is_empty() {
        return Err(Box::new(gen_resp_err(SQL_OUT_COLUMNS_IS_EMPTY, None)));
    }
    Ok(parse_dapr_body::<T>(result_set, columns, enum_flds)?)
}

pub fn de_sql_result_implicit<T: Default + ModelTrait + Debug + DaprBody>(
    result_set: &[u8],
    columns: &Vec<String>,
    enum_flds: fn(&str, &str) -> HttpResult<(bool, Option<i32>)>,
) -> HttpResult<Vec<T>> {
    if columns.is_empty() {
        return Err(Box::new(gen_resp_err(SQL_OUT_COLUMNS_IS_EMPTY, None)));
    }
    let vs = parse_dapr_body::<T>(result_set, columns, enum_flds)?;
    let mut n_vs = Vec::<T>::new();
    for mut v in vs {
        let n_v = v.downcast_mut::<T>().ok_or(format!("downcast fail"))?;
        n_vs.push(n_v.to_owned());
    }
    Ok(n_vs)
}

pub fn de_sql_result_implicit_first<T: Default + ModelTrait + Debug + DaprBody>(
    result_set: &[u8],
    columns: &Vec<String>,
    enum_flds: fn(&str, &str) -> HttpResult<(bool, Option<i32>)>,
) -> HttpResult<T> {
    if columns.is_empty() {
        return Err(Box::new(gen_resp_err(SQL_OUT_COLUMNS_IS_EMPTY, None)));
    }
    let mut vs = parse_dapr_body::<T>(result_set, columns, enum_flds)?;
    if vs.is_empty() {
        return Err(Box::new(gen_resp_err(DATA_NOT_FOUND, None)));
    }
    let n_v = vs[0].downcast_mut::<T>().ok_or(format!("downcast fail"))?;
    Ok(n_v.to_owned())
}

#[derive(Debug)]
struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Node { data, next: None }
    }
}

#[derive(Debug)]
struct Stack<T> {
    data: Option<Box<Node<T>>>,
    length: usize,
}

impl<T: Copy> Stack<T> {
    fn new() -> Self {
        Stack { data: None, length: 0 }
    }
    fn push(&mut self, data: T) {
        let mut new_node = Node::new(data);
        if self.data.is_some() {
            let head = self.data.take();
            new_node.next = head;
            self.data = Some(Box::new(new_node));
        } else {
            self.data = Some(Box::new(new_node));
        }
        self.length += 1
    }
    fn pop(&mut self) -> Option<T> {
        if let Some(ref mut head) = self.data {
            self.length -= 1;
            let data = head.data;
            self.data = head.next.take();
            return Some(data);
        }
        None
    }
    fn last(&mut self) -> Option<T> {
        if let Some(ref mut head) = self.data {
            return Some(head.data);
        }
        None
    }
    fn length(&self) -> usize {
        self.length
    }
}

fn parse_dapr_body<T: ModelTrait + Debug + Default + DaprBody>(
    body: &[u8],
    columns: &Vec<String>,
    enum_convert: fn(&str, &str) -> HttpResult<(bool, Option<i32>)>,
) -> HttpResult<Vec<Box<dyn DaprBody>>> {
    let utf8_str = String::from_utf8_lossy(&body);
    let chars = utf8_str.chars();

    let mut ts = Vec::<Box<dyn DaprBody>>::new();

    let mut t: T = Default::default();
    let mut stack = Stack::<char>::new();
    let mut pos = 0;
    let mut fd_val: Vec<char> = Vec::new();
    let mut back_slash: bool = false;

    for item in chars {
        match item {
            '[' => {
                if let Some(head) = stack.last() {
                    match head {
                        '"' => {
                            fd_val.push(item);
                        }
                        '[' => {
                            t.clear_model();
                            pos = 0;
                            stack.push(item);
                        }
                        ',' => {
                            stack.pop();
                            t.clear_model();
                            pos = 0;
                            stack.push(item);
                        }
                        _ => {
                            return Err(Box::new(gen_resp_err(DAPR_DATA_ILLEGAL, None)));
                        }
                    }
                } else {
                    stack.push(item);
                }
            }
            ']' => {
                if let Some(head) = stack.last() {
                    match head {
                        '"' => {
                            fd_val.push(item);
                        }
                        '[' => {
                            if stack.length() == 2 {
                                let mut value = fd_val.iter().collect::<String>();
                                let f_name = columns.get(pos).ok_or("field column not found")?;
                                value = match enum_convert(f_name, &value) {
                                    Ok((t, t_i32)) => match t {
                                        true => t_i32.ok_or("enum field i32 value not found")?.to_string(),
                                        false => value,
                                    },
                                    Err(err) => return Err(err),
                                };
                                t.set_field(value, f_name)?;

                                ts.push(Box::new(t.clone_model()));
                                t.clear_model();
                                fd_val.clear();
                                pos = 0;
                                stack.pop();
                            } else if stack.length() == 1 {
                                stack.pop();
                            }
                        }
                        ']' => {
                            stack.pop();
                        }
                        _ => {}
                    }
                } else {
                    return Err(Box::new(gen_resp_err(DAPR_DATA_ILLEGAL, None)));
                }
            }
            ',' => {
                if let Some(head) = stack.last() {
                    match head {
                        '"' => {
                            fd_val.push(item);
                        }
                        '[' => {
                            if stack.length() == 2 {
                                let mut value = fd_val.iter().collect::<String>();
                                let f_name = columns.get(pos).ok_or("field column not found")?;
                                value = match enum_convert(f_name, &value) {
                                    Ok((t, t_i32)) => match t {
                                        true => t_i32.ok_or("enum field i32 value not found")?.to_string(),
                                        false => value,
                                    },
                                    Err(err) => return Err(err),
                                };
                                t.set_field(value, f_name)?;
                                fd_val.clear();
                                pos += 1;
                            }
                        }
                        _ => {
                            return Err(Box::new(gen_resp_err(DAPR_DATA_ILLEGAL, None)));
                        }
                    }
                } else {
                    return Err(Box::new(gen_resp_err(DAPR_DATA_ILLEGAL, None)));
                }
            }
            '"' => {
                if let true = back_slash {
                    fd_val.push(item);
                    back_slash = false;
                    continue;
                }
                if let Some(head) = stack.last() {
                    match head {
                        '"' => {
                            stack.pop();
                        }
                        '[' => {
                            fd_val.clear();
                            stack.push(item);
                        }
                        _ => {
                            return Err(Box::new(gen_resp_err(DAPR_DATA_ILLEGAL, None)));
                        }
                    }
                } else {
                    return Err(Box::new(gen_resp_err(DAPR_DATA_ILLEGAL, None)));
                }
            }
            '\\' => {
                back_slash = true;
            }
            _ => {
                fd_val.push(item);
            }
        }
    }

    Ok(ts)
}

pub fn find_dapr_config(config_name: &str) -> HttpResult<DaprConfig> {
    Ok(DAPR_CONFIG
        .get(config_name)
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(String::from(config_name))))?
        .clone())
}

pub fn find_dapr_execute<'a>(
    exec: &'a mut HashMap<String, (DaprRequest, DaprResponse, Option<Vec<Box<dyn DaprBody>>>)>,
    execute_name: &'a str,
) -> HttpResult<&'a mut (DaprRequest, DaprResponse, Option<Vec<Box<dyn DaprBody>>>)> {
    Ok(exec
        .get_mut(execute_name)
        .ok_or(gen_resp_err(DAPR_EXECUTE_NOT_EXIST, Some(String::from(execute_name))))?)
}

pub fn set_dapr_req<I: ModelTrait + Message + Default, O: ModelTrait + Message, C>(
    mut context: ContextWrapper<I, O, C>,
    dapr_req: DaprRequest,
    execute_name: &str,
) -> HttpResult<ContextWrapper<I, O, C>> {
    context
        .exec
        .insert(execute_name.to_string(), (dapr_req, DaprResponse { ..Default::default() }, None));

    context.exec_name = Some(execute_name.to_string());

    Ok(context)
}

pub fn set_dapr_res<I: ModelTrait + Message + Default, O: ModelTrait + Message, C>(
    mut context: ContextWrapper<I, O, C>,
    dapr_res: Vec<Box<dyn DaprBody>>,
    execute_name: &str,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let exec = context
        .exec
        .get_mut(execute_name)
        .ok_or(gen_resp_err(DAPR_EXECUTE_NOT_EXIST, Some(String::from(execute_name))))?;

    exec.2 = Some(dapr_res);

    Ok(context)
}

pub fn find_invoke_service(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<InvokeServiceRequest> {
    Ok(dapr_config
        .clone()
        .invoke_service
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "invoke_service"))))?)
}

pub fn find_get_state(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<GetStateRequest> {
    Ok(dapr_config
        .clone()
        .get_state
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "get_state"))))?)
}

pub fn find_get_bulk_state(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<GetBulkStateRequest> {
    Ok(dapr_config
        .clone()
        .get_bulk_state
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "get_bulk_state"))))?)
}

pub fn find_query_state(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<QueryStateRequest> {
    Ok(dapr_config
        .clone()
        .query_state
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "query_state"))))?)
}

pub fn find_save_state(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<SaveStateRequest> {
    Ok(dapr_config
        .clone()
        .save_state
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "save_state"))))?)
}

pub fn find_transaction_state(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<ExecuteStateTransactionRequest> {
    Ok(dapr_config
        .clone()
        .transaction_state
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "transaction_state"))))?)
}

pub fn find_delete_state(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<DeleteStateRequest> {
    Ok(dapr_config
        .clone()
        .delete_state
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "delete_state"))))?)
}

pub fn find_delete_bulk_state(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<DeleteBulkStateRequest> {
    Ok(dapr_config
        .clone()
        .delete_bulk_state
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "delete_bulk_state"))))?)
}

pub fn find_invoke_binding(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<InvokeBindingRequest> {
    Ok(dapr_config
        .clone()
        .invoke_binding
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "invoke_binding"))))?)
}

pub fn find_invoke_binding_sql(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<InvokeBindingSqlRequest> {
    Ok(dapr_config
        .clone()
        .invoke_binding_sql
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "invoke_binding_sql"))))?)
}

pub fn find_publish_event(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<PublishEventRequest> {
    Ok(dapr_config
        .clone()
        .publish_event
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "publish_event"))))?)
}

pub fn find_publish_bulk_event(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<BulkPublishRequest> {
    Ok(dapr_config
        .clone()
        .publish_bulk_event
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "publish_bulk_event"))))?)
}

pub fn find_get_secret(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<GetSecretRequest> {
    Ok(dapr_config
        .clone()
        .get_secret
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "get_secret"))))?)
}

pub fn find_get_bluk_secret(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<GetBulkSecretRequest> {
    Ok(dapr_config
        .clone()
        .get_bluk_secret
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "get_bluk_secret"))))?)
}

pub fn find_get_configuration(dapr_config: &DaprRequest, config_name: &str) -> HttpResult<GetConfigurationRequest> {
    Ok(dapr_config
        .clone()
        .get_configuration
        .ok_or(gen_resp_err(DAPR_CONFIG_NOT_EXIST, Some(format!("{}.{}", config_name, "get_configuration"))))?)
}

static DIALECT: GenericDialect = GenericDialect {};

fn select_columns(sqls: &str) -> HttpResult<Vec<String>> {
    let mut columns = Vec::<String>::new();
    let mut ast = Parser::parse_sql(&DIALECT, sqls)?;

    if ast.len() != 1 {
        return Err(Box::new(gen_resp_err(QUERY_SQL_IS_NOT_UNIQUE, Some(String::from(sqls)))));
    }

    match ast.pop().unwrap() {
        Statement::Query(query) => match *query.body {
            SetExpr::Select(s) => {
                for item in s.projection.iter() {
                    match item {
                        SelectItem::ExprWithAlias { alias, .. } => columns.push(alias.value.clone()),
                        SelectItem::UnnamedExpr(expr) => match expr {
                            Expr::Identifier(ident) => columns.push(ident.value.clone()),
                            Expr::CompoundIdentifier(idents) => columns.push(idents.last().unwrap().value.clone()),
                            Expr::CompositeAccess { key, .. } => columns.push(key.value.clone()),
                            Expr::Named { name, .. } => columns.push(name.value.clone()),
                            // Expr::IsFalse(_)
                            // | Expr::IsNotFalse(_)
                            // | Expr::IsTrue(_)
                            // | Expr::IsNotTrue(_)
                            // | Expr::IsNull(_)
                            // | Expr::IsNotNull(_)
                            // | Expr::IsUnknown(_)
                            // | Expr::IsNotUnknown(_)
                            // | Expr::IsDistinctFrom(_, _)
                            // | Expr::IsNotDistinctFrom(_, _)
                            // | Expr::InList { .. }
                            // | Expr::JsonAccess { .. }
                            // | Expr::InUnnest { .. }
                            // | Expr::Between { .. }
                            // | Expr::BinaryOp { .. }
                            // | Expr::Like { .. }
                            // | Expr::ILike { .. }
                            // | Expr::SimilarTo { .. }
                            // | Expr::RLike { .. }
                            // | Expr::AnyOp { .. }
                            // | Expr::AllOp { .. }
                            // | Expr::UnaryOp { .. }
                            // | Expr::Cast { .. }
                            // | Expr::TryCast { .. }
                            // | Expr::SafeCast { .. }
                            // | Expr::AtTimeZone { .. }
                            // | Expr::Extract { .. }
                            // | Expr::Ceil { .. }
                            // | Expr::Floor { .. }
                            // | Expr::Position { .. }
                            // | Expr::Substring { .. }
                            // | Expr::Trim { .. }
                            // | Expr::Overlay { .. }
                            // | Expr::Collate { .. }
                            // | Expr::IntroducedString { .. }
                            // | Expr::TypedString { .. }
                            // | Expr::MapAccess { .. }
                            // | Expr::AggregateExpressionWithFilter { .. }
                            // | Expr::Case { .. }
                            // | Expr::Exists { .. }
                            // | Expr::Struct { .. }
                            // | Expr::ArrayIndex { .. }
                            // | Expr::MatchAgainst { .. }
                            // | Expr::Nested(_)
                            // | Expr::Value(_)
                            // | Expr::Function(_)
                            // | Expr::Subquery(_)
                            // | Expr::ArraySubquery(_)
                            // | Expr::ListAgg(_)
                            // | Expr::ArrayAgg(_)
                            // | Expr::GroupingSets(_)
                            // | Expr::Cube(_)
                            // | Expr::Rollup(_)
                            // | Expr::Tuple(_)
                            // | Expr::Array(_)
                            // | Expr::Interval(_)
                            // | Expr::InSubquery { .. } => {}
                            _ => {}
                        },
                        SelectItem::QualifiedWildcard(..) | SelectItem::Wildcard(_) => {
                            return Err(Box::new(gen_resp_err(SQL_NOT_SUPPORT, Some(format!("{}. {}", "*", sqls)))));
                        }
                    };
                }
            }
            SetExpr::Query(_) => {}
            _ => {}
        },
        _ => {
            return Err(Box::new(gen_resp_err(SQL_NOT_VALID, Some(String::from(sqls)))));
        }
    };

    Ok(columns)
}

pub fn trans_sql_info(
    sqls_tuple: Vec<(String, Vec<rbs::Value>, bool, Option<u64>, Option<u64>)>,
    operation: SqlOperation,
    dapr_config: &DaprConfig,
) -> HttpResult<Vec<SqlWithParams>> {
    let mut res: Vec<SqlWithParams> = vec![];
    match operation {
        SqlOperation::Query => {
            if sqls_tuple.len() != 1 {
                return Err(Box::new(gen_resp_err(DAPR_DATA_ILLEGAL, Some(String::from("query action have only 1 sql.")))));
            }
            let mut deed = de_paramize(sqls_tuple)?;
            let (sql, vs, is_page, offset, page_size) = deed.get_mut(0).unwrap();
            let output_columns = select_columns(&sql)?;
            if !sql.ends_with(";") {
                sql.push(';');
            }
            res.push(SqlWithParams {
                sql: sql.clone(),
                output_columns,
                params: format!("[{}]", parse_sql_params(vs)?),
                is_page: is_page.clone(),
                offset: offset.clone(),
                page_size: page_size.clone(),
            });
        }
        SqlOperation::QueryPage => {
            if sqls_tuple.len() != 2 {
                return Err(Box::new(gen_resp_err(DAPR_DATA_ILLEGAL, Some(String::from("query with page have 2 sqls")))));
            }
            let mut deed = de_paramize(sqls_tuple)?;
            let mut page_sqls = Vec::<(String, Vec<rbs::Value>, bool, Option<u64>, Option<u64>)>::new();
            let mut query_sqls = Vec::<(String, Vec<rbs::Value>, bool, Option<u64>, Option<u64>)>::new();
            deed.iter_mut().for_each(|item| {
                if item.2 {
                    page_sqls.push(item.to_owned());
                } else {
                    query_sqls.push(item.to_owned());
                }
            });
            if page_sqls.len() != 1 {
                return Err(Box::new(gen_resp_err(
                    DAPR_DATA_ILLEGAL,
                    Some(String::from("query page must have 1 sql that return the total `count`")),
                )));
            }
            if query_sqls.len() != 1 {
                return Err(Box::new(gen_resp_err(
                    DAPR_DATA_ILLEGAL,
                    Some(String::from("query page must have 1 sql that return the query content")),
                )));
            }
            let (page_sql, vs, is_page, offset, page_size) = page_sqls.get_mut(0).unwrap();
            if let None = offset {
                return Err(Box::new(gen_resp_err(
                    DAPR_DATA_ILLEGAL,
                    Some(String::from("page sql must have `offset` param")),
                )));
            }
            if let None = page_size {
                return Err(Box::new(gen_resp_err(
                    DAPR_DATA_ILLEGAL,
                    Some(String::from("page sql must have `page_size` param")),
                )));
            }
            let output_columns = select_columns(&page_sql)?;
            if !page_sql.ends_with(";") {
                page_sql.push(';');
            }
            res.push(SqlWithParams {
                sql: page_sql.clone(),
                output_columns,
                params: format!("[{}]", parse_sql_params(vs)?),
                is_page: is_page.clone(),
                offset: offset.clone(),
                page_size: page_size.clone(),
            });

            let (query_sql, vs, is_page, offset, page_size) = query_sqls.get_mut(0).unwrap();
            let output_columns = select_columns(&query_sql)?;
            if !query_sql.ends_with(";") {
                query_sql.push(';');
            }
            res.push(SqlWithParams {
                sql: query_sql.clone(),
                output_columns,
                params: format!("[{}]", parse_sql_params(vs)?),
                is_page: is_page.clone(),
                offset: offset.clone(),
                page_size: page_size.clone(),
            });
        }
        SqlOperation::Exec => {
            if sqls_tuple.is_empty() {
                return Err(Box::new(gen_resp_err(
                    DAPR_DATA_ILLEGAL,
                    Some(String::from("exec action must have at least 1 sql")),
                )));
            }
            let mut deed = de_paramize(sqls_tuple)?;

            let mut tx_sql = String::from("");
            let mut tx_params = String::from("");

            tx_params.push_str("[");

            let mut is_first = true;

            for (sql, vs, _, _, _) in deed.iter_mut() {
                tx_sql.push_str(&sql);
                if !sql.ends_with(";") {
                    tx_sql.push_str(";");
                }
                if is_first {
                    is_first = false;
                } else {
                    tx_params.push_str(",");
                }
                tx_params.push_str(&parse_sql_params(vs)?);
            }

            tx_params.push_str("]");

            res.push(SqlWithParams {
                sql: tx_sql.clone(),
                output_columns: vec![],
                params: tx_params,
                is_page: false,
                offset: None,
                page_size: None,
            });
        }
        SqlOperation::ExecTransaction => {
            if sqls_tuple.len() == 0 {
                return Err(Box::new(gen_resp_err(DAPR_DATA_ILLEGAL, Some(String::from("exec action have 1 sql at least")))));
            }

            let mut deed = de_paramize(sqls_tuple)?;

            let mut tx_sql = String::from("");
            let mut tx_params = String::from("");

            tx_sql.push_str("BEGIN;");
            tx_params.push_str("[");

            let mut is_first = true;

            for (sql, vs, _, _, _) in deed.iter_mut() {
                tx_sql.push_str(&sql);
                if !sql.ends_with(";") {
                    tx_sql.push_str(";");
                }
                if is_first {
                    is_first = false;
                } else {
                    tx_params.push_str(",");
                }
                tx_params.push_str(&parse_sql_params(vs)?);
            }

            tx_sql.push_str("COMMIT;");
            tx_params.push_str("]");
            res.push(SqlWithParams {
                sql: tx_sql,
                output_columns: vec![],
                params: tx_params,
                is_page: false,
                offset: None,
                page_size: None,
            });
        }
    }

    match &dapr_config.binding {
        None => {}
        Some(config) => {
            if config.component_type == "bindings.postgresql" {
                for sql_with_param in res.iter_mut() {
                    let mut new_sql = String::from("");
                    let chars = sql_with_param.sql.clone().chars().collect::<Vec<char>>();
                    let mut index = 1;
                    for c in chars.iter() {
                        if c == &'?' {
                            new_sql.push('$');
                            new_sql.push_str(&index.to_string());
                            index += 1;
                        } else {
                            new_sql.push(*c);
                        }
                    }
                    sql_with_param.sql = new_sql;
                }
            }
        }
    }

    Ok(res)
}

fn parse_sql_params(params: &mut Vec<rbs::Value>) -> HttpResult<String> {
    let vs: Vec<String> = params.iter().map(|v| v.to_string()).collect();
    let params = format!("{}", vs.join(","));
    Ok(params)
}

fn de_paramize(
    mut sqls: Vec<(String, Vec<rbs::Value>, bool, Option<u64>, Option<u64>)>,
) -> HttpResult<Vec<(String, Vec<rbs::Value>, bool, Option<u64>, Option<u64>)>> {
    let mut new_values = Vec::<(String, Vec<rbs::Value>, bool, Option<u64>, Option<u64>)>::new();
    for (sql, values, is_page, offset, page_size) in sqls.iter_mut() {
        let mut new_sql = String::from("");
        let mut index = 0;
        for c in sql.chars() {
            if c == '?' {
                if values[index].is_str() {
                    new_sql.push('\'');
                    new_sql.push_str(values[index].as_str().unwrap());
                    new_sql.push('\'');
                } else if values[index].is_null() {
                    new_sql.push_str("null");
                } else {
                    new_sql.push_str(String::from_utf8_lossy(&values[index].clone().into_bytes().unwrap()).to_string().as_str());
                }
                index += 1;
            } else {
                new_sql.push(c);
            }
        }

        new_values.push((new_sql, values.clone(), is_page.clone(), offset.clone(), page_size.clone()));
    }

    Ok(new_values)
}

pub async fn find_response_auth_header(params: &Params) -> HttpResult<(Option<String>, Option<String>)> {
    let skip_ifs = SKIP_AUTH_IFS.read().await;
    if skip_ifs.contains(&params.uri) {
        return Ok((None, None));
    }

    let tag = INTERNAL_AUTH_TAG.read().await;

    if let None = *tag {
        return Err(Box::new(gen_resp_err(INTERNAL_AUTH_TAG_NOT_SET, None)));
    }

    if params.header.contains_key(AuthHeader::XSGAuthInternal.lower_case_value()) {
        return Ok((Some(tag.clone().unwrap()), Some(tag.clone().unwrap())));
    } else if params.header.contains_key(AuthHeader::XSGAuthInternal.upper_case_value()) {
        return Ok((Some(tag.clone().unwrap()), Some(tag.clone().unwrap())));
    } else if params.header.contains_key(AuthHeader::XSGAuthJWT.lower_case_value()) {
        return Ok((
            Some(AuthHeader::XSGAuthJWT.lower_case_value().to_string()),
            Some(params.header.get(AuthHeader::XSGAuthJWT.lower_case_value()).unwrap().to_owned()),
        ));
    } else if params.header.contains_key(AuthHeader::XSGAuthJWT.upper_case_value()) {
        return Ok((
            Some(AuthHeader::XSGAuthJWT.upper_case_value().to_string()),
            Some(params.header.get(AuthHeader::XSGAuthJWT.upper_case_value()).unwrap().to_owned()),
        ));
    } else if params.header.contains_key(AuthHeader::XSGAuthBasic.lower_case_value()) {
    } else if params.header.contains_key(AuthHeader::XSGAuthBasic.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthOAuth2.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthOAuth2.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthAksk.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthAksk.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthApiKey.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthApiKey.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthDigestAuth.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthDigestAuth.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthOIDC.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthOIDC.upper_case_value()) {
        todo!();
    } else {
        return Err(Box::new(gen_resp_err(AUTH_ERROR, Some(String::from("at least one auth type needed")))));
    }

    return Ok((None, None));
}

pub async fn auth_ict(params: &mut Params) -> HttpResult<()> {
    let skip_ifs = SKIP_AUTH_IFS.read().await;
    if skip_ifs.contains(&params.uri) {
        return Ok(());
    }

    let setted_tag = INTERNAL_AUTH_TAG.read().await;

    if let None = *setted_tag {
        return Err(Box::new(gen_resp_err(INTERNAL_AUTH_TAG_NOT_SET, None)));
    }

    if params.header.contains_key(AuthHeader::XSGAuthInternal.lower_case_value()) {
        let internal_tag = params.header.get(AuthHeader::XSGAuthInternal.lower_case_value());
        match internal_tag {
            None => {
                return Err(Box::new(gen_resp_err(AUTH_ERROR, Some(String::from("internal auth value not found")))));
            }
            Some(tag) => {
                if setted_tag.as_ref().unwrap().ne(tag) {
                    return Err(Box::new(gen_resp_err(AUTH_ERROR, Some(String::from("internal auth fail")))));
                } else {
                    return Ok(());
                }
            }
        }
    } else if params.header.contains_key(AuthHeader::XSGAuthInternal.upper_case_value()) {
        let internal_tag = params.header.get(AuthHeader::XSGAuthInternal.upper_case_value());
        match internal_tag {
            None => {
                return Err(Box::new(gen_resp_err(AUTH_ERROR, Some(String::from("internal auth tag value not found")))));
            }
            Some(tag) => {
                if setted_tag.as_ref().unwrap().ne(tag) {
                    return Err(Box::new(gen_resp_err(AUTH_ERROR, Some(String::from("internal auth fail")))));
                } else {
                    return Ok(());
                }
            }
        }
    } else if params.header.contains_key(AuthHeader::XSGAuthJWT.lower_case_value()) {
        let jwt_value = params.header.get(AuthHeader::XSGAuthJWT.lower_case_value());
        match jwt_value {
            None => {
                return Err(Box::new(gen_resp_err(AUTH_ERROR, Some(String::from("jwt auth value not found")))));
            }
            Some(jwt_token) => {
                let token = auth(jwt_token).await.map_err(|err| {
                    error!("auth error: {}", err);
                    return Box::new(gen_resp_err(AUTH_ERROR, Some(err.to_string())));
                })?;
                params.header.insert(AuthHeader::XSGAuthJWT.lower_case_value().to_string(), token);
                return Ok(());
            }
        }
    } else if params.header.contains_key(AuthHeader::XSGAuthJWT.upper_case_value()) {
        let jwt_value = params.header.get(AuthHeader::XSGAuthJWT.upper_case_value());
        match jwt_value {
            None => {
                return Err(Box::new(gen_resp_err(AUTH_ERROR, Some(String::from("jwt auth value not found")))));
            }
            Some(jwt_token) => {
                let token = auth(jwt_token).await.map_err(|err| {
                    error!("auth result : {}", err);
                    return Box::new(gen_resp_err(AUTH_ERROR, Some(err.to_string())));
                })?;
                params.header.insert(AuthHeader::XSGAuthJWT.upper_case_value().to_string(), token);
                return Ok(());
            }
        }
    } else if params.header.contains_key(AuthHeader::XSGAuthBasic.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthBasic.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthOAuth2.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthOAuth2.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthAksk.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthAksk.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthApiKey.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthApiKey.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthDigestAuth.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthDigestAuth.upper_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthOIDC.lower_case_value()) {
        todo!();
    } else if params.header.contains_key(AuthHeader::XSGAuthOIDC.upper_case_value()) {
        todo!();
    } else {
        return Err(Box::new(gen_resp_err(AUTH_ERROR, Some(String::from("at least one auth type needed")))));
    }
}

async fn auth(token: &String) -> HttpResult<String> {
    let execute_name = "auth-jwt";
    let mut dapr_req_ins = DaprRequest::make_invoke_service(
        "auth-serverlessguide-dev".to_string(),
        "/auth/approve".to_string(),
        "application/json".to_string(),
        MethodEnum::POST,
        "".to_string(),
    )?;
    let invoke_service = dapr_req_ins.invoke_service.as_mut().ok_or("invoke_service make error")?;
    let message = invoke_service.message.as_mut().ok_or("invoke_service message not found")?;
    let data = JwtToken {
        token: Some(token.to_string()),
    };
    message.data = Some(prost_types::Any {
        type_url: "".to_string(),
        value: serde_json::json!(data).to_string().as_bytes().to_vec(),
    });
    let setted_tag = INTERNAL_AUTH_TAG.read().await;
    message.headers.insert(
        AuthHeader::XSGAuthInternal.upper_case_value().to_string(),
        setted_tag
            .as_ref()
            .ok_or(gen_resp_err(AUTH_ERROR, Some(String::from("internal auth tag value not found"))))?
            .to_string(),
    );

    let context: ContextWrapper<EmptyInPut, EmptyOutPut, EmptyInnerContext> = Default::default();
    let context = set_dapr_req(context, dapr_req_ins, execute_name)?;
    let mut context = invoke_service_http(context).await?;

    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;
    let response = res
        .invoke_service
        .clone()
        .ok_or(format!("execute '{}' of invoke_service response not found", execute_name))?;

    let Some(data) = &response.data else {
        return Err(Box::new(gen_resp_err(DATA_NOT_FOUND, Some(String::from("response data is empty from auth")))));
    };

    let token = serde_json::from_slice::<Res<IfRes<JwtToken>>>(&data.value[..])?;
    if token.message.ne("success") {
        return Err(Box::new(gen_resp_err(
            DAPR_REQUEST_FAIL,
            Some(format!(
                "response data from auth validate error: {}, response data: {}",
                token.message,
                json!(token)
            )),
        )));
    }

    Ok(token
        .result
        .ok_or("result is empty")?
        .output
        .ok_or("result output is empty")?
        .token
        .ok_or("result output token string is empty")?)
}
