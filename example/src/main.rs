mod biz;
mod biz_model;

use bevy_reflect::{GetField, Reflect};
use biz_model::*;
use dapr::appcallback::InvokeResponse;
use http_body_util::Either;
use pipe_trait::*;
use rbatis::*;
use serde::*;
use sg_sdk_inner::{daprs::*, log::*, model::*, start::*, traits::*, util::*, *};
use sg_sdk_macro::*;
use std::collections::*;
use std::str::FromStr;
use tracing::{debug, error, info, trace, warn};
use validator::Validate;
use validator_derive::Validate;

#[macro_use]
extern crate lazy_static;
extern crate rbatis;

async fn query_by_app_id(params: &Params) -> HttpResult<(IfRes<AppVersion>, HashMap<String, String>)> {
    params
        .pipe(util::params_to_model::<QueryAppVersions, AppVersion, UserWithIdSid>)
        .await?
        .pipe(util::validate)?
        .pipe(biz::prepare_inner_context_for_query_by_app_id)?
        .pipe(biz::pre_check_permission)?
        .pipe(daprs::invoke_binding_grpc_sql)
        .await?
        .pipe(biz::post_check_permission)?
        .pipe(biz::pre_query_by_app_id)?
        .pipe(daprs::invoke_binding_grpc_sql)
        .await?
        .pipe(biz::post_query_by_app_id)?
        .pipe(util::res)
        .await
}

async fn insert(params: &Params) -> HttpResult<(IfRes<EmptyOutPut>, HashMap<String, String>)> {
    params
        .pipe(util::params_to_model::<AppVersion, EmptyOutPut, UserWithIdSid>)
        .await?
        .pipe(util::validate)?
        .pipe(biz::prepare_inner_context_for_insert)?
        .pipe(biz::pre_check_permission_for_insert)?
        .pipe(daprs::invoke_binding_grpc_sql)
        .await?
        .pipe(biz::post_check_permission_for_insert)?
        .pipe(biz::pre_get_snowflake_id)?
        .pipe(daprs::invoke_service_http)
        .await?
        .pipe(biz::pre_insert)?
        .pipe(daprs::invoke_binding_grpc_sql)
        .await?
        .pipe(biz::post_insert)?
        .pipe(util::res)
        .await
}

async fn env_prepare(params: &Params) -> HttpResult<(IfRes<EmptyOutPut>, HashMap<String, String>)> {
    params
        .pipe(util::params_to_model::<AppVersion, EmptyOutPut, UserWithIdSid>)
        .await?
        .pipe(util::validate)?
        .pipe(biz::prepare_inner_context_for_insert)?
        .pipe(biz::pre_check_permission_for_env_prepare)?
        .pipe(daprs::invoke_binding_grpc_sql)
        .await?
        .pipe(biz::post_check_permission_for_env_prepare)?
        .pipe(biz::pre_query_by_app_version_id)?
        .pipe(daprs::invoke_binding_grpc_sql)
        .await?
        .pipe(biz::post_query_by_app_version_id)?
        .pipe(biz::pre_prepare_env)?
        .pipe(daprs::invoke_binding_grpc)
        .await?
        .pipe(biz::post_prepare_env)?
        .pipe(util::res)
        .await
}

#[tokio::main]
async fn main() -> HttpResult<()> {
    init_log();

    ForConfig::insert_uri().await?; // 和 uri! 成对出现
    ForConfig::insert_biz_result().await?; // #[biz_result_handler
    ForConfig::insert_income_param().await?; // income_param!
    // ForConfig::set_skip_auth_uri().await?; // skip_auth_uri!
    ForConfig::set_internal_auth_tag().await?; // internal_auth_tag!

    start_http_grpc::<ForConfig>(8080, 8088).await
}

internal_auth_tag!(ForConfig, "Serverless-Guide");

// skip_auth_uri!(ForConfig, (INSERT, QUERY_BY_APP_ID));

uri! {
    ForConfig,
    (QUERY_BY_APP_ID, GET, "^/example/\\d{19}$", Query, false, true);
    (INSERT, POST, "^/example$", Insert, false, false);
    (ENV_PREPARE, GET, "^/example/\\d{19}/env-prepare$", Function, false, false);
}

income_param! {
    ForConfig,
    (QUERY_BY_APP_ID, [(app_id, 2, Path, Number, true)]);
    (INSERT, [(app_id, app_id, Body, Number, true), (version, version, Body, String, true)]);
    (ENV_PREPARE, [(app_id, 2, Path, Number, true)]);
}

#[biz_result_handler(1002,<CUSTOM_BIZ_RES, 500, 41, "custom biz result message">)] // 依据业务情况， 100241
#[uri_handler(QUERY_BY_APP_ID => query_by_app_id, INSERT => insert, ENV_PREPARE => env_prepare)] // 固定
#[derive(Copy, Clone)] // 固定
struct ForConfig();
