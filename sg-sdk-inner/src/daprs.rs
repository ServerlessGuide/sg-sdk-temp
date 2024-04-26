use std::{collections::HashMap, str::FromStr};

use crate::model::{DaprBody, InvokeBindingSqlResponse, SqlOperation, SqlResponse};
use crate::traits::ModelTrait;
use crate::util::{err_boxed, err_boxed_full, err_boxed_full_string};
use crate::{
    config::get_dapr_client,
    inner_biz_result::*,
    model::{ContextWrapper, DaprRequest, DaprResponse},
    util::{self, err_full, hyper_request},
};
use crate::{HttpResult, ENVS};
use dapr::dapr::dapr::proto::common::v1::state_options::{StateConcurrency, StateConsistency};
use dapr::dapr::dapr::proto::{common::v1::InvokeResponse, runtime::v1::*};
use futures_util::future::join;
use http_body_util::BodyExt;
use hyper::{header, Method, StatusCode};
use tracing::{debug, error, info, trace, warn};

pub fn check_env_value(value: &str) -> HttpResult<&String> {
    let Some(dapr_host) = ENVS.get(value) else {
        return Err(err_boxed_full_string(ENV_PARAMETER_ERROR, format!("env param {} not found", value)));
    };
    Ok(dapr_host)
}

fn dapr_get_bulk_state_url_http(dapr_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/state/{1}/bulk", dapr_host, dapr_name,))
}

fn dapr_invoke_service_url_http(app_id: &str, method: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/invoke/{1}/method/{2}", dapr_host, app_id, method))
}

fn dapr_delete_or_get_state_url_http(store_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/state/{1}/", dapr_host, store_name))
}

fn dapr_delete_bulk_state_url_http(store_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/state/{1}/bulk", dapr_host, store_name))
}

fn dapr_query_state_url_http(dapr_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0-alpha1/state/{1}/query", dapr_host, dapr_name,))
}

fn dapr_save_state_url_http(store_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/state/{1}", dapr_host, store_name,))
}

fn dapr_transaction_state_url_http(store_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/state/{1}/transaction", dapr_host, store_name,))
}

fn dapr_invoke_binding_url_http(binding_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/bindings/{1}", dapr_host, binding_name,))
}

fn dapr_publish_event_url_http(pubsub_name: &str, topic: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/publish/{1}/{2}", dapr_host, pubsub_name, topic))
}

fn dapr_publish_bulk_url_http(pubsub_name: &str, topic: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0-alpha1/publish/bulk/{1}/{2}", dapr_host, pubsub_name, topic))
}

fn dapr_get_secret_url_http(secret_store_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/secrets/{1}/", dapr_host, secret_store_name))
}

fn dapr_get_bulk_secret_url_http(secret_store_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/secrets/{1}/bulk", dapr_host, secret_store_name))
}

fn dapr_get_configuration_url_http(configuration_store_name: &str) -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    Ok(format!("http://{0}:3500/v1.0/configuration/{1}", dapr_host, configuration_store_name))
}

pub fn dapr_url_grpc() -> HttpResult<String> {
    let dapr_host = check_env_value("DAPR_HOST")?;

    let dapr_port = check_env_value("DAPR_GRPC_PORT")?;

    Ok(format!("http://{0}:{1}", dapr_host, dapr_port))
}

fn find_dapr_execute<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: &ContextWrapper<I, O, C>,
) -> HttpResult<&(DaprRequest, DaprResponse, Option<Vec<Box<dyn DaprBody>>>)> {
    let Some(exec_name) = &cw.exec_name else {
        return Err(err_boxed_full(EXEC_NAME_NOT_EXIST, "ContextWrapper.dapr_execute_name"));
    };
    let Some(execute) = cw.exec.get(exec_name) else {
        return Err(err_boxed_full_string(
            DAPR_EXECUTE_NOT_EXIST,
            format!("ContextWrapper.dapr_execute[{}]", exec_name),
        ));
    };

    Ok(execute)
}

