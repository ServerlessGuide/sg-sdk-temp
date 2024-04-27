use dapr::dapr::dapr::proto::{common::v1::*, runtime::v1::*};

use crate::{model::*, traits::*, util::*, *};

use self::inner_biz_result::*;

impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
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
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_invoke_binding_sql_operation(mut self, operation: SqlOperation) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.operation = operation;

        Ok(self)
    }

    pub fn dapr_invoke_binding_sql_sqls(mut self, sqls: Vec<SqlWithParams>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.sqls = sqls;

        Ok(self)
    }

    pub fn dapr_invoke_binding_sql_data(mut self, data: Vec<u8>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.data = data;

        Ok(self)
    }

    pub fn dapr_invoke_binding_sql_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }

    pub fn dapr_invoke_binding_sql_select_page(mut self, is_select_page: bool) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding_sql.as_mut().ok_or("please init dapr request first")?;

        req.is_select_page = Some(is_select_page);

        Ok(self)
    }
}

// 这里全部是`invoke_binding`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_invoke_binding_operation(mut self, operation: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding.as_mut().ok_or("please init dapr request first")?;

        req.operation = operation.to_string();

        Ok(self)
    }

    pub fn dapr_invoke_binding_data(mut self, data: Vec<u8>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding.as_mut().ok_or("please init dapr request first")?;

        req.data = data;

        Ok(self)
    }

    pub fn dapr_invoke_binding_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_binding.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`get_state`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_get_state_key(mut self, key: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.get_state.as_mut().ok_or("please init dapr request first")?;

        req.key = key.to_string();

        Ok(self)
    }

    pub fn dapr_get_state_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.get_state.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`get_bulk_state`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_get_bulk_state_keys(mut self, mut keys: Vec<&str>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.get_bulk_state.as_mut().ok_or("please init dapr request first")?;

        req.keys = keys.iter_mut().map(|e| e.to_string()).collect();

        Ok(self)
    }

    pub fn dapr_get_bulk_state_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.get_bulk_state.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`query_state`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_query_state_query(mut self, query: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.query_state.as_mut().ok_or("please init dapr request first")?;

        req.query = query.to_string();

        Ok(self)
    }
    pub fn dapr_query_state_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.query_state.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`save_state`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_save_state_extend_state(mut self, key: &str, value: Vec<u8>, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.save_state.as_mut().ok_or("please init dapr request first")?;

        req.states.extend(vec![StateItem {
            key: key.to_string(),
            value,
            etag: None,
            metadata,
            options: None,
        }]);

        Ok(self)
    }
}

// 这里全部是`transaction_state`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_transaction_state_extend_operation(
        mut self,
        operation_type: &str,
        key: Option<&str>,
        value: Option<Vec<u8>>,
    ) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.transaction_state.as_mut().ok_or("please init dapr request first")?;

        req.operations.extend(vec![TransactionalStateOperation {
            operation_type: operation_type.to_string(),
            request: match key {
                None => None,
                Some(k) => match value {
                    None => Some(StateItem {
                        key: k.to_string(),
                        value: vec![],
                        etag: None,
                        metadata: HashMap::new(),
                        options: None,
                    }),
                    Some(v) => Some(StateItem {
                        key: k.to_string(),
                        value: v,
                        etag: None,
                        metadata: HashMap::new(),
                        options: None,
                    }),
                },
            },
        }]);

        Ok(self)
    }

    pub fn dapr_transaction_state_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.transaction_state.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`delete_state`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_delete_state_key(mut self, key: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.delete_state.as_mut().ok_or("please init dapr request first")?;

        req.key = key.to_string();

        Ok(self)
    }

    pub fn dapr_delete_state_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.delete_state.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }

    pub fn dapr_delete_state_etag(mut self, etag: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.delete_state.as_mut().ok_or("please init dapr request first")?;

        req.etag = Some(Etag { value: etag.to_string() });

        Ok(self)
    }
}

// 这里全部是`delete_bulk_state`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_delete_bulk_state_extend_state(mut self, key: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.delete_bulk_state.as_mut().ok_or("please init dapr request first")?;

        req.states.extend(vec![StateItem {
            key: key.to_string(),
            value: vec![],
            etag: None,
            metadata: HashMap::new(),
            options: None,
        }]);

        Ok(self)
    }
}

