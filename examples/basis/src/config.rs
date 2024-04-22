use tracing::{debug, error, info, trace, warn};

use crate::biz_result;
use crate::daprs;
use crate::model::*;
use crate::uri;
use crate::util::*;

use std::collections::*;

lazy_static! {
    pub static ref SERVICE_ID: i64 = 6999453093112840201;
    pub static ref BIZ_RESULT_PREFIX: i16 = 1054;
    pub static ref SKIP_AUTH_IFS: Vec<String> = vec![];
    pub static ref INTERNAL_AUTH_TAG: Option<String> = Some(String::from("Serverless-Guide"));
    pub static ref INCOME_PARAM_MAP: HashMap<String, ExtraParamMap> = {
        let mut params = HashMap::<String, ExtraParamMap>::new();

        let mut interface_params = HashMap::<String, IncomeParamDef>::new();
        interface_params.insert(
            String::from("db_database_id"),
            IncomeParamDef {
                name: String::from("3"),
                from: ParamFrom::Path,
                param_type: ParamType::Number,
                required: true,
            },
        );
        params.insert(URI::QUERY_ALL_SMS.name().to_string(), ExtraParamMap { params: interface_params });

        let mut interface_params = HashMap::<String, IncomeParamDef>::new();
        interface_params.insert(
            String::from("id"),
            IncomeParamDef {
                name: String::from("2"),
                from: ParamFrom::Path,
                param_type: ParamType::Number,
                required: true,
            },
        );
        params.insert(URI::QUERY_ONE_BY_ID.name().to_string(), ExtraParamMap { params: interface_params });

        params
    };
}

uri! {
    (QUERY_ALL_SMS, hyper::Method::GET, "^/db-sm/sms/\\d{19}$", "QUERY_ALL_SMS", Action::Query, false, true);
    (QUERY_ONE_BY_ID, hyper::Method::GET, "^/db-sm/\\d{19}$", "QUERY_ONE_BY_ID", Action::Query, false, false);
}

biz_result! {
    (OK, 200, 105300, "success", "OK");
    (URI_NOT_MATCH, 404, 105301, "uri match nothing", "URI_NOT_MATCH");
    (BODY_PARAMETER_ILLEGAL, 400, 105302, "body parameter illegal", "BODY_PARAMETER_ILLEGAL");
    (CONVERT_TO_MODEL_ERROR, 500, 105303, "convert to model error", "CONVERT_TO_MODEL_ERROR");
    (PARAMETER_ILLEGAL, 400, 105304, "parameter illegal", "PARAMETER_ILLEGAL");
    (HEADER_NOT_FOUND, 400, 105305, "header not found", "HEADER_NOT_FOUND");
    (PARAM_MAP_PARSE_ERROR, 500, 105306, "param map parse error", "PARAM_MAP_PARSE_ERROR");
    (PATH_PARAM_NOT_EXIST, 500, 105307, "path param not exist", "PATH_PARAM_NOT_EXIST");
    (BODY_PARAM_NOT_EXIST, 500, 105308, "body param not exist", "BODY_PARAM_NOT_EXIST");
    (QUERY_PARAM_NOT_EXIST, 500, 105309, "query param not exist", "QUERY_PARAM_NOT_EXIST");
    (URL_PARSE_ERROR, 500, 105310, "url parse error", "URL_PARSE_ERROR");
    (DAPR_HTTP_REQ_BUILD_ERROR, 500, 105311, "dapr request build error", "DAPR_HTTP_REQ_BUILD_ERROR");
    (DAPR_REQUEST_FAIL, 500, 105312, "dapr request fail", "DAPR_REQUEST_FAIL");
    (REQUEST_METHOD_NOT_ALLOWED, 500, 105313, "request method not allowed", "REQUEST_METHOD_NOT_ALLOWED");
    (ENV_PARAMETER_ERROR, 500, 105314, "env parameter error", "ENV_PARAMETER_ERROR");
    (DAPR_DATA_ILLEGAL, 500, 105315, "dapr data illegal", "DAPR_DATA_ILLEGAL");
    (ENUM_NOT_FOUND, 500, 105316, "enum not found", "ENUM_NOT_FOUND");
    (IMPLICIT_RESPONSE_ERROR, 500, 105317, "implicit response error", "IMPLICIT_RESPONSE_ERROR");
    (BIZ_RESULT_NOT_FOUND, 500, 105318, "biz result not found", "BIZ_RESULT_NOT_FOUND");
    (DAPR_CONFIG_NOT_EXIST, 500, 105319, "dapr config not exist", "DAPR_CONFIG_NOT_EXIST");
    (EXEC_NAME_NOT_EXIST, 500, 105320, "execute name not exist", "EXEC_NAME_NOT_EXIST");
    (DAPR_EXECUTE_NOT_EXIST, 500, 105321, "dapr execute not exist", "DAPR_EXECUTE_NOT_EXIST");
    (QUERY_SQL_IS_NOT_UNIQUE, 500, 105322, "query sql is not unique", "QUERY_SQL_IS_NOT_UNIQUE");
    (SQL_NOT_VALID, 500, 105323, "sql not valid", "SQL_NOT_VALID");
    (SQL_NOT_SUPPORT, 500, 105324, "sql not support", "SQL_NOT_SUPPORT");
    (DATA_NOT_FOUND, 400, 105325, "data not found", "DATA_NOT_FOUND");
    (SQL_OUT_COLUMNS_IS_EMPTY, 500, 105326, "sql out_columns is empty", "SQL_OUT_COLUMNS_IS_EMPTY");
    (DATA_ERROR, 500, 105327, "data error", "DATA_ERROR");
    (AUTH_ERROR, 401, 105328, "auth error", "AUTH_ERROR");
    (INTERNAL_AUTH_TAG_NOT_SET, 500, 105329, "internal auth tag not set", "INTERNAL_AUTH_TAG_NOT_SET");
}