fn append_metadata_to_url(mut url: String, metadata: &HashMap<String, String>) -> String {
    if !metadata.is_empty() {
        url.push_str("?");
        metadata.into_iter().for_each(|(k, v)| {
            url.push_str("metadata.");
            url.push_str(k.as_str());
            url.push_str("=");
            url.push_str(v.as_str());
            url.push_str("&");
        });
    }
    url
}

pub async fn invoke_service_grpc<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.invoke_service {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.invoke_service"));
        }
    };

    let Some(message) = &config.message else {
        return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.invoke_service.message"));
    };

    let response = get_dapr_client()
        .await?
        .invoke_service(config.id.to_owned(), message.method.to_owned(), message.data.to_owned())
        .await;

    debug!("invoke dapr service '{} {}' response: {:?}", config.id, message.method, response);

    if let Err(err) = response {
        return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
    }
    let response = response.unwrap();

    dapr_execute.invoke_service = Some(response);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn invoke_service_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.invoke_service {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.invoke_service"));
        }
    };

    let Some(message) = &config.message else {
        return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.invoke_service.message"));
    };

    let Some(http_extension) = &message.http_extension else {
        return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.invoke_service.message.http_extension"));
    };

    let mut url = dapr_invoke_service_url_http(&config.id, &message.method)?;
    if !&http_extension.querystring.starts_with("?") {
        url.push_str("?");
    }
    url.push_str(&http_extension.querystring);

    let http_method = Method::from_str(http_extension.verb().as_str_name())?;

    let data = match &message.data {
        None => None,
        Some(data) => Some(data.value.to_owned()),
    };

    debug!("json body is: {:?}", data);

    let mut response = hyper_request(url, http_method, data, Some(message.headers.clone())).await?;

    let body_bytes = response.body_mut().collect().await?.to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes);
    debug!("response from dapr body: {}", body_str);

    if response.status() != StatusCode::OK {
        return Err(err_boxed_full_string(
            DAPR_REQUEST_FAIL,
            format!("request to {} fail with status code {}, body: {}", &config.id, &response.status(), body_str),
        ));
    }

    let content_type = match response.headers().get(header::CONTENT_TYPE) {
        None => "application/json".to_string(),
        Some(c_t) => c_t.to_str()?.to_string(),
    };

    let mut headers = HashMap::<String, String>::new();
    for (k, v) in response.headers().iter() {
        let key = k.to_string();
        let value = v.to_str()?.to_owned();
        headers.insert(key, value);
    }

    let data = InvokeResponse {
        data: Some(prost_types::Any {
            type_url: "".to_string(),
            value: body_bytes.to_vec(),
        }),
        content_type,
        headers,
    };
    dapr_execute.invoke_service = Some(data);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn get_state_grpc<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.get_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.get_state"));
        }
    };

    let response = get_dapr_client()
        .await?
        .get_state(config.store_name.clone(), config.key.clone(), Some(config.metadata.clone()))
        .await;

    debug!("get dapr state '{}' response: {:?}", config.store_name, response);

    if let Err(err) = response {
        return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
    }
    let response = response.unwrap();

    dapr_execute.get_state = Some(response);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn get_state_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.get_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.get_state"));
        }
    };

    let mut url = dapr_delete_or_get_state_url_http(config.store_name.as_str())?;
    url.push_str(config.key.as_str());
    let mut url_with_metadata = append_metadata_to_url(url, &config.metadata);
    let consistency = &config.consistency();
    let url = match consistency {
        StateConsistency::ConsistencyUnspecified => url_with_metadata,
        StateConsistency::ConsistencyStrong => {
            if config.metadata.is_empty() {
                url_with_metadata.push_str("?consistency=strong");
            } else {
                url_with_metadata.push_str("&consistency=strong");
            }
            url_with_metadata
        }
        StateConsistency::ConsistencyEventual => {
            if config.metadata.is_empty() {
                url_with_metadata.push_str("?consistency=eventual");
            } else {
                url_with_metadata.push_str("&consistency=eventual");
            }
            url_with_metadata
        }
    };

    let mut response = hyper_request(url, Method::GET, None, None).await?;

    let body_bytes = response.body_mut().collect().await?.to_bytes();
    debug!("response from dapr body: {}", String::from_utf8_lossy(&body_bytes));

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    let data = serde_json::from_slice::<GetStateResponse>(&body_bytes)?;
    dapr_execute.get_state = Some(data);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn get_bulk_state_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.get_bulk_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.get_bulk_state"));
        }
    };

    let data = serde_json::json!({
        "keys": serde_json::Value::from(config.keys.to_vec()),
        "parallelism": serde_json::Value::from(config.parallelism),
    })
    .to_string()
    .into_bytes();

    let url = dapr_get_bulk_state_url_http(config.store_name.as_str())?;
    let url = append_metadata_to_url(url, &config.metadata);

    let mut response = hyper_request(url, Method::POST, Some(data), None).await?;

    let body_bytes = response.body_mut().collect().await?.to_bytes();
    debug!("response from dapr body: {}", String::from_utf8_lossy(&body_bytes));

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    let data = serde_json::from_slice::<GetBulkStateResponse>(&body_bytes)?;
    dapr_execute.get_bulk_state = Some(data);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn query_state_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.query_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.query_state"));
        }
    };

    let url = dapr_query_state_url_http(config.store_name.as_str())?;
    let url_with_metadata = append_metadata_to_url(url, &config.metadata);

    let mut response = hyper_request(url_with_metadata, Method::POST, Some(config.query.clone().into_bytes()), None).await?;

    let body_bytes = response.body_mut().collect().await?.to_bytes();
    debug!("response from dapr body: {}", String::from_utf8_lossy(&body_bytes));

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    let data = serde_json::from_slice::<QueryStateResponse>(&body_bytes)?;

    dapr_execute.query_state = Some(data);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn save_state_grpc<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let (dapr_config, _, _) = find_dapr_execute(&cw)?;

    let config = match &dapr_config.save_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.save_state"));
        }
    };

    let data = config.states.iter().map(|item| (item.key.to_owned(), item.value.to_owned()));

    let response = get_dapr_client().await?.save_state(config.store_name.clone(), data).await;

    debug!("save dapr state '{}' response: {:?}", config.store_name, response);

    if let Err(err) = response {
        return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
    }

    Ok(cw)
}

