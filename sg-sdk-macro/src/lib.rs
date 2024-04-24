use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Token;

#[proc_macro_attribute]
pub fn uri_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: URIHandlerArgs = match syn::parse(args) {
        Ok(args) => args,
        Err(err) => return input_and_compile_error(input, err),
    };

    let ast = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let mut tokens: TokenStream = format!(
        "register_uri_handler!({}, {}, {});\n",
        args.uri.value(),
        ast.sig.ident.to_string(),
        args.acceptor.to_string()
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
struct URIHandlerArgs {
    uri: syn::LitStr,
    acceptor: syn::Ident,
}

impl syn::parse::Parse for URIHandlerArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let uri = input.parse::<syn::LitStr>().map_err(|mut err| {
            err.combine(syn::Error::new(
                err.span(),
                r#"invalid uri definition, expected #[("uri_handler("<uri>, <acceptor>")")]"#,
            ));

            err
        })?;

        if !input.peek(Token![,]) {
            return Err(syn::Error::new(input.span(), "have not the acceptor"));
        }

        input.parse::<Token![,]>()?;

        let acceptor = input.parse::<syn::Ident>().map_err(|mut err| {
            err.combine(syn::Error::new(
                err.span(),
                r#"invalid uri definition, expected #[("uri_handler("<uri>, <acceptor>")")]"#,
            ));

            err
        })?;

        Ok(Self { uri, acceptor })
    }
}
