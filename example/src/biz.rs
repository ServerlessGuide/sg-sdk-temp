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

    context.inner_context = Some(UserWithIdSid { id: Some(id), sid: Some(sid) });

    Ok(context)
}

pub fn pre_check_permission(
    context: ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>,
) -> HttpResult<ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>> {
    let execute_name = "query_rel_exist";

    let dapr_config = find_dapr_config("app-version")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding_sql(&dapr_config)?;
    let invoke_binding_sql = dapr_req_ins.invoke_binding_sql.as_mut().ok_or("DaprRequest.invoke_binding_sql make error")?;

    let inner_context = context.inner_context.clone().ok_or("inner context not exist")?;

    let id = &inner_context.id.ok_or("user id not found")?;

    let operation = SqlOperation::Query;
    invoke_binding_sql.operation = operation.clone();
    invoke_binding_sql.sqls = trans_sql_info(
        vec![(
            String::from(
                "select
    r.id as rel_id
from
    public.rel_user_app_role r, public.role as l
where
    l.id = r.role_id
    and r.user_id = ?
    and r.app_id = ?;
",
            ),
            vec![
                rbs::Value::I64(id.to_owned().parse()?),
                rbs::Value::I64(context.input.clone().ok_or("input param not found")?.app_id.ok_or("app id not found")?),
            ],
            false,
            None,
            None,
        )],
        operation.clone(),
        &dapr_config,
    )?;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub fn post_check_permission(
    mut context: ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>,
) -> HttpResult<ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>> {
    let execute_name = "query_rel_exist";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding_sql
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding_sql response not found", execute_name))?;

    let first = response.responses.first().unwrap();
    let res = de_sql_result_implicit::<RelId>(&first.data, &first.output_columns, RelId::enum_convert)?;
    if res.len() != 1 {
        return Err(err_boxed_full(AUTH_ERROR, "you don't have permission to access the app"));
    }

    Ok(context)
}

pub fn pre_query_by_app_id(
    context: ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>,
) -> HttpResult<ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>> {
    let execute_name = "query_by_app_id";

    let dapr_config = find_dapr_config("app-version")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding_sql(&dapr_config)?;
    let invoke_binding_sql = dapr_req_ins.invoke_binding_sql.as_mut().ok_or("DaprRequest.invoke_binding_sql make error")?;

    let app_id = context.input.clone().ok_or("input not exist")?.app_id.ok_or("app_id not exist")?;

    let operation = SqlOperation::Query;
    invoke_binding_sql.operation = operation.clone();
    invoke_binding_sql.sqls = trans_sql_info(vec![AppVersion::select_by_column("app_id", app_id)?], operation.clone(), &dapr_config)?;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub fn post_query_by_app_id(
    mut context: ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>,
) -> HttpResult<ContextWrapper<QueryAppVersions, AppVersion, UserWithIdSid>> {
    let execute_name = "query_by_app_id";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding_sql
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding_sql response not found", execute_name))?;

    if response.responses.is_empty() {
        return Err(err_boxed(DATA_NOT_FOUND));
    }

    let first = response.responses.first().unwrap();
    let res = de_sql_result_implicit::<AppVersion>(&first.data, &first.output_columns, AppVersion::enum_convert)?;
    context.outputs = res;

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

    context.inner_context = Some(UserWithIdSid { id: Some(id), sid: Some(sid) });

    Ok(context)
}

pub fn pre_check_permission_for_insert(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let execute_name = "query_rel_exist";

    let dapr_config = find_dapr_config("app-version")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding_sql(&dapr_config)?;
    let invoke_binding_sql = dapr_req_ins.invoke_binding_sql.as_mut().ok_or("DaprRequest.invoke_binding_sql make error")?;

    let inner_context = context.inner_context.clone().ok_or("inner context not exist")?;

    let id = &inner_context.id.ok_or("user id not found")?;

    let operation = SqlOperation::Query;
    invoke_binding_sql.operation = operation.clone();
    invoke_binding_sql.sqls = trans_sql_info(
        vec![(
            String::from(
                "select
    r.id as rel_id
from
    public.rel_user_app_role r, public.role as l
where
    l.id = r.role_id
    and r.user_id = ?
    and r.app_id = ?;
",
            ),
            vec![
                rbs::Value::I64(id.to_owned().parse()?),
                rbs::Value::I64(context.input.clone().ok_or("input param not found")?.app_id.ok_or("app id not found")?),
            ],
            false,
            None,
            None,
        )],
        operation.clone(),
        &dapr_config,
    )?;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub fn post_check_permission_for_insert(
    mut context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let execute_name = "query_rel_exist";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding_sql
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding_sql response not found", execute_name))?;

    let first = response.responses.first().unwrap();
    let res = de_sql_result_implicit::<RelId>(&first.data, &first.output_columns, RelId::enum_convert)?;
    if res.len() != 1 {
        return Err(err_boxed_full(AUTH_ERROR, "you don't have permission to access the app"));
    }

    Ok(context)
}

pub fn pre_get_snowflake_id(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let execute_name = "get_snowflake_id";

    let dapr_req_ins = DaprRequest::make_invoke_service(
        "id-serverlessguide-dev".to_string(),
        "id/bulk".to_string(),
        "application/json".to_string(),
        MethodEnum::GET,
        format!("num={}", 1),
    )?;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub fn pre_insert(mut context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let execute_name = "get_snowflake_id";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_service
        .as_mut()
        .ok_or(format!("execute '{}' of invoke_service response not found", execute_name))?;

    let Some(data) = &response.data else {
        return Err(err_boxed(DATA_NOT_FOUND));
    };

    let ids = de_any_json::<BulkIdRes>(data)?
        .downcast_mut::<BulkIdRes>()
        .ok_or("downcast error")?
        .result
        .clone();

    if ids.len() != 1 {
        return Err(err_boxed_full(DATA_ERROR, "get ids from id length not 1"));
    }

    let execute_name = "insert";

    let dapr_config = find_dapr_config("app-version")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding_sql(&dapr_config)?;
    let invoke_binding_sql = dapr_req_ins.invoke_binding_sql.as_mut().ok_or("DaprRequest.invoke_binding_sql make error")?;

    let mut data = context.input.clone().unwrap();
    let time = utc_timestamp();
    data.create_time = Some(time.to_string());
    data.update_time = Some(time.to_string());
    data.active = Some(true);
    data.id = Some(ids[0]);

    let operation = SqlOperation::Exec;
    invoke_binding_sql.operation = operation.clone();
    invoke_binding_sql.sqls = trans_sql_info(AppVersion::insert(&data)?, operation.clone(), &dapr_config)?;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub fn post_insert(mut context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    context.output = Some(EmptyOutPut {});

    Ok(context)
}

pub fn pre_check_permission_for_env_prepare(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let execute_name = "query_rel_exist";

    let dapr_config = find_dapr_config("app-version")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding_sql(&dapr_config)?;
    let invoke_binding_sql = dapr_req_ins.invoke_binding_sql.as_mut().ok_or("DaprRequest.invoke_binding_sql make error")?;

    let inner_context = context.inner_context.clone().ok_or("inner context not exist")?;

    let id = &inner_context.id.ok_or("user id not found")?;

    let operation = SqlOperation::Query;
    invoke_binding_sql.operation = operation.clone();
    invoke_binding_sql.sqls = trans_sql_info(
        vec![(
            String::from(
                "select
    r.id as rel_id
from
    public.rel_user_app_role r, public.role l, public.app_version v
where
    l.id = r.role_id
    and v.app_id = r.app_id
    and l.code != 'StandBy'
    and r.user_id = ?
    and v.id = ?;
",
            ),
            vec![
                rbs::Value::I64(id.to_owned().parse()?),
                rbs::Value::I64(context.input.clone().ok_or("input param not found")?.id.ok_or("id not found")?),
            ],
            false,
            None,
            None,
        )],
        operation.clone(),
        &dapr_config,
    )?;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub fn post_check_permission_for_env_prepare(
    mut context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let execute_name = "query_rel_exist";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding_sql
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding_sql response not found", execute_name))?;

    let first = response.responses.first().unwrap();
    let res = de_sql_result_implicit::<RelId>(&first.data, &first.output_columns, RelId::enum_convert)?;
    if res.len() != 1 {
        return Err(err_boxed_full(AUTH_ERROR, "you don't have permission to access the app"));
    }

    Ok(context)
}

pub fn pre_prepare_env(
    mut context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let execute_name = "query_by_app_version_id";
    let (_, _, de_res) = find_dapr_execute(&mut context.exec, execute_name)?;

    let de_res = de_res.as_mut().ok_or(err_boxed(DATA_NOT_FOUND))?;
    if de_res.is_empty() {
        return Err(err_boxed(DATA_NOT_FOUND));
    }
    let app_info = de_res[0].downcast_mut::<AppCodeAndVersion>().ok_or("downcast error")?;
    let app_code = app_info.code.clone().ok_or("app code not found")?;
    let app_version = app_info.version.clone().ok_or("app version not found")?;
    let app_domain = app_info.domain.clone().ok_or("app domain not found")?;

    let faas_namespace = format!("{}-{}", app_code, app_version.replace(".", "-"));
    let mid_namespace = format!("{}-{}-mid", app_code, app_version.replace(".", "-"));

    let execute_name = "prepare_env";

    let dapr_config = find_dapr_config("app-version-map-builder-svc")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding(&dapr_config)?;
    let invoke_binding = dapr_req_ins.invoke_binding.as_mut().ok_or("DaprRequest.invoke_binding make error")?;

    invoke_binding.operation = String::from("post");
    let mut metadata = HashMap::<String, String>::new();
    metadata.insert(String::from("path"), String::from("/app"));
    invoke_binding.metadata = metadata;

    let mut target: AppInMapBuilder = Default::default();
    target.name = Some(app_code);
    target.version = Some(app_version);
    target.namespaces = vec![faas_namespace];
    target.rq_namespaces = vec![mid_namespace];
    target.domains = vec![app_domain];

    invoke_binding.data = serde_json::to_vec(&target)?;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub fn post_prepare_env(
    mut context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    context.output = Some(EmptyOutPut {});

    Ok(context)
}

pub fn pre_query_by_app_version_id(
    context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let execute_name = "query_by_app_version_id";

    let dapr_config = find_dapr_config("app-version")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding_sql(&dapr_config)?;
    let invoke_binding_sql = dapr_req_ins.invoke_binding_sql.as_mut().ok_or("DaprRequest.invoke_binding_sql make error")?;

    let operation = SqlOperation::Query;
    invoke_binding_sql.operation = operation.clone();
    invoke_binding_sql.sqls = trans_sql_info(
        vec![(
            String::from(
                "select
    p.code,
    v.version,
    p.domain
from
    public.app p, public.app_version v
where
    p.id = v.app_id
    and v.id = ?;",
            ),
            vec![rbs::Value::I64(context.input.clone().ok_or("input param not found")?.id.ok_or("id not found")?)],
            false,
            None,
            None,
        )],
        operation.clone(),
        &dapr_config,
    )?;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub fn post_query_by_app_version_id(
    mut context: ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>,
) -> HttpResult<ContextWrapper<AppVersion, EmptyOutPut, UserWithIdSid>> {
    let execute_name = "query_by_app_version_id";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding_sql
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding_sql response not found", execute_name))?;

    if response.responses.is_empty() {
        return Err(err_boxed(DATA_NOT_FOUND));
    }

    let first = response.responses.first().unwrap();
    let res = de_sql_result_implicit_first::<AppCodeAndVersion>(&first.data, &first.output_columns, AppCodeAndVersion::enum_convert)?;

    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
    dapr_res.push(Box::new(res));

    Ok(set_dapr_res(context, dapr_res, execute_name)?)
}
