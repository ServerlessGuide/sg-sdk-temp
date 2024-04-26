use crate::biz_result;
use crate::HttpResult;

biz_result! {
    InnerConfigForSelfUse,
    (OK, 200, 999900, "success");
    (URI_NOT_MATCH, 404, 999901, "uri match nothing");
    (BODY_PARAMETER_ILLEGAL, 400, 999902, "body parameter illegal");
    (CONVERT_TO_MODEL_ERROR, 500, 999903, "convert to model error");
    (PARAMETER_ILLEGAL, 400, 999904, "parameter illegal");
    (HEADER_NOT_FOUND, 400, 999905, "header not found");
    (PARAM_MAP_PARSE_ERROR, 500, 999906, "param map parse error");
    (PATH_PARAM_NOT_EXIST, 500, 999907, "path param not exist");
    (BODY_PARAM_NOT_EXIST, 500, 999908, "body param not exist");
    (QUERY_PARAM_NOT_EXIST, 500, 999909, "query param not exist");
    (URL_PARSE_ERROR, 500, 999910, "url parse error");
    (DAPR_HTTP_REQ_BUILD_ERROR, 500, 999911, "dapr request build error");
    (DAPR_REQUEST_FAIL, 500, 999912, "dapr request fail");
    (REQUEST_METHOD_NOT_ALLOWED, 500, 999913, "request method not allowed");
    (ENV_PARAMETER_ERROR, 500, 999914, "env parameter error");
    (DAPR_DATA_ILLEGAL, 500, 999915, "dapr data illegal");
    (ENUM_NOT_FOUND, 500, 999916, "enum not found");
    (IMPLICIT_RESPONSE_ERROR, 500, 999917, "implicit response error");
    (BIZ_RESULT_NOT_FOUND, 500, 999918, "biz result not found");
    (DAPR_COMPONENT_NOT_EXIST, 500, 999919, "dapr config not exist");
    (EXEC_NAME_NOT_EXIST, 500, 999920, "execute name not exist");
    (DAPR_EXECUTE_NOT_EXIST, 500, 999921, "dapr execute not exist");
    (QUERY_SQL_IS_NOT_UNIQUE, 500, 999922, "query sql is not unique");
    (SQL_NOT_VALID, 500, 999923, "sql not valid");
    (SQL_NOT_SUPPORT, 500, 999924, "sql not support");
    (DATA_NOT_FOUND, 400, 999925, "data not found");
    (SQL_OUT_COLUMNS_IS_EMPTY, 500, 999926, "sql out_columns is empty");
    (DATA_ERROR, 500, 999927, "data error");
    (AUTH_ERROR, 401, 999928, "auth error");
    (INTERNAL_AUTH_TAG_NOT_SET, 500, 999929, "internal auth tag not set");
}

struct InnerConfigForSelfUse();