pub async fn save_state_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let (dapr_config, _, _) = find_dapr_execute(&cw)?;

    let config = match &dapr_config.save_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.save_state"));
        }
    };

    let data = serde_json::json!(config.states).to_string().into_bytes();

    let url = dapr_save_state_url_http(config.store_name.as_str())?;

    let response = hyper_request(url, Method::POST, Some(data), None).await?;

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    Ok(cw)
}

pub async fn transaction_state_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let (dapr_config, _, _) = find_dapr_execute(&cw)?;

    let config = match &dapr_config.transaction_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.transaction_state"));
        }
    };

    let data = serde_json::json!({
        "operations": config.operations,
        "metadata": config.metadata,
    })
    .to_string()
    .into_bytes();

    let url = dapr_transaction_state_url_http(config.store_name.as_str())?;

    let response = hyper_request(url, Method::POST, Some(data), None).await?;

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    Ok(cw)
}

pub async fn delete_state_grpc<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let (dapr_config, _, _) = find_dapr_execute(&cw)?;

    let config = match &dapr_config.delete_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.delete_state"));
        }
    };

    let response = get_dapr_client()
        .await?
        .delete_state(config.store_name.clone(), config.key.clone(), Some(config.metadata.clone()))
        .await;

    debug!("delete dapr state '{}' response: {:?}", config.store_name, response);

    if let Err(err) = response {
        return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
    }

    Ok(cw)
}

