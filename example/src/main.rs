mod biz;
mod biz_model;

pub use bevy_reflect::{GetField, Reflect};
pub use biz_model::*;
use dapr::appcallback::InvokeResponse;
pub use model_macro::*;
pub use model_macro_derive::*;
pub use pipe_trait::*;
pub use rbatis::crud;
pub use serde::*;
pub use sg_sdk_inner::{config::*, daprs::*, log::*, model::*, start::*, util::*, *};
use sg_sdk_macro::*;
pub use std::collections::*;
pub use std::str::FromStr;
pub use tracing::{debug, error, info, trace, warn};
pub use validator::Validate;
pub use validator_derive::Validate;

#[macro_use]
extern crate lazy_static;
extern crate rbatis;

async fn query_by_app_id(params: &Params) -> HttpResult<IfRes<AppVersion>> {
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

async fn insert(params: &Params) -> HttpResult<IfRes<EmptyOutPut>> {
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

async fn env_prepare(params: &Params) -> HttpResult<IfRes<EmptyOutPut>> {
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

    // start_http::<ForConfig>(8080).await
    // start_grpc::<ForConfig>(8088).await

    start_http_grpc::<ForConfig>(8080, 8088).await
}

uri! {
    ForConfig,
    (QUERY_BY_APP_ID, GET, "^/app-version/\\d{19}$", Query, false, true);
    (INSERT, POST, "^/app-version$", Insert, false, false);
    (ENV_PREPARE, GET, "^/app-version/\\d{19}/env-prepare$", Function, false, false);
}

income_param! {
    ForConfig,
    (QUERY_BY_APP_ID, [(app_id, 2, Path, Number, true)]);
    (INSERT, [(app_id, app_id, Body, Number, true), (version, version, Body, String, true)]);
    (ENV_PREPARE, [(app_id, 2, Path, Number, true)]);
}

#[biz_result_handler(1002,<CUSTOM_BIZ_RES, 500, 41, "custom biz result message">)]
#[uri_handler(QUERY_BY_APP_ID => query_by_app_id, INSERT => insert, ENV_PREPARE => env_prepare)]
#[derive(Copy, Clone)]
struct ForConfig();
