use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields, FieldsNamed, Meta, Token};

#[proc_macro_derive(EnumFieldsConvert)]
pub fn enum_convert_for_sql(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_copy = input.clone();
    let derive_input = parse_macro_input!(input_copy as DeriveInput);

    let struct_name = derive_input.ident.clone();

    let fields: FieldsNamed = match derive_input.data {
        Data::Struct(DataStruct { fields: Fields::Named(n), .. }) => n,
        _ => return input_and_compile_error(input, syn::Error::new(derive_input.span(), "can only be used on struct")),
    };

    let mut enum_fields = Vec::<(syn::Ident, String)>::new();

    for field in fields.named {
        let Some(field_name) = field.ident else {
            continue;
        };

        for attr in field.attrs {
            if !attr.path().is_ident("prost") {
                continue;
            }

            match attr.meta {
                Meta::List(mnv) => match mnv.tokens.clone().into_iter().nth(0) {
                    Some(first) => {
                        let Ok(ident) = syn::parse::<syn::Ident>(first.into_token_stream().into()) else {
                            continue;
                        };

                        if ident.to_string() == String::from("enumeration") {
                            let Some(enum_name) = mnv.tokens.into_iter().nth(2) else {
                                continue;
                            };

                            let Ok(lit) = syn::parse::<syn::LitStr>(enum_name.into_token_stream().into()) else {
                                continue;
                            };

                            enum_fields.push((field_name.clone(), lit.value()));
                        }
                    }
                    None => {
                        continue;
                    }
                },
                _ => {
                    continue;
                }
            }
        }
    }

    let mut enum_field_names = Vec::<String>::new();
    let mut pattern_branches = Vec::<String>::new();
    enum_fields.iter().for_each(|(field_name, enum_name)| {
        enum_field_names.push(format!("\"{}\"", field_name));
        pattern_branches.push(format!("\"{}\" => Ok((true, {}::lit_val_to_i32(f_value))),", field_name, enum_name));
    });

    let enum_field_names_str = enum_field_names.join(",");
    let pattern_branches_str = pattern_branches.join("\n");

    let enum_field_names_tokens: proc_macro2::TokenStream = match syn::parse_str(&enum_field_names_str) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input, syn::Error::new(struct_name.span(), "construct token stream error from string")),
    };

    let pattern_branches_tokens: proc_macro2::TokenStream = match syn::parse_str(&pattern_branches_str) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input, syn::Error::new(struct_name.span(), "construct token stream error from string")),
    };

    let gen = quote! {
        impl #struct_name {
            pub fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)> {
                let enum_flds = [#enum_field_names_tokens].to_vec();
                if enum_flds.contains(&f_name) {
                    match f_name {
                        #pattern_branches_tokens
                        _ => Err(Box::new(gen_resp_err(
                            ENUM_NOT_FOUND,
                            Some(format!("enum field {} not found", f_name)),
                        ))),
                    }
                } else {
                    Ok((false, None))
                }
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(Dapr)]
pub fn dapr_body(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = match syn::parse::<syn::ItemStruct>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let name = &ast.ident;
    let gen = quote! {
        impl DaprBody for #name {}
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn biz_result_handler(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut args: BizResultHandlerArgs = match syn::parse(args) {
        Ok(args) => args,
        Err(err) => return input_and_compile_error(input, err),
    };

    let ast = match syn::parse::<syn::ItemStruct>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let mut biz_res_needed = Vec::<BizResultArg>::new();
    biz_res_needed.push(BizResultArg::new("OK", 200, 00, "success"));
    biz_res_needed.push(BizResultArg::new("URI_NOT_MATCH", 404, 01, "uri match nothing"));
    biz_res_needed.push(BizResultArg::new("BODY_PARAMETER_ILLEGAL", 400, 02, "body parameter illegal"));
    biz_res_needed.push(BizResultArg::new("CONVERT_TO_MODEL_ERROR", 500, 03, "convert to model error"));
    biz_res_needed.push(BizResultArg::new("PARAMETER_ILLEGAL", 400, 04, "parameter illegal"));
    biz_res_needed.push(BizResultArg::new("HEADER_NOT_FOUND", 400, 05, "header not found"));
    biz_res_needed.push(BizResultArg::new("PARAM_MAP_PARSE_ERROR", 500, 06, "param map parse error"));
    biz_res_needed.push(BizResultArg::new("PATH_PARAM_NOT_EXIST", 500, 07, "path param not exist"));
    biz_res_needed.push(BizResultArg::new("BODY_PARAM_NOT_EXIST", 500, 08, "body param not exist"));
    biz_res_needed.push(BizResultArg::new("QUERY_PARAM_NOT_EXIST", 500, 09, "query param not exist"));
    biz_res_needed.push(BizResultArg::new("URL_PARSE_ERROR", 500, 10, "url parse error"));
    biz_res_needed.push(BizResultArg::new("DAPR_HTTP_REQ_BUILD_ERROR", 500, 11, "dapr request build error"));
    biz_res_needed.push(BizResultArg::new("DAPR_REQUEST_FAIL", 500, 12, "dapr request fail"));
    biz_res_needed.push(BizResultArg::new("REQUEST_METHOD_NOT_ALLOWED", 500, 13, "request method not allowed"));
    biz_res_needed.push(BizResultArg::new("ENV_PARAMETER_ERROR", 500, 14, "env parameter error"));
    biz_res_needed.push(BizResultArg::new("DAPR_DATA_ILLEGAL", 500, 15, "dapr data illegal"));
    biz_res_needed.push(BizResultArg::new("ENUM_NOT_FOUND", 500, 16, "enum not found"));
    biz_res_needed.push(BizResultArg::new("IMPLICIT_RESPONSE_ERROR", 500, 17, "implicit response error"));
    biz_res_needed.push(BizResultArg::new("BIZ_RESULT_NOT_FOUND", 500, 18, "biz result not found"));
    biz_res_needed.push(BizResultArg::new("DAPR_CONFIG_NOT_EXIST", 500, 19, "dapr config not exist"));
    biz_res_needed.push(BizResultArg::new("EXEC_NAME_NOT_EXIST", 500, 20, "execute name not exist"));
    biz_res_needed.push(BizResultArg::new("DAPR_EXECUTE_NOT_EXIST", 500, 21, "dapr execute not exist"));
    biz_res_needed.push(BizResultArg::new("QUERY_SQL_IS_NOT_UNIQUE", 500, 22, "query sql is not unique"));
    biz_res_needed.push(BizResultArg::new("SQL_NOT_VALID", 500, 23, "sql not valid"));
    biz_res_needed.push(BizResultArg::new("SQL_NOT_SUPPORT", 500, 24, "sql not support"));
    biz_res_needed.push(BizResultArg::new("DATA_NOT_FOUND", 400, 25, "data not found"));
    biz_res_needed.push(BizResultArg::new("SQL_OUT_COLUMNS_IS_EMPTY", 500, 26, "sql out_columns is empty"));
    biz_res_needed.push(BizResultArg::new("DATA_ERROR", 500, 27, "data error"));
    biz_res_needed.push(BizResultArg::new("AUTH_ERROR", 401, 28, "auth error"));
    biz_res_needed.push(BizResultArg::new("INTERNAL_AUTH_TAG_NOT_SET", 500, 29, "internal auth tag not set"));

    args.biz_results.extend(biz_res_needed);

    args.biz_results.iter_mut().for_each(|biz_res| {
        let new_biz_code: u32 = format!("{}{:02}", args.biz_code_prefix, biz_res.biz_code)
            .parse()
            .expect("error occur when construct new biz_code from biz_code_prefix and biz_code");

        biz_res.biz_code = new_biz_code;
    });

    let mut tokens: proc_macro::TokenStream = format!("biz_result!({}, {});", ast.ident.to_string(), args.to_string(),)
        .parse()
        .expect("parse biz result handler content to token stream error");

    tokens.extend(input);

    tokens
}

#[proc_macro_attribute]
pub fn uri_handler(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args: URIHandlerArgs = match syn::parse(args) {
        Ok(args) => args,
        Err(err) => return input_and_compile_error(input, err),
    };

    let ast = match syn::parse::<syn::ItemStruct>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let mut tokens: proc_macro::TokenStream = format!(
        "generate_http_dispatcher!({}, [{}]);\ngenerate_grpc_dispatcher!({}, [{}]);\n",
        ast.ident.to_string(),
        args.to_string(),
        ast.ident.to_string(),
        args.to_string()
    )
    .parse()
    .expect("parse uri handler content to token stream error");

    tokens.extend(input);

    tokens
}

fn input_and_compile_error(mut item: proc_macro::TokenStream, err: syn::Error) -> proc_macro::TokenStream {
    let compile_err = proc_macro::TokenStream::from(err.to_compile_error());
    item.extend(compile_err);
    item
}

#[derive(Debug)]
struct BizResultHandlerArgs {
    biz_code_prefix: u16,
    biz_results: Vec<BizResultArg>,
}

impl ToString for BizResultHandlerArgs {
    fn to_string(&self) -> String {
        self.biz_results.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(";").to_string()
    }
}

#[derive(Debug)]
struct BizResultArg {
    name: String,
    status_code: u16,
    biz_code: u32,
    message: String,
}

impl BizResultArg {
    fn new(name: &str, status_code: u16, biz_code: u32, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status_code,
            biz_code,
            message: message.to_string(),
        }
    }
}

impl ToString for BizResultArg {
    fn to_string(&self) -> String {
        format!("({}, {}, {}, \"{}\")", self.name, self.status_code, self.biz_code, self.message)
    }
}

impl syn::parse::Parse for BizResultHandlerArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let biz_code_prefix = input.parse::<syn::LitInt>().map_err(|mut err| {
            err.combine(syn::Error::new(
                err.span(),
                r#"invalid biz_code_prefix definition, expected #[("biz_result_handler("<biz_code_prefix>, <<name>,<status_code>,<biz_code>, <message>>;...")")]"#,
            ));

            err
        })?;

        input.parse::<Token![,]>()?;

        let mut biz_results = Vec::<BizResultArg>::new();
        let mut begin = true;

        while input.peek(Token![;]) || begin {
            if !begin {
                input.parse::<Token![;]>()?;
            }

            begin = false;

            input.parse::<Token![<]>()?;

            let name = input.parse::<syn::Ident>().map_err(|mut err| {
                err.combine(syn::Error::new(
                    err.span(),
                    r#"invalid biz result name definition, expected #[("biz_result_handler("<biz_code_prefix>, <<name>,<status_code>,<biz_code>, <message>>;...")")]"#,
                ));

                err
            })?.to_string();

            input.parse::<Token![,]>()?;

            let status_code = input.parse::<syn::LitInt>().map_err(|mut err| {
                err.combine(syn::Error::new(
                    err.span(),
                    r#"invalid biz result status_code definition, expected #[("biz_result_handler("<biz_code_prefix>, <<name>,<status_code>,<biz_code>, <message>>;...")")]"#,
                ));

                err
            })?;

            input.parse::<Token![,]>()?;

            let biz_code = input.parse::<syn::LitInt>().map_err(|mut err| {
                err.combine(syn::Error::new(
                    err.span(),
                    r#"invalid biz result biz_code definition, expected #[("biz_result_handler("<biz_code_prefix>, <<name>,<status_code>,<biz_code>, <message>>;...")")]"#,
                ));

                err
            })?;

            input.parse::<Token![,]>()?;

            let message = input.parse::<syn::LitStr>().map_err(|mut err| {
                err.combine(syn::Error::new(
                    err.span(),
                    r#"invalid biz result message definition, expected #[("biz_result_handler("<biz_code_prefix>, <<name>,<status_code>,<biz_code>, <message>>;...")")]"#,
                ));

                err
            })?;

            input.parse::<Token![>]>()?;

            biz_results.push(BizResultArg {
                name: name,
                status_code: status_code.base10_digits().parse().map_err(|e| syn::Error::new(input.span(), e))?,
                biz_code: biz_code.base10_digits().parse().map_err(|e| syn::Error::new(input.span(), e))?,
                message: message.value(),
            })
        }

        Ok(Self {
            biz_code_prefix: biz_code_prefix.base10_digits().parse().map_err(|e| syn::Error::new(input.span(), e))?,
            biz_results,
        })
    }
}