pub async fn delete_state_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let (dapr_config, _, _) = find_dapr_execute(&cw)?;

    let config = match &dapr_config.delete_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.delete_state"));
        }
    };

    let mut url = dapr_delete_or_get_state_url_http(config.store_name.as_str())?;
    url.push_str(&config.key);
    let mut url_with_metadata = append_metadata_to_url(url, &config.metadata);

    let mut query_string_existed = !config.metadata.is_empty();
    let url = match &config.options {
        None => url_with_metadata,
        Some(options) => {
            match options.concurrency() {
                StateConcurrency::ConcurrencyUnspecified => {}
                StateConcurrency::ConcurrencyFirstWrite => {
                    if query_string_existed {
                        url_with_metadata.push_str("&concurrency=first-write");
                    } else {
                        query_string_existed = true;
                        url_with_metadata.push_str("?concurrency=first-write");
                    }
                }
                StateConcurrency::ConcurrencyLastWrite => {
                    if query_string_existed {
                        url_with_metadata.push_str("&concurrency=last-write");
                    } else {
                        query_string_existed = true;
                        url_with_metadata.push_str("?concurrency=last-write");
                    }
                }
            }
            match options.consistency() {
                StateConsistency::ConsistencyUnspecified => {}
                StateConsistency::ConsistencyStrong => {
                    if query_string_existed {
                        url_with_metadata.push_str("&consistency=strong");
                    } else {
                        url_with_metadata.push_str("?consistency=strong");
                    }
                }
                StateConsistency::ConsistencyEventual => {
                    if query_string_existed {
                        url_with_metadata.push_str("&consistency=eventual");
                    } else {
                        url_with_metadata.push_str("?consistency=eventual");
                    }
                }
            }
            url_with_metadata
        }
    };

    let response = hyper_request(url, Method::POST, None, None).await?;

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    Ok(cw)
}

pub async fn delete_bulk_state_grpc<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let (dapr_config, _, _) = find_dapr_execute(&cw)?;

    let config = match &dapr_config.delete_bulk_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.delete_bulk_state"));
        }
    };

    let data = config.states.iter().map(|item| (item.key.to_owned(), item.value.to_owned()));

    let response = get_dapr_client().await?.delete_bulk_state(config.store_name.clone(), data).await;

    debug!("delete dapr bulk state '{}' response: {:?}", config.store_name, response);

    if let Err(err) = response {
        return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
    }

    Ok(cw)
}

pub async fn delete_bulk_state_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let (dapr_config, _, _) = find_dapr_execute(&cw)?;

    let config = match &dapr_config.delete_bulk_state {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.delete_bulk_state"));
        }
    };

    let data = serde_json::json!(config.states).to_string().into_bytes();

    let url = dapr_delete_bulk_state_url_http(config.store_name.as_str())?;

    let response = hyper_request(url, Method::DELETE, Some(data), None).await?;

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    Ok(cw)
}

