use dapr::dapr::dapr::proto::{common::v1::*, runtime::v1::*};

use crate::{model::*, traits::*, util::*, *};

use self::inner_biz_result::*;

impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C> ContextWrapper<I, O, C> {
    pub fn get_current_dapr_component<F>(mut self, f: F) -> HttpResult<ContextWrapper<I, O, C>>
    where
        F: FnOnce(Option<DaprComponentInfo>),
    {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        f(req._dapr_config.clone());

        Ok(self)
    }
}

// 这里全部是`invoke_binding_sql`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C> ContextWrapper<I, O, C> {
    pub fn set_binding_sql_operation(mut self, operation: SqlOperation) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.operation = operation;

        Ok(self)
    }

    pub fn set_binding_sql_sqls(mut self, sqls: Vec<SqlWithParams>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.sqls = sqls;

        Ok(self)
    }

    pub fn set_binding_sql_data(mut self, data: Vec<u8>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.data = data;

        Ok(self)
    }

    pub fn set_binding_sql_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }

    pub fn set_binding_sql_select_page(mut self, is_select_page: bool) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.is_select_page = Some(is_select_page);

        Ok(self)
    }
}

// 这里全部是`invoke_binding`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C> ContextWrapper<I, O, C> {
    pub fn set_binding_operation(mut self, operation: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding.as_mut().ok_or("please init dapr request first")?;

        req.operation = operation.to_string();

        Ok(self)
    }

    pub fn set_binding_data(mut self, data: Vec<u8>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding.as_mut().ok_or("please init dapr request first")?;

        req.data = data;

        Ok(self)
    }

    pub fn set_binding_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C> ContextWrapper<I, O, C> {
    pub fn dapr_get_state(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_get_state()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_get_bulk_state(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_get_bulk_state()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_query_state(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_query_state()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_save_state(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_save_state()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_transaction_state(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_transaction_state()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_delete_state(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_delete_state()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_delete_bulk_state(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_delete_bulk_state()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_invoke_binding(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_invoke_binding()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_invoke_binding_sql(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_invoke_binding_sql()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_publish_event(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_publish_event()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_publish_bulk_event(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_publish_bulk_event()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_get_secret(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_get_secret()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_get_bulk_secret(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_get_bulk_secret()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_get_configuration(self, exec_name: &str, component_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let dapr_req_ins = find_dapr_binding(component_name)?.make_get_configuration()?;

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }

    pub fn dapr_invoke_service(
        self,
        exec_name: &str,
        id: &str,
        method: &str,
        content_type: &str,
        http_method: MethodEnum,
        query_string: &str,
    ) -> HttpResult<ContextWrapper<I, O, C>> {
        let mut dapr_req_ins: DaprRequest = Default::default();

        dapr_req_ins.invoke_service = Some(InvokeServiceRequest {
            id: id.to_string(),
            message: Some(InvokeRequest {
                method: method.to_string(),
                data: None,
                content_type: content_type.to_string(),
                http_extension: Some(HttpExtension {
                    verb: http_method.to_i32(),
                    querystring: query_string.to_string(),
                }),
                headers: HashMap::<String, String>::new(),
            }),
        });

        Ok(set_dapr_req(self, dapr_req_ins, exec_name)?)
    }
}

impl DaprComponentInfo {
    pub fn make_get_state(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::State != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "state")));
        };

        s.get_state = Some(GetStateRequest {
            store_name: self.name.clone(),
            key: "".to_string(),
            consistency: 2,
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }

    pub fn make_get_bulk_state(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::State != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "state")));
        };

        s.get_bulk_state = Some(GetBulkStateRequest {
            store_name: self.name.clone(),
            keys: vec!["".to_string()],
            parallelism: 1,
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }

    pub fn make_query_state(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::State != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "state")));
        };

        s.query_state = Some(QueryStateRequest {
            store_name: self.name.clone(),
            query: "".to_string(),
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }

    pub fn make_save_state(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::State != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "state")));
        };

        s.save_state = Some(SaveStateRequest {
            store_name: self.name.clone(),
            states: vec![],
        });

        Ok(s)
    }

    pub fn make_transaction_state(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::State != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "state")));
        };

        s.transaction_state = Some(ExecuteStateTransactionRequest {
            store_name: self.name.clone(),
            operations: vec![],
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }

    pub fn make_delete_state(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::State != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "state")));
        };

        s.delete_state = Some(DeleteStateRequest {
            store_name: self.name.clone(),
            key: "".to_string(),
            etag: None,
            options: None,
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }

    pub fn make_delete_bulk_state(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::State != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "state")));
        };

        s.delete_bulk_state = Some(DeleteBulkStateRequest {
            store_name: self.name.clone(),
            states: vec![],
        });

        Ok(s)
    }

    pub fn make_invoke_binding(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::Binding != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "binding")));
        };

        s.invoke_binding = Some(InvokeBindingRequest {
            name: self.name.clone(),
            data: vec![],
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
            operation: "query".to_string(),
        });

        Ok(s)
    }

    pub fn make_invoke_binding_sql(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::Binding != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "binding")));
        };

        s.invoke_binding_sql = Some(InvokeBindingSqlRequest {
            name: self.name.clone(),
            data: vec![],
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
            operation: SqlOperation::Query,
            sqls: vec![],
            is_select_page: None,
        });

        Ok(s)
    }

    pub fn make_publish_event(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::Pubsub != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "pubsub")));
        };

        s.publish_event = Some(PublishEventRequest {
            pubsub_name: self.name.clone(),
            topic: self.topic.clone().ok_or(err_boxed_full_string(
                DAPR_COMPONENT_NOT_EXIST,
                format!("{}.{}.topic not found", "DaprConfig", "pubsub"),
            ))?,
            data: vec![],
            data_content_type: "application/json".to_string(),
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }

    pub fn make_publish_bulk_event(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::Pubsub != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "pubsub")));
        };

        s.publish_bulk_event = Some(BulkPublishRequest {
            pubsub_name: self.name.clone(),
            topic: self.topic.clone().ok_or(err_boxed_full_string(
                DAPR_COMPONENT_NOT_EXIST,
                format!("{}.{}.topic not found", "DaprConfig", "pubsub"),
            ))?,
            entries: vec![],
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }

    pub fn make_get_secret(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::Secret != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "secret")));
        };

        s.get_secret = Some(GetSecretRequest {
            store_name: self.name.clone(),
            key: "".to_string(),
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }

    pub fn make_get_bulk_secret(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::Secret != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "secret")));
        };

        s.get_bluk_secret = Some(GetBulkSecretRequest {
            store_name: self.name.clone(),
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }

    pub fn make_get_configuration(&self) -> HttpResult<DaprRequest> {
        let mut s: DaprRequest = Default::default();
        s._dapr_config = Some(self.clone());

        if DaprBuildBlockType::Conf != self.bb_type {
            return Err(err_boxed_full_string(DAPR_COMPONENT_NOT_EXIST, format!("{}.{}", "DaprConfig", "conf")));
        };

        s.get_configuration = Some(GetConfigurationRequest {
            store_name: self.name.clone(),
            keys: vec![],
            metadata: self.metadata.clone().map_or(HashMap::new(), |v| v),
        });

        Ok(s)
    }
}
