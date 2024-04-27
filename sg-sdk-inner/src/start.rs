use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use dapr::{
    appcallback::{
        BindingEventRequest, BindingEventResponse, InvokeRequest, InvokeResponse, ListInputBindingsResponse, ListTopicSubscriptionsResponse, TopicEventRequest,
        TopicEventResponse,
    },
    dapr::dapr::proto::runtime::v1::app_callback_server::{AppCallback, AppCallbackServer},
};
use http_body_util::Either;
use hyper::{body::Incoming, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
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
    inner_biz_result::*,
    model::{IfRes, Params},
    util::{self, auth_ict, find_response_auth_header, parse_params_grpc},
    GrpcResult, HttpResult, *,
};

use self::traits::{DaprBody, GrpcRequestDispatcherTrait, HttpRequestDispatcherTrait, ModelTrait};

pub async fn start_http<HttpDispatcher: HttpRequestDispatcherTrait + Send + Copy + 'static>(port: u16) -> HttpResult<()> {
    let http_addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Listening on http port {}", http_addr);

    let http_serve = async move {
        let listener = TcpListener::bind(http_addr).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(TokioIo::new(stream), service_fn(move |req| http_service::<HttpDispatcher>(req)))
                    .await
                {
                    error!("Error serving connection: {:?}", err);
                }
            });
        }
    };

    http_serve.await
}

pub async fn start_grpc<GrpcDispatcher: GrpcRequestDispatcherTrait + Send + Copy + Sync + 'static>(port: u16) -> HttpResult<()> {
    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Listening on grpc port {}", grpc_addr);

    let grpc_serve = async move {
        let callback_service = GrpcService::<GrpcDispatcher> { _placeholder: None };

        Server::builder()
            .add_service(AppCallbackServer::new(callback_service))
            .serve(grpc_addr)
            .await
            .unwrap();
        Ok(())
    };

    grpc_serve.await
}

pub async fn start_http_grpc<OneDispatcher: HttpRequestDispatcherTrait + GrpcRequestDispatcherTrait + Send + Copy + Sync + 'static>(
    http_port: u16,
    grpc_port: u16,
) -> HttpResult<()> {
    start_http::<OneDispatcher>(http_port).await?;
    start_grpc::<OneDispatcher>(grpc_port).await?;
    Ok(())
}

async fn http_service<HttpDispatcher: HttpRequestDispatcherTrait + Send + Copy + 'static>(
    req: Request<Incoming>,
) -> HttpResult<Response<Either<body::Body, body::BodySt>>> {
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

    HttpDispatcher::do_http_dispatch(params).await
}

pub struct GrpcService<GrpcDispatcher: GrpcRequestDispatcherTrait + Send + Copy + 'static> {
    _placeholder: Option<GrpcDispatcher>,
}

#[tonic::async_trait]
impl<GrpcDispatcher: GrpcRequestDispatcherTrait + Send + Copy + Sync> AppCallback for GrpcService<GrpcDispatcher> {
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

        GrpcDispatcher::do_grpc_dispatch(params).await
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

pub async fn handle_http<T: Serialize + prost::Message + ModelTrait + Default + DaprBody>(
    http_res: HttpResult<(IfRes<T>, HashMap<String, String>)>,
    params: &Params,
) -> HttpResult<Response<Either<body::Body, body::BodySt>>> {
    match http_res {
        Ok((if_res, response_header)) => Ok(util::gen_resp_ok(OK, if_res, response_header, &params).await),
        Err(err) => Ok(util::err_resolve(err).await),
    }
}

pub async fn handle_grpc<T: prost::Message + ModelTrait + Default + Serialize>(
    http_res: HttpResult<(IfRes<T>, HashMap<String, String>)>,
    params: &Params,
) -> GrpcResult<tonic::Response<InvokeResponse>> {
    match http_res {
        Ok(if_res) => {
            let mut response = tonic::Response::new(InvokeResponse {
                content_type: "application/grpc".to_string(),
                data: Some(prost_types::Any {
                    type_url: "".to_string(),
                    value: if_res.0.to_message().encode_to_vec(),
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