pub async fn invoke_binding_grpc_sql<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match dapr_config.invoke_binding_sql.to_owned() {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.invoke_binding_sql"));
        }
    };

    match config.operation {
        SqlOperation::QueryPage => {
            let query_sql = config.sqls.iter().filter(|item| !item.is_page).take(1).next().ok_or("query sql not exist")?;
            let page_sql = config.sqls.iter().filter(|item| item.is_page).take(1).next().ok_or("page sql not exist")?;

            let mut query_metadata = HashMap::<String, String>::new();
            let mut page_metadata = HashMap::<String, String>::new();

            query_metadata.insert("sql".to_string(), query_sql.sql.clone());
            query_metadata.insert("params".to_string(), query_sql.params.clone());

            page_metadata.insert("sql".to_string(), page_sql.sql.clone());
            page_metadata.insert("params".to_string(), page_sql.params.clone());

            let query = get_dapr_client()
                .await?
                .invoke_binding(config.name.clone(), config.data.clone(), query_metadata, config.operation.to_string());
            let page = get_dapr_client()
                .await?
                .invoke_binding(config.name.clone(), config.data.clone(), page_metadata, config.operation.to_string());
            let join_res = join(query, page).await;

            debug!("invoke dapr binding sql query response: {:?}", join_res.0);
            debug!("invoke dapr binding sql page query response: {:?}", join_res.1);

            if let Err(err) = join_res.0 {
                return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
            }
            if let Err(err) = join_res.1 {
                return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
            }

            let query_res = join_res.0?;
            let page_res = join_res.1?;

            let response = InvokeBindingSqlResponse {
                responses: [
                    SqlResponse {
                        data: query_res.data,
                        metadata: query_res.metadata,
                        is_page: query_sql.is_page,
                        output_columns: query_sql.output_columns.clone(),
                    },
                    SqlResponse {
                        data: page_res.data,
                        metadata: page_res.metadata,
                        is_page: page_sql.is_page,
                        output_columns: page_sql.output_columns.clone(),
                    },
                ]
                .to_vec(),
            };

            dapr_execute.invoke_binding_sql = Some(response);

            cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

            return Ok(cw);
        }
        _ => {
            let sql = config.sqls.iter().take(1).next().ok_or("sql not exist")?;

            let mut metadata = HashMap::<String, String>::new();

            metadata.insert("sql".to_string(), sql.sql.clone());
            metadata.insert("params".to_string(), sql.params.clone());

            let response = get_dapr_client()
                .await?
                .invoke_binding(config.name.clone(), config.data.clone(), metadata, config.operation.to_string())
                .await;

            debug!("invoke dapr binding sql response: {:#?}", response);

            if let Err(err) = response {
                return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
            }

            let sql_res = response?;

            let response = InvokeBindingSqlResponse {
                responses: [SqlResponse {
                    data: sql_res.data,
                    metadata: sql_res.metadata,
                    is_page: sql.is_page,
                    output_columns: sql.output_columns.clone(),
                }]
                .to_vec(),
            };

            dapr_execute.invoke_binding_sql = Some(response);

            cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

            return Ok(cw);
        }
    }
}

pub async fn invoke_binding_grpc<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match dapr_config.invoke_binding.to_owned() {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.invoke_binding"));
        }
    };

    let response = get_dapr_client()
        .await?
        .invoke_binding(config.name.clone(), config.data.clone(), config.metadata.clone(), config.operation.clone())
        .await;

    debug!("invoke dapr binding response: {:?}", response);

    if let Err(err) = response {
        return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
    }
    let response = response.unwrap();

    dapr_execute.invoke_binding = Some(response);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn invoke_binding_http_sql<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match dapr_config.invoke_binding_sql.to_owned() {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.invoke_binding_sql"));
        }
    };

    let url = dapr_invoke_binding_url_http(config.name.as_str())?;

    match config.operation {
        SqlOperation::QueryPage => {
            let query_sql = config.sqls.iter().filter(|item| !item.is_page).take(1).next().ok_or("query sql not exist")?;
            let page_sql = config.sqls.iter().filter(|item| item.is_page).take(1).next().ok_or("page sql not exist")?;

            let mut query_metadata = HashMap::<String, String>::new();
            let mut page_metadata = HashMap::<String, String>::new();

            query_metadata.insert("sql".to_string(), query_sql.sql.clone());
            query_metadata.insert("params".to_string(), query_sql.params.clone());

            page_metadata.insert("sql".to_string(), page_sql.sql.clone());
            page_metadata.insert("params".to_string(), page_sql.params.clone());

            let query_data = serde_json::json!({
                "data": config.data,
                "metadata": query_metadata,
                "operation": config.operation.to_string(),
            })
            .to_string()
            .into_bytes();
            let page_data = serde_json::json!({
                "data": config.data,
                "metadata": page_metadata,
                "operation": config.operation.to_string(),
            })
            .to_string()
            .into_bytes();

            let query = hyper_request(url.clone(), Method::POST, Some(query_data), None);
            let page = hyper_request(url.clone(), Method::POST, Some(page_data), None);

            let join_res = join(query, page).await;

            debug!("invoke dapr binding sql query response: {:?}", join_res.0);
            debug!("invoke dapr binding sql page query response: {:?}", join_res.1);

            if let Err(err) = join_res.0 {
                return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
            }
            if let Err(err) = join_res.1 {
                return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
            }

            let mut query_res_bytes = join_res.0?.body_mut().collect().await?.to_bytes();
            let mut page_res_bytes = join_res.1?.body_mut().collect().await?.to_bytes();

            let query_data = serde_json::from_slice::<InvokeBindingResponse>(&query_res_bytes)?;

            debug!("response from dapr body: {}", String::from_utf8_lossy(&page_res_bytes));
            let page_data = serde_json::from_slice::<InvokeBindingResponse>(&page_res_bytes)?;

            let response = InvokeBindingSqlResponse {
                responses: [
                    SqlResponse {
                        data: query_data.data,
                        metadata: query_data.metadata,
                        is_page: query_sql.is_page,
                        output_columns: query_sql.output_columns.clone(),
                    },
                    SqlResponse {
                        data: page_data.data,
                        metadata: page_data.metadata,
                        is_page: page_sql.is_page,
                        output_columns: page_sql.output_columns.clone(),
                    },
                ]
                .to_vec(),
            };

            dapr_execute.invoke_binding_sql = Some(response);

            cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

            return Ok(cw);
        }
        _ => {
            let sql = config.sqls.iter().take(1).next().ok_or("sql not exist")?;

            let mut metadata = HashMap::<String, String>::new();

            metadata.insert("sql".to_string(), sql.sql.clone());
            metadata.insert("params".to_string(), sql.params.clone());

            let data = serde_json::json!({
                "data": config.data,
                "metadata": metadata,
                "operation": config.operation.to_string(),
            })
            .to_string()
            .into_bytes();

            let response = hyper_request(url, Method::POST, Some(data), None).await;

            debug!("invoke dapr binding sql response: {:#?}", response);

            if let Err(err) = response {
                return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
            }

            let mut sql_res = response?;

            let body_bytes = sql_res.body_mut().collect().await?.to_bytes();
            debug!("response from dapr body: {}", String::from_utf8_lossy(&body_bytes));
            let res_data = serde_json::from_slice::<InvokeBindingResponse>(&body_bytes)?;

            let response = InvokeBindingSqlResponse {
                responses: [SqlResponse {
                    data: res_data.data,
                    metadata: res_data.metadata,
                    is_page: sql.is_page,
                    output_columns: sql.output_columns.clone(),
                }]
                .to_vec(),
            };

            dapr_execute.invoke_binding_sql = Some(response);

            cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

            return Ok(cw);
        }
    }
}

