use crate::*;

lazy_static! {}

pub fn prepare_inner_context_for_query_by_app_id(
    mut context: ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>,
) -> HttpResult<ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>> {
    let jwt_token_val = if let Some(v) = context.header.get(AuthHeader::XSGAuthJWT.lower_case_value()) {
        v.to_string()
    } else if let Some(v) = context.header.get(AuthHeader::XSGAuthJWT.lower_case_value()) {
        v.to_string()
    } else {
        return Err(err_boxed_full(DATA_ERROR, "jwt header not found"));
    };

    let claim_vec = jwt_token_val.split(".").collect::<Vec<&str>>();
    let claim_str = claim_vec.get(1).ok_or("jwt token format error")?;

    use base64::engine::general_purpose::*;
    use base64::Engine;

    let decoded = STANDARD_NO_PAD.decode(claim_str)?;
    let claims = serde_json::from_slice::<UserWithIdSid>(&decoded)?;
    let sid = claims.sid.ok_or("jwt token claim not correct")?;
    let id = claims.id.ok_or("jwt token claim not correct")?;

    context.inner_context.id = Some(id);
    context.inner_context.sid = Some(sid);

    Ok(context)
}

pub fn pre_check_permission(
    context: ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>,
) -> HttpResult<ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>> {
    let id = context.inner_context.id.clone().ok_or("user id not found")?;
    let app_id = context.input.app_id.ok_or("app id not found")?;

    let mut dapr_comp = None;

    let sql = r#"
select
    r.id as rel_id
from
    public.rel_user_app_role r, public.role as l
where
    l.id = r.role_id
    and r.user_id = ?
    and r.app_id = ?;
"#;

    let context = context
        .dapr_invoke_binding_sql("query_rel_exist", "sg-base-role")?
        .dapr_invoke_binding_sql_operation(SqlOperation::Query)?
        .get_current_dapr_component(|d| dapr_comp = d)?
        .dapr_invoke_binding_sql_sqls(
            SqlsBuilder::new()
                .dapr_component(dapr_comp.as_ref().ok_or("dapr component not found")?)
                .operation(SqlOperation::Query)
                .sql_builder(
                    SqlBuilder::new()
                        .sql(sql)
                        .param_extend(rbs::Value::I64(id.to_owned().parse()?))
                        .param_extend(rbs::Value::I64(app_id.to_owned())),
                )
                .build()?,
        )?;

    Ok(context)
}

pub fn post_check_permission(
    context: ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>,
) -> HttpResult<ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>> {
    let (context, _) = context
        .decode_sql_one::<RelId>("query_rel_exist")
        .map_err(|_| err_boxed_full(AUTH_ERROR, "you don't have permission to access the app"))?
        .get_dapr_resp_one::<RelId>("query_rel_exist")
        .map_err(|_| err_boxed_full(AUTH_ERROR, "you don't have permission to access the app"))?;

    Ok(context)
}

pub fn pre_query_by_app_id(
    context: ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>,
) -> HttpResult<ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>> {
    let app_id = context.input.app_id.ok_or("app_id not exist")?;

    let mut dapr_comp = None;

    let context = context
        .dapr_invoke_binding_sql("query_by_app_id", "sg-base-role")?
        .get_current_dapr_component(|d| dapr_comp = d)?
        .dapr_invoke_binding_sql_operation(SqlOperation::Query)?
        .dapr_invoke_binding_sql_sqls(trans_sql_info(
            vec![AppVersion::select_by_column("app_id", app_id)?],
            SqlOperation::Query,
            dapr_comp.as_ref().ok_or("dapr component not found")?,
        )?)?;

    Ok(context)
}

pub fn post_query_by_app_id(
    context: ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>,
) -> HttpResult<ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>> {
    let (mut context, list) = context
        .decode_sql_list::<AppVersion>("query_by_app_id")?
        .get_dapr_resp_list::<AppVersion>("query_by_app_id")?;

    context.outputs = list;

    Ok(context)
}