// 这里全部是`publish_event`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_publish_event_data(mut self, data: Vec<u8>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.publish_event.as_mut().ok_or("please init dapr request first")?;

        req.data = data;

        Ok(self)
    }

    pub fn dapr_publish_event_data_content_type(mut self, data_content_type: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.publish_event.as_mut().ok_or("please init dapr request first")?;

        req.data_content_type = data_content_type.to_string();

        Ok(self)
    }

    pub fn dapr_publish_event_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.publish_event.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`publish_bulk_event`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_publish_bulk_event_extend_entry(mut self, entry_id: &str, event: Vec<u8>, content_type: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.publish_bulk_event.as_mut().ok_or("please init dapr request first")?;

        req.entries.extend(vec![BulkPublishRequestEntry {
            entry_id: entry_id.to_string(),
            event,
            content_type: content_type.to_string(),
            metadata: HashMap::new(),
        }]);

        Ok(self)
    }

    pub fn dapr_publish_bulk_event_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.publish_bulk_event.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`get_secret`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_get_secret_key(mut self, key: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.get_secret.as_mut().ok_or("please init dapr request first")?;

        req.key = key.to_string();

        Ok(self)
    }

    pub fn dapr_get_secret_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.get_secret.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`get_bulk_secret`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_get_bulk_secret_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.get_bluk_secret.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`get_configuration`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_get_configuration_extend_key(mut self, key: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.get_configuration.as_mut().ok_or("please init dapr request first")?;

        req.keys.extend(vec![key.to_string()]);

        Ok(self)
    }

    pub fn dapr_get_configuration_metadata(mut self, metadata: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.get_configuration.as_mut().ok_or("please init dapr request first")?;

        req.metadata.extend(metadata);

        Ok(self)
    }
}

// 这里全部是`invoke_service`相关的方法
impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn dapr_invoke_service_uri(mut self, uri: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_service.as_mut().ok_or("please init dapr request first")?;

        req.message.as_mut().ok_or("invoke service init error")?.method = uri.to_string();

        Ok(self)
    }

    pub fn dapr_invoke_service_content_type(mut self, content_type: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_service.as_mut().ok_or("please init dapr request first")?;

        req.message.as_mut().ok_or("invoke service init error")?.content_type = content_type.to_string();

        Ok(self)
    }

    pub fn dapr_invoke_service_data(mut self, data: Vec<u8>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_service.as_mut().ok_or("please init dapr request first")?;

        req.message.as_mut().ok_or("invoke service init error")?.data = Some(prost_types::Any {
            type_url: "".to_string(),
            value: data,
        });

        Ok(self)
    }

    pub fn dapr_invoke_service_query_string(mut self, query_string: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_service.as_mut().ok_or("please init dapr request first")?;

        req.message
            .as_mut()
            .ok_or("invoke service init error with message not found")?
            .http_extension
            .as_mut()
            .ok_or("invoke service init error with message.http_extension not found")?
            .querystring = query_string.to_string();

        Ok(self)
    }

    pub fn dapr_invoke_service_headers(mut self, headers: HashMap<String, String>) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, _, _) = find_dapr_execute(&mut self.exec, self.exec_name.as_ref().ok_or("please set dapr exec first")?)?;
        let req = req.invoke_service.as_mut().ok_or("please init dapr request first")?;

        req.message.as_mut().ok_or("invoke service init error with message not found")?.headers = headers;

        Ok(self)
    }
}

impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
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

    pub fn dapr_invoke_service(self, exec_name: &str, id: &str, http_method: MethodEnum) -> HttpResult<ContextWrapper<I, O, C>> {
        let mut dapr_req_ins: DaprRequest = Default::default();

        dapr_req_ins._dapr_config = Some(DaprComponentInfo {
            bb_type: DaprBuildBlockType::InvokeService,
            bo_type: DaprOperationType::InvokeService,
            name: String::new(),
            component_type: String::new(),
            namespace: None,
            metadata: None,
            topic: None,
        });

        dapr_req_ins.invoke_service = Some(InvokeServiceRequest {
            id: id.to_string(),
            message: Some(InvokeRequest {
                method: "".to_string(),
                data: None,
                content_type: "".to_string(),
                http_extension: Some(HttpExtension {
                    verb: http_method.to_i32(),
                    querystring: "".to_string(),
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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::GetState;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::GetBulkState;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::QueryState;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::SaveState;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::TransactionState;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::DeleteState;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::DeleteBulkState;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::InvokeBinding;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::InvokeBindingSql;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::PublishEvent;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::PublishBulkEvent;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::GetSecret;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::GetBulkSecret;
        s._dapr_config = Some(bb_type);

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

        let mut bb_type = self.clone();
        bb_type.bo_type = DaprOperationType::GetConfiguration;
        s._dapr_config = Some(bb_type);

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
