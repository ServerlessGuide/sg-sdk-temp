use serde::Deserialize;

use crate::{inner_biz_result::*, model::*, traits::*, util::*, HttpResult};

impl<I: ModelTrait + prost::Message + Default, O: ModelTrait + prost::Message, C: Clone> ContextWrapper<I, O, C> {
    pub fn get_dapr_resp_list<T: DaprBody + Clone>(mut self, exec_name: &str) -> HttpResult<(ContextWrapper<I, O, C>, Vec<T>)> {
        let (_, _, res_wrapper) = find_dapr_execute(&mut self.exec, exec_name)?;

        match res_wrapper {
            None => return Err(err_boxed(DATA_NOT_FOUND)),
            Some(rw) => {
                let mut res_w = Vec::<T>::new();
                for e in rw.iter_mut() {
                    res_w.push(e.downcast_mut::<T>().ok_or("downcast error")?.clone());
                }
                Ok((self, res_w))
            }
        }
    }

    pub fn get_dapr_resp_one<T: DaprBody + Clone>(mut self, exec_name: &str) -> HttpResult<(ContextWrapper<I, O, C>, T)> {
        let (_, _, res_wrapper) = find_dapr_execute(&mut self.exec, exec_name)?;

        match res_wrapper {
            None => return Err(err_boxed(DATA_NOT_FOUND)),
            Some(rw) => {
                if rw.len() != 1 {
                    return Err(err_boxed_full(DATA_ERROR, "dapr response len not 1"));
                }
                let t = rw.first_mut().unwrap().downcast_mut::<T>().ok_or("downcast error")?.clone();
                Ok((self, t))
            }
        }
    }

    pub fn decode_sql_list<T: ModelTrait + DaprBody + Default + EnumConvert>(mut self, exec_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, res, _) = find_dapr_execute(&mut self.exec, exec_name)?;

        let Some(dapr_comp) = req._dapr_config.as_ref() else {
            return Err(err_boxed(DAPR_COMPONENT_NOT_EXIST));
        };

