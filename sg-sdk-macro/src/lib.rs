use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::Token;

#[proc_macro_attribute]
pub fn biz_result_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut args: BizResultHandlerArgs = match syn::parse(args) {
        Ok(args) => args,
        Err(err) => return input_and_compile_error(input, err),
    };

    let ast = match syn::parse::<syn::ItemStruct>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let prefix = args.biz_code_prefix.base10_digits();
    args.biz_results.iter_mut().for_each(|biz_res| {
        let old_biz_code = biz_res.biz_code.base10_digits();

        let new_biz_code: u32 = format!("{}{:02}", prefix, old_biz_code)
            .parse()
            .expect("error occur when construct new biz_code from biz_code_prefix and biz_code");

        biz_res.biz_code = syn::LitInt::new(new_biz_code.to_string().as_str(), biz_res.biz_code.span());
    });

    let mut tokens: TokenStream = format!("biz_result!({}, {});", ast.ident.to_string(), args.to_string(),)
        .parse()
        .expect("parse biz result handler content to token stream error");

    tokens.extend(input);

    tokens
}

#[proc_macro_attribute]
pub fn uri_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: URIHandlerArgs = match syn::parse(args) {
        Ok(args) => args,
        Err(err) => return input_and_compile_error(input, err),
    };

    let ast = match syn::parse::<syn::ItemStruct>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let mut tokens: TokenStream = format!(
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

fn input_and_compile_error(mut item: TokenStream, err: syn::Error) -> TokenStream {
    let compile_err = TokenStream::from(err.to_compile_error());
    item.extend(compile_err);
    item
}

#[derive(Debug)]
struct BizResultHandlerArgs {
    biz_code_prefix: syn::LitInt,
    biz_results: Vec<BizResultArg>,
}

impl ToString for BizResultHandlerArgs {
    fn to_string(&self) -> String {
        self.biz_results.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(";").to_string()
    }
}

#[derive(Debug)]
struct BizResultArg {
    name: syn::Ident,
    status_code: syn::LitInt,
    biz_code: syn::LitInt,
    message: syn::LitStr,
}

impl ToString for BizResultArg {
    fn to_string(&self) -> String {
        format!("({}, {}, {}, \"{}\")", self.name, self.status_code, self.biz_code, self.message.value())
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
            })?;

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
                name,
                status_code,
                biz_code,
                message,
            })
        }

        Ok(Self { biz_code_prefix, biz_results })
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