pub fn prepare_inner_context_for_insert(
    mut context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let jwt_token_val = if let Some(v) = context.header.get(AuthHeader::XSGAuthJWT.lower_case_value()) {
        v.to_string()
    } else if let Some(v) = context.header.get(AuthHeader::XSGAuthJWT.lower_case_value()) {
        v.to_string()
    } else {
        return Err(err_boxed_full(DATA_ERROR, "jwt header not found"));
    };

    let claim_vec = jwt_token_val.split(".").collect::<Vec<&str>>();
    let claim_str = claim_vec.get(1).ok_or("jwt token format error")?;

    use base64::engine::general_purpose::*;
    use base64::Engine;

    let decoded = STANDARD_NO_PAD.decode(claim_str)?;
    let claims = serde_json::from_slice::<UserWithIdSid>(&decoded)?;
    let sid = claims.sid.ok_or("jwt token claim not correct")?;
    let id = claims.id.ok_or("jwt token claim not correct")?;

    context.inner_context.id = Some(id);
    context.inner_context.sid = Some(sid);

    Ok(context)
}

pub fn pre_check_permission_for_insert(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let id = context.inner_context.id.clone().ok_or("user id not found")?;
    let app_id = context.input.app_id.clone().ok_or("app id not found")?;

    let mut dapr_comp = None;

    let sql = r#"
select
    r.id as rel_id
from
    public.rel_user_app_role r, public.role as l
where
    l.id = r.role_id
    and r.user_id = ?
    and r.app_id = ?;
"#;

    let context = context
        .dapr_invoke_binding_sql("query_rel_exist", "sg-base-role")?
        .get_current_dapr_component(|d| dapr_comp = d)?
        .dapr_invoke_binding_sql_operation(SqlOperation::Query)?
        .dapr_invoke_binding_sql_sqls(
            SqlsBuilder::new()
                .dapr_component(dapr_comp.as_ref().ok_or("dapr component not found")?)
                .operation(SqlOperation::Query)
                .sql_builder(
                    SqlBuilder::new()
                        .sql(sql)
                        .param_extend(rbs::Value::I64(id.to_owned().parse()?))
                        .param_extend(rbs::Value::I64(app_id)),
                )
                .build()?,
        )?;

    Ok(context)
}

pub fn post_check_permission_for_insert(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let (context, _) = context
        .decode_sql_one::<RelId>("query_rel_exist")
        .map_err(|_| err_boxed_full(AUTH_ERROR, "you don't have permission to access the app"))?
        .get_dapr_resp_one::<RelId>("query_rel_exist")
        .map_err(|_| err_boxed_full(AUTH_ERROR, "you don't have permission to access the app"))?;

    Ok(context)
}

pub fn pre_get_snowflake_id(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let context = context
        .dapr_invoke_service("get_snowflake_id", "id-serverlessguide-dev", MethodEnum::GET)?
        .dapr_invoke_service_uri("id/bulk")?
        .dapr_invoke_service_content_type("application/json")?
        .dapr_invoke_service_query_string("num=1")?;

    Ok(context)
}

pub fn pre_insert(context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let (context, bulk_ids) = context
        .decode_sql_one::<BulkIdRes>("get_snowflake_id")?
        .get_dapr_resp_one::<BulkIdRes>("get_snowflake_id")?;

    let ids = bulk_ids.result;
    if ids.len() != 1 {
        return Err(err_boxed_full(DATA_ERROR, "get ids from id length not 1"));
    }

    let mut data = context.input.clone();
    let time = utc_timestamp();
    data.create_time = Some(time.to_string());
    data.update_time = Some(time.to_string());
    data.active = Some(true);
    data.id = Some(ids[0]);

    let mut dapr_comp = None;

    let context = context
        .dapr_invoke_binding_sql("insert", "sg-base-role")?
        .get_current_dapr_component(|d| dapr_comp = d)?
        .dapr_invoke_binding_sql_operation(SqlOperation::Exec)?
        .dapr_invoke_binding_sql_sqls(trans_sql_info(
            AppVersion::insert(&data)?,
            SqlOperation::Exec,
            dapr_comp.as_ref().ok_or("dapr component not found")?,
        )?)?;

    Ok(context)
}

pub fn post_insert(context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    Ok(context)
}

