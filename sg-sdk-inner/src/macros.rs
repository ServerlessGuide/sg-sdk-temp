#[macro_export]
macro_rules! biz_result {
    (
        $(
            ($konst:ident, $status_code:expr, $biz_code:expr, $message:expr);
        )*
    ) => {
        $(
            pub const $konst: crate::util::BizResult<'static> = crate::util::BizResult($status_code, $biz_code, $message, stringify!($konst));
        )*
    }
}

#[macro_export]
macro_rules! register_biz_result {
    ($($konst:ident$(,)?)*) => {
        $(
            util::insert_biz_result($konst).await?;
        )*
    }
}

#[macro_export]
macro_rules! income_param {
    (
        $(
            ($konst:ident,[$(($target:ident,$name:expr,$from:ident,$type:ident,$require:expr)$(,)?)*]);
        )*
    ) => {
        $(
            let _ = crate::util::insert_income_param($konst, vec![$((String::from(stringify!($target)),String::from(stringify!($name)),crate::model::ParamFrom::$from,crate::model::ParamType::$type,$require),)*]).await;
        )*
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
        $(
            ($konst:ident, $method:ident, $path:expr, $action:ident, $bulk_input:expr, $bulk_output:expr);
        )*
    ) => {
        $(
            pub const $konst: URI = URI(hyper::Method::$method, $path, stringify!($konst), crate::model::Action::$action, $bulk_input, $bulk_output);
        )*
    }
}

#[macro_export]
macro_rules! register_uri {
    ($($konst:ident$(,)?)*) => {
        $(
            util::insert_uri($konst).await?;
        )*
    }
}
