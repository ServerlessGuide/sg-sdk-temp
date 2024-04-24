use crate::*;

lazy_static! {}

pub fn prepare_inner_context<I: DaprBody + ModelTrait + Default + prost::Message, O: DaprBody + ModelTrait + Default + prost::Message>(
    mut context: ContextWrapper<I, O, UserWithIdSid>,
) -> HttpResult<ContextWrapper<I, O, UserWithIdSid>> {
    let jwt_token_val = if let Some(v) = context.header.get(AuthHeader::XSGAuthJWT.lower_case_value()) {
        v.to_string()
    } else if let Some(v) = context.header.get(AuthHeader::XSGAuthJWT.lower_case_value()) {
        v.to_string()
    } else {
        return Err(Box::new(util::gen_resp_err(DATA_ERROR, Some(String::from("jwt header not found")))));
    };

    let claim_vec = jwt_token_val.split(".").collect::<Vec<&str>>();
    let claim_str = claim_vec.get(1).ok_or("jwt token format error")?;

    use base64::engine::general_purpose::*;
    use base64::Engine;

    let decoded = STANDARD_NO_PAD.decode(claim_str)?;
    let claims = serde_json::from_slice::<UserWithIdSid>(&decoded)?;
    let sid = claims.sid.ok_or("jwt token claim not correct")?;
    let id = claims.id.ok_or("jwt token claim not correct")?;

    context.inner_context = Some(UserWithIdSid {
        id: Some(id),
        sid: Some(sid),
        app_code: None,
        snow_id: None,
        app_version: None,
        db_database_id: None,
        db_database_name: None,
        db_user_name: None,
        db_user_password: None,
        require_instance_name: None,
        require_name: None,
        require_type: None,
        sm_name: None,
    });

    Ok(context)
}

