#[macro_export]
macro_rules! biz_result {
    (
        $acceptor:ident,
        $(
            ($konst:ident, $status_code:expr, $biz_code:expr, $message:expr)$(;)?
        )*
    ) => {
        $(
            pub const $konst: crate::util::BizResult<'static> = crate::util::BizResult($status_code, $biz_code, $message, stringify!($konst));
        )*

        impl $acceptor {
            async fn insert_biz_result() -> HttpResult<()> {
                $(
                    util::insert_biz_result($konst).await?;
                )*
                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! income_param {
    (
        $acceptor:ident,
        $(
            ($konst:ident,[$(($target:ident,$name:expr,$from:ident,$type:ident,$require:expr)$(,)?)*]);
        )*
    ) => {
        impl $acceptor {
            async fn insert_income_param() -> HttpResult<()> {
                $(
                    let _ = crate::util::insert_income_param($konst, vec![$((String::from(stringify!($target)),String::from(stringify!($name)),crate::model::ParamFrom::$from,crate::model::ParamType::$type,$require),)*]).await;
                )*
                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! internal_auth_tag {
    ($tag:expr) => {
        let _ = crate::util::set_internal_auth_tag($tag).await;
    };
}

#[macro_export]
macro_rules! skip_auth_uri {
    ($($target:ident$(,)?)*) => {
        $(
            let _ = crate::util::set_skip_auth_uri($target).await;
        )*
    };
}

#[macro_export]
macro_rules! uri {
    (
        $acceptor:ident,
        $(
            ($konst:ident, $method:ident, $path:expr, $action:ident, $bulk_input:expr, $bulk_output:expr);
        )*
    ) => {
        $(
            pub const $konst: URI = URI(hyper::Method::$method, $path, stringify!($konst), crate::model::Action::$action, $bulk_input, $bulk_output);
        )*

        impl $acceptor {
            async fn insert_uri() -> HttpResult<()> {
                $(
                    util::insert_uri($konst).await?;
                )*
                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! generate_http_dispatcher {
    ($acceptor:ident,[$(($uri_name:ident, $fn_name:ident)$(,)?)*]) => {
        impl HttpRequestDispatcherTrait for $acceptor {
            async fn do_http_dispatch(params: Params) -> HttpResult<hyper::Response<body::Body>> {
                match params.uri.as_str() {
                    $(
                        stringify!($uri_name) => handle_http($fn_name(&params).await, &params).await,
                    )*

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
        }
    };
}

#[macro_export]
macro_rules! generate_grpc_dispatcher {
    ($acceptor:ident,[$(($uri_name:ident, $fn_name:ident)$(,)?)*]) => {
        impl GrpcRequestDispatcherTrait for $acceptor {
            async fn do_grpc_dispatch(params: Params) -> GrpcResult<tonic::Response<InvokeResponse>> {
                match params.uri.as_str() {
                    $(
                        stringify!($uri_name) => handle_grpc($fn_name(&params).await, &params).await,
                    )*

                    _ => {
                        eprintln!("[request begin] error: uri match nothing");
                        return GrpcResult::Err(tonic::Status::internal(URI_NOT_MATCH.message()));
                    }
                }
            }
        }
    };
}
