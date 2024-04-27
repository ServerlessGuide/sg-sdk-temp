use crate::{
    model::{DaprComponentInfo, SqlBuilder, SqlOperation, SqlWithParams, SqlsBuilder},
    util::trans_sql_info,
    HttpResult,
};

impl<'a> SqlsBuilder<'a> {
    pub fn new() -> Self {
        Self {
            sql_builders: vec![],
            operation: SqlOperation::Query,
            dapr_component: None,
        }
    }

    pub fn sql_builder(mut self, sql_builder: SqlBuilder) -> Self {
        self.sql_builders.push(sql_builder);
        self
    }

    pub fn operation(mut self, operation: SqlOperation) -> Self {
        self.operation = operation;
        self
    }

    pub fn dapr_component(mut self, dapr_component: &'a DaprComponentInfo) -> Self {
        self.dapr_component = Some(dapr_component);
        self
    }

    pub fn build(self) -> HttpResult<Vec<SqlWithParams>> {
        let mut sql_tuples = Vec::<(String, Vec<rbs::Value>, bool, Option<u64>, Option<u64>)>::new();
        for sql_builder in self.sql_builders {
            sql_tuples.push((
                sql_builder.sql.ok_or("sql not found")?,
                sql_builder.params,
                sql_builder.page,
                sql_builder.offset,
                sql_builder.page_size,
            ));
        }

        Ok(trans_sql_info(
            sql_tuples,
            self.operation,
            self.dapr_component.ok_or("dapr component not found")?,
        )?)
    }
}

impl SqlBuilder {
    pub fn new() -> Self {
        Self {
            sql: None,
            params: vec![],
            output_columns: vec![],
            page: false,
            offset: None,
            page_size: None,
        }
    }

    pub fn sql(mut self, sql: &str) -> Self {
        self.sql = Some(sql.to_string());
        self
    }

    pub fn params(mut self, params: Vec<rbs::Value>) -> Self {
        self.params = params;
        self
    }

    pub fn param_extend(mut self, param: rbs::Value) -> Self {
        self.params.push(param);
        self
    }

    pub fn output_columns(mut self, output_columns: Vec<&str>) -> Self {
        self.output_columns = output_columns.iter().map(|e| e.to_string()).collect();
        self
    }

    pub fn output_columns_extend(mut self, output_column: &str) -> Self {
        self.output_columns.push(output_column.to_string());
        self
    }

    pub fn page(mut self, is_page: bool) -> Self {
        self.page = is_page;
        self
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn page_size(mut self, page_size: u64) -> Self {
        self.page_size = Some(page_size);
        self
    }
}
