use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn uri_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    // route::with_method(Some(route::MethodType::$variant), args, input)
    input
}
