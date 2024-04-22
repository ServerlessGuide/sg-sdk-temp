mod biz;
mod biz_model;
mod config;

use biz::*;
use biz_model::*;
use config::*;
use model_macro::ModelTrait;
use pipe_trait::*;
use sg_sdk_temp::config::*;
use sg_sdk_temp::daprs::*;
use sg_sdk_temp::envs::*;
use sg_sdk_temp::log::*;
use sg_sdk_temp::model::*;
use sg_sdk_temp::start::*;
use sg_sdk_temp::util::*;
use sg_sdk_temp::*;
use tracing::{debug, error, info, trace, warn};

#[macro_use]
extern crate lazy_static;
extern crate rbatis;

async fn query_all_sms(params: &Params) -> HttpResult<IfRes<StorageModelInfo>> {
    params
        .pipe(util::params_to_model::<DBStorageModel, StorageModelInfo, UserWithIdSid>)?
        .pipe(util::validate)?
        .pipe(biz::prepare_inner_context)?
        .pipe(biz::pre_check_user_for_query_all)?
        .pipe(daprs::invoke_binding_grpc_sql)
        .await?
        .pipe(biz::post_check_user_for_query_all)?
        .pipe(biz::pre_query_all_file)?
        .pipe(daprs::invoke_binding_grpc)
        .await?
        .pipe(biz::post_query_all_file)
        .await?
        .pipe(util::res)
}

async fn query_one_by_id(params: &Params) -> HttpResult<IfRes<StorageModelInfo>> {
    params
        .pipe(util::params_to_model::<DBStorageModel, StorageModelInfo, UserWithIdSid>)?
        .pipe(util::validate)?
        .pipe(biz::prepare_inner_context)?
        .pipe(biz::pre_check_user_for_query_by_id)?
        .pipe(daprs::invoke_binding_grpc_sql)
        .await?
        .pipe(biz::post_check_user_for_query_by_id)?
        .pipe(biz::pre_query_one_by_id_sql)?
        .pipe(daprs::invoke_binding_grpc_sql)
        .await?
        .pipe(biz::post_query_one_by_id_sql)?
        .pipe(biz::pre_query_one_by_id)?
        .pipe(daprs::invoke_binding_grpc)
        .await?
        .pipe(biz::post_query_one_by_id)
        .await?
        .pipe(util::res)
}

#[tokio::main]
async fn main() -> HttpResult<()> {
    init();
    init_log();

    // start_http(8080).await
    // start_grpc(8088).await

    start_http_grpc(8080, 8088).await
}
