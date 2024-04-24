mod biz;
mod biz_model;

pub use bevy_reflect::{GetField, Reflect};
pub use biz_model::*;
pub use model_macro::*;
pub use model_macro_derive::*;
pub use pipe_trait::*;
pub use rbatis::crud;
pub use serde::*;
pub use sg_sdk_inner::{config::*, daprs::*, log::*, model::*, start::*, util::*, *};
pub use std::collections::*;
pub use std::str::FromStr;
pub use tracing::{debug, error, info, trace, warn};
pub use validator::Validate;
pub use validator_derive::Validate;

#[macro_use]
extern crate lazy_static;
extern crate rbatis;

async fn query_all_sms(params: &Params) -> HttpResult<IfRes<StorageModelInfo>> {
    params
        .pipe(util::params_to_model::<DBStorageModel, StorageModelInfo, UserWithIdSid>)
        .await?
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
        .await
}

async fn query_one_by_id(params: &Params) -> HttpResult<IfRes<StorageModelInfo>> {
    params
        .pipe(util::params_to_model::<DBStorageModel, StorageModelInfo, UserWithIdSid>)
        .await?
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
        .await
}

#[tokio::main]
async fn main() -> HttpResult<()> {
    init_log();

    // 一定要设置，不设置的话，会使用默认值-1，-1在运行时会报错
    biz_code_prefix!(1024);
    register_biz_result!(DATA_ERROR_1,);

    // start_http(8080).await
    // start_grpc(8088).await

    start_http_grpc(8080, 8088).await
}

biz_result! {(DATA_ERROR_1, 500, 27, "data error");}