        match dapr_comp.bb_type {
            DaprBuildBlockType::Binding => {
                let response = res
                    .invoke_binding_sql
                    .as_ref()
                    .ok_or(format!("execute '{}' of invoke_binding_sql response not found", exec_name))?;
                match response.responses.first() {
                    Some(first) => {
                        let ts = de_sql_result_implicit::<T>(&first.data, &first.output_columns, T::enum_convert)?;
                        let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                        for t in ts {
                            dapr_res.push(Box::new(t));
                        }
                        Ok(set_dapr_res(self, dapr_res, exec_name)?)
                    }
                    None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sql data result not found")),
                }
            }
            _ => {
                return Err(err_boxed_full(
                    DAPR_DATA_ILLEGAL,
                    "can not decode sql result from any other dapr build block than binding",
                ));
            }
        }
    }

    pub fn decode_sql_one<T: ModelTrait + DaprBody + Default + EnumConvert>(mut self, exec_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, res, _) = find_dapr_execute(&mut self.exec, exec_name)?;

        let Some(dapr_comp) = req._dapr_config.as_ref() else {
            return Err(err_boxed(DAPR_COMPONENT_NOT_EXIST));
        };

        match dapr_comp.bb_type {
            DaprBuildBlockType::Binding => {
                let response = res
                    .invoke_binding_sql
                    .as_ref()
                    .ok_or(format!("execute '{}' of invoke_binding_sql response not found", exec_name))?;
                match response.responses.first() {
                    Some(first) => {
                        let t = de_sql_result_implicit_first::<T>(&first.data, &first.output_columns, T::enum_convert)?;
                        let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                        dapr_res.push(Box::new(t));
                        Ok(set_dapr_res(self, dapr_res, exec_name)?)
                    }
                    None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sql data result not found")),
                }
            }
            _ => {
                return Err(err_boxed_full(
                    DAPR_DATA_ILLEGAL,
                    "can not decode sql result from any other dapr build block than binding",
                ));
            }
        }
    }

    pub fn decode_json_list<T: for<'de> Deserialize<'de> + DaprBody>(mut self, exec_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, res, _) = find_dapr_execute(&mut self.exec, exec_name)?;

        let Some(dapr_comp) = req._dapr_config.as_ref() else {
            return Err(err_boxed(DAPR_COMPONENT_NOT_EXIST));
        };

        let dapr_res: Vec<Box<dyn DaprBody>> = match dapr_comp.bb_type {
            DaprBuildBlockType::Binding => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr Binding can not be None")),
                DaprOperationType::InvokeBinding => {
                    let resp = res
                        .invoke_binding
                        .as_ref()
                        .ok_or("sub operation type InvokeBinding of dapr Binding not found")?;

                    let ts = serde_json::from_slice::<Vec<T>>(&resp.data)?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    for t in ts {
                        dapr_res.push(Box::new(t));
                    }
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request Binding can only be InvokeBinding when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::State => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr State can not be None")),
                DaprOperationType::GetState => {
                    let resp = res.get_state.as_ref().ok_or("sub operation type GetState of dapr State not found")?;
                    let ts = serde_json::from_slice::<Vec<T>>(&resp.data)?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    for t in ts {
                        dapr_res.push(Box::new(t));
                    }
                    dapr_res
                }
                DaprOperationType::GetBulkState => {
                    let resp = res.get_bulk_state.as_ref().ok_or("sub operation type GetBulkState of dapr State not found")?;
                    let mut ts = Vec::<Box<dyn DaprBody>>::new();
                    for ele in resp.items.iter() {
                        ts.push(Box::new(serde_json::from_slice::<T>(&ele.data)?));
                    }
                    ts
                }
                DaprOperationType::QueryState => {
                    let resp = res.query_state.as_ref().ok_or("sub operation type QueryState of dapr State not found")?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    for ele in resp.results.iter() {
                        dapr_res.push(Box::new(serde_json::from_slice::<T>(&ele.data)?));
                    }
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request State can only be GetState | GetBulkState | QueryState when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::Pubsub => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "dapr Pubsub have no response body")),
            DaprBuildBlockType::Secret => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr Secret can not be None")),
                DaprOperationType::GetSecret => {
                    let resp = res.get_secret.as_ref().ok_or("sub operation type GetSecret of dapr Secret not found")?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    dapr_res.push(Box::new(resp.clone()));
                    dapr_res
                }
                DaprOperationType::GetBulkSecret => {
                    let resp = res
                        .get_bluk_secret
                        .as_ref()
                        .ok_or("sub operation type GetBulkSecret of dapr Secret not found")?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    dapr_res.push(Box::new(resp.clone()));
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request Secret can only be GetSecret | GetBulkSecret when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::Conf => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr Conf can not be None")),
                DaprOperationType::GetConfiguration => {
                    let resp = res
                        .get_configuration
                        .as_ref()
                        .ok_or("sub operation type GetConfiguration of dapr Conf not found")?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    dapr_res.push(Box::new(resp.clone()));
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request Conf can only be GetConfiguration when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::InvokeService => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr InvokeService can not be None")),
                DaprOperationType::InvokeService => {
                    let resp = res
                        .invoke_service
                        .as_ref()
                        .ok_or("sub operation type InvokeService of dapr InvokeService not found")?;
                    let ts = serde_json::from_slice::<Vec<T>>(&resp.data.as_ref().ok_or("response data of invoke service not found")?.value)?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    for t in ts {
                        dapr_res.push(Box::new(t));
                    }
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request InvokeService can only be InvokeService when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "build block type of dapr request not found")),
        };

        Ok(set_dapr_res(self, dapr_res, exec_name)?)
    }

    pub fn decode_json_one<T: for<'de> Deserialize<'de> + DaprBody>(mut self, exec_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
        let (req, res, _) = find_dapr_execute(&mut self.exec, exec_name)?;

        let Some(dapr_comp) = req._dapr_config.as_ref() else {
            return Err(err_boxed(DAPR_COMPONENT_NOT_EXIST));
        };

        let dapr_res: Vec<Box<dyn DaprBody>> = match dapr_comp.bb_type {
            DaprBuildBlockType::Binding => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr Binding can not be None")),
                DaprOperationType::InvokeBinding => {
                    let resp = res
                        .invoke_binding
                        .as_ref()
                        .ok_or("sub operation type InvokeBinding of dapr Binding not found")?;

                    let t = serde_json::from_slice::<T>(&resp.data)?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    dapr_res.push(Box::new(t));
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request Binding can only be InvokeBinding when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::State => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr State can not be None")),
                DaprOperationType::GetState => {
                    let resp = res.get_state.as_ref().ok_or("sub operation type GetState of dapr State not found")?;
                    let t = serde_json::from_slice::<T>(&resp.data)?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    dapr_res.push(Box::new(t));
                    dapr_res
                }
                DaprOperationType::GetBulkState => {
                    let resp = res.get_bulk_state.as_ref().ok_or("sub operation type GetBulkState of dapr State not found")?;
                    let mut ts = Vec::<Box<dyn DaprBody>>::new();
                    let t_o = resp.items.first().ok_or("the first response of GetBulkState not found")?;
                    ts.push(Box::new(serde_json::from_slice::<T>(&t_o.data)?));
                    ts
                }
                DaprOperationType::QueryState => {
                    let resp = res.query_state.as_ref().ok_or("sub operation type QueryState of dapr State not found")?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    let t_o = resp.results.first().ok_or("the first response of QueryState not found")?;
                    dapr_res.push(Box::new(serde_json::from_slice::<T>(&t_o.data)?));
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request State can only be GetState | GetBulkState | QueryState when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::Pubsub => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "dapr Pubsub have no response body")),
            DaprBuildBlockType::Secret => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr Secret can not be None")),
                DaprOperationType::GetSecret => {
                    let resp = res.get_secret.as_ref().ok_or("sub operation type GetSecret of dapr Secret not found")?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    dapr_res.push(Box::new(resp.clone()));
                    dapr_res
                }
                DaprOperationType::GetBulkSecret => {
                    let resp = res
                        .get_bluk_secret
                        .as_ref()
                        .ok_or("sub operation type GetBulkSecret of dapr Secret not found")?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    dapr_res.push(Box::new(resp.clone()));
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request Secret can only be GetSecret | GetBulkSecret when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::Conf => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr Conf can not be None")),
                DaprOperationType::GetConfiguration => {
                    let resp = res
                        .get_configuration
                        .as_ref()
                        .ok_or("sub operation type GetConfiguration of dapr Conf not found")?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    dapr_res.push(Box::new(resp.clone()));
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request Conf can only be GetConfiguration when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::InvokeService => match dapr_comp.bo_type {
                DaprOperationType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "sub operation type of dapr InvokeService can not be None")),
                DaprOperationType::InvokeService => {
                    let resp = res
                        .invoke_service
                        .as_ref()
                        .ok_or("sub operation type InvokeService of dapr InvokeService not found")?;
                    let ts = serde_json::from_slice::<Vec<T>>(&resp.data.as_ref().ok_or("response data of invoke service not found")?.value)?;
                    let mut dapr_res = Vec::<Box<dyn DaprBody>>::new();
                    for t in ts {
                        dapr_res.push(Box::new(t));
                    }
                    dapr_res
                }
                _ => {
                    return Err(err_boxed_full(
                        DAPR_DATA_ILLEGAL,
                        "sub operation type of dapr request InvokeService can only be InvokeService when decode with json",
                    ))
                }
            },
            DaprBuildBlockType::None => return Err(err_boxed_full(DAPR_DATA_ILLEGAL, "build block type of dapr request not found")),
        };

        Ok(set_dapr_res(self, dapr_res, exec_name)?)
    }

    // pub fn decode_prost_list<T>(mut self, exec_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
    //     Ok(self)
    // }

    // pub fn decode_prost_one<T>(mut self, exec_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
    //     Ok(self)
    // }

    // pub fn decode_prost_json_list<T>(mut self, exec_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
    //     Ok(self)
    // }

    // pub fn decode_prost_json_one<T>(mut self, exec_name: &str) -> HttpResult<ContextWrapper<I, O, C>> {
    //     Ok(self)
    // }
}