pub async fn invoke_binding_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.invoke_binding {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.invoke_binding"));
        }
    };

    let data = serde_json::json!({
        "data": config.data,
        "metadata": config.metadata,
        "operation": config.operation,
    })
    .to_string()
    .into_bytes();

    let url = dapr_invoke_binding_url_http(config.name.as_str())?;

    let mut response = hyper_request(url, Method::POST, Some(data), None).await?;

    let body_bytes = response.body_mut().collect().await?.to_bytes();
    debug!("response from dapr body: {}", String::from_utf8_lossy(&body_bytes));

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    let data = serde_json::from_slice::<InvokeBindingResponse>(&body_bytes)?;

    dapr_execute.invoke_binding = Some(data);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn publish_event_grpc<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let (dapr_config, _, _) = find_dapr_execute(&cw)?;

    let config = match &dapr_config.publish_event {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.publish_event"));
        }
    };

    let response = get_dapr_client()
        .await?
        .publish_event(
            config.pubsub_name.clone(),
            config.topic.clone(),
            config.data_content_type.clone(),
            config.data.clone(),
            Some(config.metadata.clone()),
        )
        .await;

    debug!("publish dapr event '{}.{}' response: {:?}", config.pubsub_name, config.topic, response);

    if let Err(err) = response {
        return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
    }

    Ok(cw)
}

pub async fn publish_event_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let (dapr_config, _, _) = find_dapr_execute(&cw)?;

    let config = match &dapr_config.publish_event {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.publish_event"));
        }
    };

    let url = dapr_publish_event_url_http(&config.pubsub_name, &config.topic)?;
    let url_with_metadata = append_metadata_to_url(url, &config.metadata);

    let response = hyper_request(url_with_metadata, Method::POST, Some(config.data.clone()), None).await?;

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    Ok(cw)
}

