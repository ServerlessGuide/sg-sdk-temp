use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use dapr::{
    appcallback::{
        BindingEventRequest, BindingEventResponse, InvokeRequest, InvokeResponse, ListInputBindingsResponse, ListTopicSubscriptionsResponse, TopicEventRequest,
        TopicEventResponse,
    },
    dapr::dapr::proto::runtime::v1::app_callback_server::{AppCallback, AppCallbackServer},
};
use hyper::{body::Incoming, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use model_macro::ModelTrait;
use prost::Message;
use serde::Serialize;
use tokio::net::TcpListener;
use tonic::{
    metadata::{MetadataKey, MetadataValue},
    transport::Server,
    Status,
};
use tracing::{error, info};

use crate::{
    body,
    model::{IfRes, Params, Res},
    util::{self, auth_ict, find_response_auth_header, parse_params_grpc},
    GrpcResult, HttpResult, *,
};

pub async fn start_http(port: u16) -> HttpResult<()> {
    let http_addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Listening on http port {}", http_addr);

    let http_serve = async move {
        let listener = TcpListener::bind(http_addr).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(TokioIo::new(stream), service_fn(move |req| http_service(req)))
                    .await
                {
                    error!("Error serving connection: {:?}", err);
                }
            });
        }
    };

    http_serve.await
}

pub async fn start_grpc(port: u16) -> HttpResult<()> {
    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Listening on grpc port {}", grpc_addr);

    let grpc_serve = async move {
        let callback_service = GrpcService {};

        Server::builder()
            .add_service(AppCallbackServer::new(callback_service))
            .serve(grpc_addr)
            .await
            .unwrap();
        Ok(())
    };

    grpc_serve.await
}

pub async fn start_http_grpc(http_port: u16, grpc_port: u16) -> HttpResult<()> {
    start_http(http_port).await?;
    start_grpc(grpc_port).await?;
    Ok(())
}

async fn http_service(req: Request<Incoming>) -> HttpResult<Response<body::Body>> {
    let params = util::parse_params(req).await;
    let mut params = match params {
        Ok(params) => params,
        Err(err) => {
            return Ok(util::err_resolve(err).await);
        }
    };

    match auth_ict(&mut params).await {
        Ok(_) => {}
        Err(err) => {
            return Ok(util::err_resolve(err).await);
        }
    };

    match params.uri.as_str() {
        // "QUERY_ALL_SMS" => handle_http(query_all_sms(&params).await, &params),
        // "QUERY_ONE_BY_ID" => handle_http(query_one_by_id(&params).await, &params),
        _ => {
            eprintln!("[request begin] error: uri match nothing");
            Ok(util::gen_resp(
                URI_NOT_MATCH.status_code(),
                Res::<String> {
                    code: URI_NOT_MATCH.biz_code(),
                    message: URI_NOT_MATCH.message(),
                    result: None,
                },
            ))
        }
    }
}

pub struct GrpcService {}

#[tonic::async_trait]
impl AppCallback for GrpcService {
    async fn on_invoke(&self, request: tonic::Request<InvokeRequest>) -> GrpcResult<tonic::Response<InvokeResponse>> {
        println!("grpc request: {:?}", &request);

        let params = parse_params_grpc(request).await;
        let mut params = match params {
            Ok(params) => params,
            Err(err) => {
                return GrpcResult::Err(err);
            }
        };

        match auth_ict(&mut params).await {
            Ok(_) => {}
            Err(err) => {
                return GrpcResult::Err(Status::unauthenticated(err.to_string()));
            }
        };

        match params.uri.as_str() {
            // "QUERY_ALL_SMS" => handle_grpc(query_all_sms(&params).await, &params),
            // "QUERY_ONE_BY_ID" => handle_grpc(query_one_by_id(&params).await, &params),
            _ => {
                eprintln!("request error: uri match nothing");
                return GrpcResult::Err(Status::internal(URI_NOT_MATCH.message()));
            }
        }
    }

    async fn list_topic_subscriptions(&self, _request: tonic::Request<()>) -> GrpcResult<tonic::Response<ListTopicSubscriptionsResponse>> {
        let list_subscriptions = ListTopicSubscriptionsResponse::default();
        Ok(tonic::Response::new(list_subscriptions))
    }

    async fn on_topic_event(&self, _request: tonic::Request<TopicEventRequest>) -> GrpcResult<tonic::Response<TopicEventResponse>> {
        Ok(tonic::Response::new(TopicEventResponse::default()))
    }

    async fn list_input_bindings(&self, _request: tonic::Request<()>) -> GrpcResult<tonic::Response<ListInputBindingsResponse>> {
        Ok(tonic::Response::new(ListInputBindingsResponse::default()))
    }

    async fn on_binding_event(&self, _request: tonic::Request<BindingEventRequest>) -> GrpcResult<tonic::Response<BindingEventResponse>> {
        Ok(tonic::Response::new(BindingEventResponse::default()))
    }
}

async fn handle_http<T: Serialize + prost::Message + ModelTrait + Default>(
    http_res: HttpResult<IfRes<T>>,
    params: &Params,
) -> HttpResult<Response<body::Body>> {
    match http_res {
        Ok(if_res) => Ok(util::gen_resp_ok(OK, if_res, &params).await),
        Err(err) => Ok(util::err_resolve(err).await),
    }
}

async fn handle_grpc<T: prost::Message + ModelTrait + Default>(http_res: HttpResult<IfRes<T>>, params: &Params) -> GrpcResult<tonic::Response<InvokeResponse>> {
    match http_res {
        Ok(if_res) => {
            let mut response = tonic::Response::new(InvokeResponse {
                content_type: "application/grpc".to_string(),
                data: Some(prost_types::Any {
                    type_url: "".to_string(),
                    value: if_res.to_message().encode_to_vec(),
                }),
                headers: HashMap::<String, String>::new(),
            });
            let token_pair = find_response_auth_header(params).await.unwrap();
            match token_pair.0 {
                None => {}
                Some(key) => match token_pair.1 {
                    None => {}
                    Some(value) => {
                        response
                            .metadata_mut()
                            .insert(MetadataKey::from_str(key.as_str()).unwrap(), MetadataValue::try_from(value.as_str()).unwrap());
                    }
                },
            }
            GrpcResult::Ok(response)
        }
        Err(err) => GrpcResult::Err(Status::internal(err.to_string())),
    }
}