pub fn pre_check_user_for_query_all(
    context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_app_info";

    let dapr_config = find_dapr_config("db-sm")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding_sql(&dapr_config)?;
    let invoke_binding_sql = dapr_req_ins.invoke_binding_sql.as_mut().ok_or("DaprRequest.invoke_binding_sql make error")?;

    let inner_context = context.inner_context.clone().ok_or("inner context not exist")?;

    let user_id = &inner_context.id.ok_or("user id not found")?;

    let operation = SqlOperation::Query;
    invoke_binding_sql.operation = operation.clone();
    invoke_binding_sql.sqls = trans_sql_info(
        vec![(
            String::from(
                "select
    r.id as rel_id,
    c.code as app_code,
    v.version as app_version
from
    public.rel_user_app_role r, public.role as l, public.app_require as b, public.db_database a, public.app as c, public.app_version v
where
    l.id = r.role_id
    and r.app_id = b.app_id
    and v.id = b.app_version_id
    and a.app_require_id = b.id
    and r.app_id = c.id
    and r.user_id = ?
    and a.id = ?;
",
            ),
            vec![
                rbs::Value::I64(user_id.to_owned().parse()?),
                rbs::Value::I64(
                    context
                        .input
                        .clone()
                        .ok_or("input param not found")?
                        .db_database_id
                        .ok_or("db_database_id not found")?,
                ),
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

pub fn post_check_user_for_query_all(
    mut context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_app_info";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding_sql
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding_sql response not found", execute_name))?;

    let first = response.responses.first().unwrap();
    let res = de_sql_result_implicit::<RelId>(&first.data, &first.output_columns, RelId::enum_convert)?;
    if res.len() != 1 {
        return Err(Box::new(util::gen_resp_err(
            AUTH_ERROR,
            Some(String::from("you don't have permission to access the app")),
        )));
    }

    let mut c_t = context.inner_context.clone().ok_or("inner context not found")?;
    c_t.app_code = Some(res[0].app_code.to_owned().ok_or("app code not found")?);
    c_t.app_version = Some(res[0].app_version.to_owned().ok_or("app version not found")?);

    context.inner_context = Some(c_t);

    Ok(context)
}

pub fn pre_check_user_for_query_by_id(
    context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_app_info";

    let dapr_config = find_dapr_config("db-sm")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding_sql(&dapr_config)?;
    let invoke_binding_sql = dapr_req_ins.invoke_binding_sql.as_mut().ok_or("DaprRequest.invoke_binding_sql make error")?;

    let inner_context = context.inner_context.clone().ok_or("inner context not exist")?;

    let user_id = &inner_context.id.ok_or("user id not found")?;

    let operation = SqlOperation::Query;
    invoke_binding_sql.operation = operation.clone();
    invoke_binding_sql.sqls = trans_sql_info(
        vec![(
            String::from(
                "select
    r.id as rel_id,
    c.code as app_code,
    v.version as app_version
from
    public.rel_user_app_role r, public.role as l, public.app_require as b, public.db_database a, public.app as c, public.app_version v, public.db_sm s
where
    l.id = r.role_id
    and r.app_id = b.app_id
    and v.id = b.app_version_id
    and a.app_require_id = b.id
    and r.app_id = c.id
    and s.db_database_id = a.id
    and r.user_id = ?
    and s.id = ?;
",
            ),
            vec![
                rbs::Value::I64(user_id.to_owned().parse()?),
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

pub fn post_check_user_for_query_by_id(
    mut context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_app_info";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding_sql
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding_sql response not found", execute_name))?;

    let first = response.responses.first().unwrap();
    let res = de_sql_result_implicit::<RelId>(&first.data, &first.output_columns, RelId::enum_convert)?;
    if res.len() != 1 {
        return Err(Box::new(util::gen_resp_err(
            AUTH_ERROR,
            Some(String::from("you don't have permission to access the app")),
        )));
    }

    let mut c_t = context.inner_context.clone().ok_or("inner context not found")?;
    c_t.app_code = Some(res[0].app_code.to_owned().ok_or("app code not found")?);
    c_t.app_version = Some(res[0].app_version.to_owned().ok_or("app version not found")?);

    context.inner_context = Some(c_t);

    Ok(context)
}

pub fn pre_query_one_by_id_sql(
    context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_by_id";

    let dapr_config = find_dapr_config("db-sm")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding_sql(&dapr_config)?;
    let invoke_binding_sql = dapr_req_ins.invoke_binding_sql.as_mut().ok_or("DaprRequest.invoke_binding_sql make error")?;

    let id = context.input.clone().ok_or("input not exist")?.id.ok_or("id not exist")?;

    let operation = SqlOperation::Query;
    invoke_binding_sql.operation = operation.clone();
    invoke_binding_sql.sqls = trans_sql_info(vec![DBStorageModel::select_by_column("id", id)?], operation.clone(), &dapr_config)?;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub fn post_query_one_by_id_sql(
    mut context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_by_id";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding_sql
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding_sql response not found", execute_name))?;

    if response.responses.is_empty() {
        return Err(Box::new(util::gen_resp_err(DATA_NOT_FOUND, None)));
    }

    let first = response.responses.first().unwrap();
    let res = de_sql_result_implicit_first::<DBStorageModel>(&first.data, &first.output_columns, DBStorageModel::enum_convert)?;

    let mut c_t = context.inner_context.clone().ok_or("inner context not found")?;
    c_t.db_database_id = Some(res.db_database_id.clone().ok_or("db database id not found")?);
    c_t.sm_name = Some(res.name.clone().ok_or("sm name not found")?);

    context.inner_context = Some(c_t);

    Ok(context)
}

pub fn pre_query_all_file(
    context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_all_files";

    let dapr_config = find_dapr_config("db-sm-minio")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding(&dapr_config)?;
    let invoke_binding = dapr_req_ins.invoke_binding.as_mut().ok_or("DaprRequest.invoke_binding make error")?;

    let db_database_id = context
        .input
        .clone()
        .ok_or("input not exist")?
        .db_database_id
        .ok_or("db database id not exist")?;

    invoke_binding.operation = String::from("list");
    let mut data = HashMap::<String, String>::new();
    data.insert(
        String::from("prefix"),
        format!(
            "{}/{}/StorageModel/{}-",
            context
                .inner_context
                .clone()
                .ok_or("inner context not found")?
                .app_code
                .ok_or("app code not found")?,
            context
                .inner_context
                .clone()
                .ok_or("inner context not found")?
                .app_version
                .ok_or("app version not found")?,
            db_database_id
        ),
    );

    let json_bytes = serde_json::json!(data).to_string().as_bytes().to_vec();
    invoke_binding.data = json_bytes;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub async fn post_query_all_file(
    mut context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_all_files";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding response not found", execute_name))?;

    if response.data.is_empty() {
        return Err(Box::new(util::gen_resp_err(DATA_NOT_FOUND, None)));
    }
    println!("response data from minio: {}", String::from_utf8_lossy(&response.data));

    let object_list = serde_json::from_slice::<ObjectList>(&response.data)?;

    let mut dsls = Vec::<StorageModelInfo>::new();

    for e in object_list.Contents {
        let key = e.Key.ok_or("file key not found")?;

        let dapr_config = find_dapr_config("db-sm-minio")?;

        let mut dapr_req_ins = DaprRequest::make_invoke_binding(&dapr_config)?;
        let invoke_binding = dapr_req_ins.invoke_binding.as_mut().ok_or("DaprRequest.invoke_binding make error")?;

        let mut metadata = HashMap::<String, String>::new();
        metadata.insert(String::from("key"), key);

        let response = get_dapr_client()
            .await?
            .invoke_binding(invoke_binding.name.clone(), vec![], metadata, String::from("get"))
            .await;

        debug!("invoke dapr binding response: {:?}", response);

        if let Err(err) = response {
            return Err(Box::new(util::gen_resp_err(DAPR_REQUEST_FAIL, Some(err.to_string()))));
        }
        let response = response.unwrap();

        let dsl = serde_json::from_slice::<StorageModelInfo>(&response.data)?;
        dsls.push(dsl);
    }

    context.outputs = dsls;

    Ok(context)
}

pub fn pre_query_one_by_id(
    context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_by_id";

    let dapr_config = find_dapr_config("db-sm-minio")?;
    let mut dapr_req_ins = DaprRequest::make_invoke_binding(&dapr_config)?;
    let invoke_binding = dapr_req_ins.invoke_binding.as_mut().ok_or("DaprRequest.invoke_binding make error")?;

    invoke_binding.operation = String::from("list");
    let mut data = HashMap::<String, String>::new();
    data.insert(
        String::from("prefix"),
        format!(
            "{}/{}/StorageModel/{}-{}.json",
            context
                .inner_context
                .clone()
                .ok_or("inner context not found")?
                .app_code
                .ok_or("app code not found")?,
            context
                .inner_context
                .clone()
                .ok_or("inner context not found")?
                .app_version
                .ok_or("app version not found")?,
            context
                .inner_context
                .clone()
                .ok_or("inner context not exist")?
                .db_database_id
                .ok_or("db database id not exist")?,
            context
                .inner_context
                .clone()
                .ok_or("inner context not found")?
                .sm_name
                .ok_or("sm name not found")?,
        ),
    );

    let json_bytes = serde_json::json!(data).to_string().as_bytes().to_vec();
    invoke_binding.data = json_bytes;

    Ok(set_dapr_req(context, dapr_req_ins, execute_name)?)
}

pub async fn post_query_one_by_id(
    mut context: ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>,
) -> HttpResult<ContextWrapper<DBStorageModel, StorageModelInfo, UserWithIdSid>> {
    let execute_name = "query_by_id";
    let (_, res, _) = find_dapr_execute(&mut context.exec, execute_name)?;

    let response = res
        .invoke_binding
        .clone()
        .ok_or(format!("execute '{}' of invoke_binding response not found", execute_name))?;

    if response.data.is_empty() {
        return Err(Box::new(util::gen_resp_err(DATA_NOT_FOUND, None)));
    }
    println!("response data from minio: {}", String::from_utf8_lossy(&response.data));

    let object_list = serde_json::from_slice::<ObjectList>(&response.data)?;

    if object_list.Contents.len() != 1 {
        return Err(Box::new(util::gen_resp_err(DATA_NOT_FOUND, None)));
    }

    let key = object_list.Contents[0].Key.clone().ok_or("file key not found")?;

    let dapr_config = find_dapr_config("db-sm-minio")?;

    let mut dapr_req_ins = DaprRequest::make_invoke_binding(&dapr_config)?;
    let invoke_binding = dapr_req_ins.invoke_binding.as_mut().ok_or("DaprRequest.invoke_binding make error")?;

    let mut metadata = HashMap::<String, String>::new();
    metadata.insert(String::from("key"), key);

    let response = get_dapr_client()
        .await?
        .invoke_binding(invoke_binding.name.clone(), vec![], metadata, String::from("get"))
        .await;

    debug!("invoke dapr binding response: {:?}", response);

    if let Err(err) = response {
        return Err(Box::new(util::gen_resp_err(DAPR_REQUEST_FAIL, Some(err.to_string()))));
    }
    let response = response.unwrap();

    let dsl = serde_json::from_slice::<StorageModelInfo>(&response.data)?;

    context.output = Some(dsl);

    Ok(context)
}