pub async fn publish_bulk_event_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.publish_bulk_event {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.publish_bulk_event"));
        }
    };

    let data = serde_json::json!(config.entries).to_string().into_bytes();

    let url = dapr_publish_bulk_url_http(&config.pubsub_name, &config.topic)?;
    let url_with_metadata = append_metadata_to_url(url, &config.metadata);

    let mut response = hyper_request(url_with_metadata, Method::POST, Some(data), None).await?;

    if response.status() == StatusCode::INTERNAL_SERVER_ERROR {
        let body_bytes = response.body_mut().collect().await?.to_bytes();
        debug!("response from dapr body: {}", String::from_utf8_lossy(&body_bytes));
        let data = serde_json::from_slice::<BulkPublishResponse>(&body_bytes)?;

        dapr_execute.publish_bulk_event = Some(data);

        cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

        return Ok(cw);
    }

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    Ok(cw)
}

pub async fn get_secret_grpc<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.get_secret {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.get_secret"));
        }
    };

    let response = get_dapr_client().await?.get_secret(config.store_name.clone(), config.key.clone()).await;

    debug!("get dapr secret response: {:?}", response);

    if let Err(err) = response {
        return Err(err_boxed_full_string(DAPR_REQUEST_FAIL, err.to_string()));
    }
    let response = response.unwrap();

    dapr_execute.get_secret = Some(response);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn get_secret_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.get_secret {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.get_secret"));
        }
    };

    let mut url = dapr_get_secret_url_http(&config.store_name)?;
    url.push_str(&config.key);
    let url_with_metadata = append_metadata_to_url(url, &config.metadata);

    let mut response = hyper_request(url_with_metadata, Method::GET, None, None).await?;

    let body_bytes = response.body_mut().collect().await?.to_bytes();
    debug!("response from dapr body: {}", String::from_utf8_lossy(&body_bytes));

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    let data = serde_json::from_slice::<GetSecretResponse>(&body_bytes)?;
    dapr_execute.get_secret = Some(data);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn get_bulk_secret_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.get_bluk_secret {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.get_bluk_secret"));
        }
    };

    let url = dapr_get_bulk_secret_url_http(&config.store_name)?;

    let mut response = hyper_request(url, Method::GET, None, None).await?;

    let body_bytes = response.body_mut().collect().await?.to_bytes();
    debug!("response from dapr body: {}", String::from_utf8_lossy(&body_bytes));

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    let data = serde_json::from_slice::<GetBulkSecretResponse>(&body_bytes)?;
    dapr_execute.get_bluk_secret = Some(data);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}

pub async fn get_configuration_http<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C>(
    mut cw: ContextWrapper<I, O, C>,
) -> HttpResult<ContextWrapper<I, O, C>> {
    let dapr_execute_name = cw.exec_name.clone().ok_or(err_boxed(EXEC_NAME_NOT_EXIST))?;
    let (dapr_config, dapr_execute, _) = find_dapr_execute(&cw)?;
    let mut dapr_execute = dapr_execute.clone();

    let config = match &dapr_config.get_configuration {
        Some(config) => config,
        None => {
            return Err(err_boxed_full(DAPR_CONFIG_NOT_EXIST, "dapr_config.get_configuration"));
        }
    };

    let mut url = dapr_get_configuration_url_http(&config.store_name)?;

    if config.keys.len() > 0 {
        url.push_str("?key=");

        let keys_string = config.keys.join("&key=");
        url.push_str(&keys_string);
    }

    let mut response = hyper_request(url, Method::GET, None, None).await?;

    let body_bytes = response.body_mut().collect().await?.to_bytes();
    debug!("response from dapr body: {}", String::from_utf8_lossy(&body_bytes));

    if &response.status() != &StatusCode::OK {
        return Err(err_boxed(DAPR_REQUEST_FAIL));
    }

    let data = serde_json::from_slice::<GetConfigurationResponse>(&body_bytes)?;
    dapr_execute.get_configuration = Some(data);

    cw.exec.insert(dapr_execute_name, (dapr_config.clone(), dapr_execute, None));

    Ok(cw)
}
