use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse::discouraged::AnyDelimiter, Token};

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

    let mut tokens: TokenStream = format!("register_uri_handler!({}, [{}]);\n", ast.ident.to_string(), args.to_string())
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