#[derive(Debug)]
struct URIHandlerArgs {
    handlers: Vec<URIHandler>,
}

impl ToString for URIHandlerArgs {
    fn to_string(&self) -> String {
        self.handlers.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(",").to_string()
    }
}

#[derive(Debug)]
struct URIHandler {
    uri: syn::Ident,
    fn_name: syn::Ident,
}

impl ToString for URIHandler {
    fn to_string(&self) -> String {
        format!("({}, {})", self.uri, self.fn_name)
    }
}

impl syn::parse::Parse for URIHandlerArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let mut handlers = Vec::<URIHandler>::new();
        let mut begin = true;

        while input.peek(Token![,]) || begin {
            if !begin {
                input.parse::<Token![,]>()?;
            }

            begin = false;

            let uri = input.parse::<syn::Ident>().map_err(|mut err| {
                err.combine(syn::Error::new(
                    err.span(),
                    r#"invalid uri definition, expected #[("uri_handler("<uri>, <fn_name>")")]"#,
                ));

                err
            })?;

            if !input.peek(Token![=>]) {
                return Err(syn::Error::new(input.span(), "have not the fn_name"));
            }

            input.parse::<Token![=>]>()?;

            let fn_name = input.parse::<syn::Ident>().map_err(|mut err| {
                err.combine(syn::Error::new(
                    err.span(),
                    r#"invalid uri definition, expected #[("uri_handler("<uri>, <fn_name>")")]"#,
                ));

                err
            })?;

            handlers.push(URIHandler { uri, fn_name })
        }

        Ok(Self { handlers })
    }
}