pub fn pre_check_permission_for_env_prepare(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let id = context.inner_context.id.clone().ok_or("user id not found")?;
    let app_version_id = context.input.id.ok_or("app version id not found")?;

    let mut dapr_comp = None;

    let sql = r#"
select
    r.id as rel_id
from
    public.rel_user_app_role r, public.role l, public.app_version v
where
    l.id = r.role_id
    and v.app_id = r.app_id
    and l.code != 'StandBy'
    and r.user_id = ?
    and v.id = ?;
"#;

    let context = context
        .dapr_invoke_binding_sql("query_rel_exist", "sg-base-role")?
        .get_current_dapr_component(|d| dapr_comp = d)?
        .dapr_invoke_binding_sql_operation(SqlOperation::Query)?
        .dapr_invoke_binding_sql_sqls(
            SqlsBuilder::new()
                .dapr_component(dapr_comp.as_ref().ok_or("dapr component not found")?)
                .operation(SqlOperation::Query)
                .sql_builder(
                    SqlBuilder::new()
                        .sql(sql)
                        .param_extend(rbs::Value::I64(id.parse()?))
                        .param_extend(rbs::Value::I64(app_version_id)),
                )
                .build()?,
        )?;

    Ok(context)
}

pub fn post_check_permission_for_env_prepare(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let (context, _) = context
        .decode_sql_one::<RelId>("query_rel_exist")
        .map_err(|_| err_boxed_full(AUTH_ERROR, "you don't have permission to access the app"))?
        .get_dapr_resp_one::<RelId>("query_rel_exist")
        .map_err(|_| err_boxed_full(AUTH_ERROR, "you don't have permission to access the app"))?;

    Ok(context)
}

pub fn pre_prepare_env(context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let (context, app_info) = context
        .decode_sql_one::<AppCodeAndVersion>("query_by_app_version_id")?
        .get_dapr_resp_one::<AppCodeAndVersion>("query_by_app_version_id")?;

    let app_code = app_info.code.clone().ok_or("app code not found")?;
    let app_version = app_info.version.clone().ok_or("app version not found")?;
    let app_domain = app_info.domain.clone().ok_or("app domain not found")?;

    let faas_namespace = format!("{}-{}", app_code, app_version.replace(".", "-"));
    let mid_namespace = format!("{}-{}-mid", app_code, app_version.replace(".", "-"));

    let mut metadata = HashMap::<String, String>::new();
    metadata.insert(String::from("path"), String::from("/app"));

    let mut target: AppInMapBuilder = Default::default();
    target.name = Some(app_code);
    target.version = Some(app_version);
    target.namespaces = vec![faas_namespace];
    target.rq_namespaces = vec![mid_namespace];
    target.domains = vec![app_domain];

    let context = context
        .dapr_invoke_binding("prepare_env", "sg-base-role-map-builder-svc")?
        .dapr_invoke_binding_operation("post")?
        .dapr_invoke_binding_metadata(metadata)?
        .dapr_invoke_binding_data(serde_json::to_vec(&target)?)?;

    Ok(context)
}

pub fn post_prepare_env(context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    Ok(context)
}

pub fn pre_query_by_app_version_id(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let id = context.input.id.ok_or("id not found")?;

    let mut dapr_comp = None;

    let sql = r#"
select
    p.code,
    v.version,
    p.domain
from
    public.app p, public.app_version v
where
    p.id = v.app_id
    and v.id = ?;
"#;

    let context = context
        .dapr_invoke_binding_sql("query_by_app_version_id", "sg-base-role")?
        .get_current_dapr_component(|d| dapr_comp = d)?
        .dapr_invoke_binding_sql_operation(SqlOperation::Query)?
        .dapr_invoke_binding_sql_sqls(
            SqlsBuilder::new()
                .dapr_component(dapr_comp.as_ref().ok_or("dapr component not found")?)
                .operation(SqlOperation::Query)
                .sql_builder(SqlBuilder::new().sql(sql).param_extend(rbs::Value::I64(id)))
                .build()?,
        )?;

    Ok(context)
}

pub fn post_query_by_app_version_id(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let (context, _) = context
        .decode_sql_one::<AppCodeAndVersion>("query_by_app_version_id")?
        .get_dapr_resp_one::<AppCodeAndVersion>("query_by_app_version_id")?;

    Ok(context)
}
