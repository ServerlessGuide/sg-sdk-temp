use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, Meta, Token, Type};

#[proc_macro_derive(ModelValidate)]
pub fn derive_model_validator(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl Validator for #name {
            fn checkout(&self) -> std::result::Result<usize, Box<dyn std::error::Error + Send + Sync>> {
                let checkout = self.validate();
                println!("checkout: {:#?}", checkout);
                if let Err(err) = checkout {
                    println!("err: {}", err.to_string());
                    return Err(Box::new(ResponseError{biz_res: String::from("FIELD_VALIDATE_FAIL"), message: Some(err.to_string())}));
                }
                Ok(0)
            }
        }
    };
    expanded.into()
}

#[proc_macro_derive(Model)]
pub fn derive_model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let data: FieldsNamed = match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(n), .. }) => n,
        _ => {
            return input_and_compile_error(
                name.clone().into_token_stream().into(),
                syn::Error::new(name.span(), "can only be used on struct"),
            )
        }
    };

    let fields = data.named.iter().filter_map(|field| {
        let ty = &field.ty;
        match &field.ident {
            Some(ident) => Some((ident, ty, inner_for_option(ty))),
            _ => None,
        }
    });

    let target_type = vec![
        "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "i128", "u128", "isize", "usize", "bool", "f32", "f64", "String", "char", "Option",
    ];

    let set_field_method = fields.clone().map(|(field_name, ty, option)| match ty {
        Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) if !target_type.contains(&segments[0].ident.to_string().as_str()) => {
            quote! {}
        }

        _ => match option {
            None => quote! {
                stringify!(#field_name) => {
                    let field = self.get_field_mut::<#ty>(field_name);

                    if let None = field {
                        return Err(Box::new(ResponseError{
                            biz_res: String::from("FIELD_NOT_FOUND"),
                            message: Some(String::from(format!("field '{}' not found, type {}", stringify!(#field_name), stringify!(#ty)))),
                        }));
                    }

                    if value == "null".to_string() {
                        return Ok(self);
                    }

                    let parsed = value.parse::<#ty>();
                    if let Err(_) = parsed {
                        return Err(Box::new(ResponseError{
                            biz_res: String::from("VALUE_PARSE_ERROR"),
                            message: Some(String::from(format!(
                                "can not parse value '{}' on {} type", stringify!(#field_name), stringify!(#ty)
                            ))),
                        }));
                    }
                    *field.unwrap() = parsed.unwrap();

                    Ok(self)
                }
            },

            Some(field_ty) => match field_ty {
                Type::Path(syn::TypePath {
                    path: syn::Path { segments, .. },
                    ..
                }) if !target_type.contains(&segments[0].ident.to_string().as_str()) => {
                    quote! {}
                }

                _ => quote! {
                    stringify!(#field_name) => {
                        let field = self.get_field_mut::<Option<#field_ty>>(field_name);

                        if let None = field {
                            return Err(Box::new(ResponseError{
                                biz_res: String::from("FIELD_NOT_FOUND"),
                                message: Some(String::from(format!("field '{}' not found, type {}", stringify!(#field_name), stringify!(#field_ty)))),
                            }));
                        }

                        if value == "null".to_string() {
                            *field.unwrap() = None;
                            return Ok(self);
                        }

                        let parsed = value.parse::<#field_ty>();
                        if let Err(_) = parsed {
                            return Err(Box::new(ResponseError{
                                biz_res: String::from("VALUE_PARSE_ERROR"),
                                message: Some(String::from(format!(
                                    "can not parse value '{}' on {} type", stringify!(#field_name), stringify!(#field_ty)
                                ))),
                            }));
                        }
                        *field.unwrap() = Some(parsed.unwrap());

                        Ok(self)
                    }
                },
            },
        },
    });

    let get_field_str_method = fields.clone().map(|(field_name, ty, option)| match ty {
        Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) if !target_type.contains(&segments[0].ident.to_string().as_str()) => {
            quote! {}
        }

        _ => match option {
            None => quote! {
                stringify!(#field_name) => {
                    return Some(self.#field_name.to_string());
                }
            },

            Some(field_ty) => match field_ty {
                Type::Path(syn::TypePath {
                    path: syn::Path { segments, .. },
                    ..
                }) if !target_type.contains(&segments[0].ident.to_string().as_str()) => {
                    quote! {}
                }

                _ => quote! {
                    stringify!(#field_name) => {
                        let Some(v) = self.#field_name.clone() else {
                            return None;
                        };
                        return Some(v.to_string());
                    }
                },
            },
        },
    });

    let expanded = quote! {
        impl ModelTrait for #name {
            fn clear_model(&self) -> Self {
                Default::default()
            }

            fn new() -> Self {
                Default::default()
            }

            fn clone_model(&self) -> Self {
                self.clone()
            }

            fn set_field(
                &mut self,
                value: String,
                field_name: &str,
            ) -> std::result::Result<&Self, Box<dyn std::error::Error + Send + Sync>> {
                match field_name {
                    #(
                        #set_field_method
                    )*

                    _ => {
                        return Err(Box::new(ResponseError{
                            biz_res: String::from("FIELD_MATCH_NOTHING"),
                            message: None,
                        }));
                    }
                }
            }

            fn get_field_str(&self, field_name: &str) -> Option<String> {
                match field_name {
                    #(
                        #get_field_str_method
                    )*

                    _ => return None,
                }
                None
            }
        }
    };
    expanded.into()
}

fn inner_for_option(ty: &Type) -> Option<Type> {
    match ty {
        Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) if segments[0].ident == "Option" => {
            let segment = &segments[0];

            match &segment.arguments {
                syn::PathArguments::AngleBracketed(generic) => match generic.args.first().unwrap() {
                    syn::GenericArgument::Type(ty) => Some(ty.clone()),
                    _ => None,
                },
                _ => None,
            }
        }

        _ => None,
    }
}

#[proc_macro_derive(EnumGenerate)]
pub fn enum_generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_copy = input.clone();
    let derive_input = parse_macro_input!(input as DeriveInput);

    let enum_name = derive_input.ident.clone();
    let enum_name_str = enum_name.to_string();
    let enum_name_str_lowercase = enum_name.to_string().to_lowercase();

    let variants = match derive_input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => return input_and_compile_error(input_copy, syn::Error::new(derive_input.span(), "can only be used on enum")),
    };

    let mut enum_variants: Vec<(syn::Ident, syn::LitInt)> = vec![];
    for variant in variants.iter().filter(|v| v.discriminant.is_some()) {
        let enum_variant_ident = variant.ident.clone();

        let syn::Expr::Lit(lit) = variant.discriminant.clone().unwrap().1 else {
            continue;
        };

        let syn::Lit::Int(lit_int) = lit.lit else {
            continue;
        };

        enum_variants.push((enum_variant_ident, lit_int));
    }

    let mut from_str: Vec<String> = vec![];

    enum_variants.iter().for_each(|(name, _)| {
        from_str.push(format!("\"{}\" => Ok({}::{}),", name, enum_name, name));
    });

    let from_str_token_part: proc_macro2::TokenStream = match syn::parse_str(&from_str.join("\n")) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input_copy, syn::Error::new(enum_name.span(), "construct token stream error from string")),
    };

    let gen_from_token = quote! {
        impl FromStr for #enum_name {
            type Err = ResponseError;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                match s {
                    #from_str_token_part
                    _ => Err(err(ENUM_NOT_FOUND)),
                }
            }
        }
    };

    let mut to_string: Vec<String> = vec![];

    enum_variants.iter().for_each(|(name, _)| {
        to_string.push(format!("{}::{} => String::from(\"{}\"),", enum_name, name, name));
    });

    let to_string_token_part: proc_macro2::TokenStream = match syn::parse_str(&to_string.join("\n")) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input_copy, syn::Error::new(enum_name.span(), "construct token stream error from string")),
    };

    let to_string_token = quote! {
        impl ToString for #enum_name {
            fn to_string(&self) -> String {
                match self {
                    #to_string_token_part
                }
            }
        }
    };

    let mut lit_val_to_i32: Vec<String> = vec![];

    enum_variants.iter().for_each(|(name, _)| {
        lit_val_to_i32.push(format!("\"{}\" => Some({}::{}.to_i32()),", name, enum_name, name));
    });

    let lit_val_to_i32_token_part: proc_macro2::TokenStream = match syn::parse_str(&lit_val_to_i32.join("\n")) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input_copy, syn::Error::new(enum_name.span(), "construct token stream error from string")),
    };

    let lit_val_to_i32_token = quote! {
        impl #enum_name {
            pub fn lit_val_to_i32(value: &str) -> Option<i32> {
                match value {
                    #lit_val_to_i32_token_part
                    _ => None,
                }
            }
        }
    };

    let mut to_i32: Vec<String> = vec![];

    enum_variants.iter().for_each(|(name, ord)| {
        to_i32.push(format!("{}::{} => {},", enum_name, name, ord));
    });

    let to_i32_token_part: proc_macro2::TokenStream = match syn::parse_str(&to_i32.join("\n")) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input_copy, syn::Error::new(enum_name.span(), "construct token stream error from string")),
    };

    let to_i32_token = quote! {
        impl #enum_name {
            pub fn to_i32(&self) -> i32 {
                match self {
                    #to_i32_token_part
                }
            }
        }
    };

    let enum_option = r#"
mod stringify_enum_{{enum_name_str_lowercase}}_option {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::{{enum_name_str}};

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(value) => {
                let enum_i32 = value.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
                let enum_type = {{enum_name_str}}::from_i32(enum_i32)
                    .ok_or("enum {{enum_name_str}} i32 tag not valid")
                    .map_err(|err| ser::Error::custom(err.to_string()))?;
                serializer.collect_str(&enum_type.to_string())
            }
        }
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match Option::<String>::deserialize(deserializer)? {
            None => Ok(None),
            Some(value) => {
                let enum_i32_string = {{enum_name_str}}::from_str(&value)
                    .map_err(|err| de::Error::custom(err.to_string()))?
                    .to_i32()
                    .to_string();
                match enum_i32_string.parse::<T>() {
                    Ok(t) => Ok(Some(t)),
                    Err(err) => Err(de::Error::custom(err.to_string())),
                }
            }
        }
    }
}
"#;

    let enum_option = enum_option
        .replace("{{enum_name_str}}", &enum_name_str)
        .replace("{{enum_name_str_lowercase}}", &enum_name_str_lowercase);

    let enum_option_token: proc_macro2::TokenStream = match syn::parse_str(&enum_option) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input_copy, syn::Error::new(enum_name.span(), "construct token stream error from string")),
    };

    let enum_prim = r#"
mod stringify_enum_{{enum_name_str_lowercase}}_prim {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::{{enum_name_str}};

    pub fn serialize<T, S>(value: &T, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        let enum_i32 = value.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
        let enum_type = {{enum_name_str}}::from_i32(enum_i32)
            .ok_or("enum {{enum_name_str}} i32 tag not valid")
            .map_err(|err| ser::Error::custom(err.to_string()))?;
        serializer.collect_str(&enum_type.to_string())
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(enum_str) => match {{enum_name_str}}::from_str(&enum_str) {
                Ok(enum_type) => Ok(enum_type.to_i32().to_string().parse::<T>().map_err(|err| de::Error::custom(err.to_string()))?),
                Err(err) => Err(de::Error::custom(err.to_string())),
            },
            Err(err) => Err(err),
        }
    }
}
"#;

    let enum_prim = enum_prim
        .replace("{{enum_name_str}}", &enum_name_str)
        .replace("{{enum_name_str_lowercase}}", &enum_name_str_lowercase);

    let enum_prim_token: proc_macro2::TokenStream = match syn::parse_str(&enum_prim) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input_copy, syn::Error::new(enum_name.span(), "construct token stream error from string")),
    };

    let enum_vec = r#"
mod stringify_enum_{{enum_name_str_lowercase}}_vec {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{
        de::{self},
        ser, Deserialize, Deserializer, Serializer,
    };

    use super::{{enum_name_str}};

    pub fn serialize<T, S>(value: &Vec<T>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        let mut seq = Vec::<String>::new();
        for t in value {
            let enum_i32 = t.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
            let enum_type = {{enum_name_str}}::from_i32(enum_i32)
                .ok_or("enum {{enum_name_str}} i32 tag not valid")
                .map_err(|err| ser::Error::custom(err.to_string()))?;
            seq.push(enum_type.to_string())
        }
        serializer.collect_seq(seq)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> std::result::Result<Vec<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match Vec::<String>::deserialize(deserializer) {
            Ok(enum_strs) => {
                let mut seq = Vec::<T>::new();
                for enum_str in enum_strs {
                    match {{enum_name_str}}::from_str(&enum_str) {
                        Ok(enum_type) => {
                            let act = enum_type.to_i32().to_string().parse::<T>().map_err(|err| de::Error::custom(err.to_string()))?;
                            seq.push(act);
                        }
                        Err(err) => return Err(de::Error::custom(err.to_string())),
                    }
                }
                Ok(seq)
            }
            Err(err) => Err(err),
        }
    }
}
"#;

    let enum_vec = enum_vec
        .replace("{{enum_name_str}}", &enum_name_str)
        .replace("{{enum_name_str_lowercase}}", &enum_name_str_lowercase);

    let enum_vec_token: proc_macro2::TokenStream = match syn::parse_str(&enum_vec) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input_copy, syn::Error::new(enum_name.span(), "construct token stream error from string")),
    };

    let enum_map = r#"
mod stringify_enum_{{enum_name_str_lowercase}}_map {
    use std::hash::Hash;
    use std::str::FromStr;
    use std::{collections::HashMap, fmt::Display};

    use serde::{
        de::{self},
        ser, Deserializer, Serializer,
    };
    use serde::{Deserialize, Serialize};

    use super::{{enum_name_str}};

    pub fn serialize<K, V, S>(value: &HashMap<K, V>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        K: Eq + PartialEq + Hash + Clone + Serialize,
        V: Display,
        S: Serializer,
    {
        let mut map = HashMap::<K, String>::new();
        for (k, v) in value {
            let enum_i32 = v.to_string().parse::<i32>().map_err(|err| ser::Error::custom(err.to_string()))?;
            let enum_type = {{enum_name_str}}::from_i32(enum_i32)
                .ok_or("enum {{enum_name_str}} i32 tag not valid")
                .map_err(|err| ser::Error::custom(err.to_string()))?;
            map.insert(k.clone(), enum_type.to_string());
        }
        serializer.collect_map(map)
    }

    pub fn deserialize<'de, K, V, D>(deserializer: D) -> std::result::Result<HashMap<K, V>, D::Error>
    where
        V: FromStr,
        V::Err: Display,
        D: Deserializer<'de>,
        K: Deserialize<'de> + Eq + Hash,
    {
        match HashMap::<K, String>::deserialize(deserializer) {
            Ok(enum_strs) => {
                let mut map = HashMap::<K, V>::new();
                for (k, v) in enum_strs {
                    match {{enum_name_str}}::from_str(&v) {
                        Ok(enum_type) => {
                            let act = enum_type.to_i32().to_string().parse::<V>().map_err(|err| de::Error::custom(err.to_string()))?;
                            map.insert(k, act);
                        }
                        Err(err) => return Err(de::Error::custom(err.to_string())),
                    }
                }
                Ok(map)
            }
            Err(err) => Err(err),
        }
    }
}
"#;

    let enum_map = enum_map
        .replace("{{enum_name_str}}", &enum_name_str)
        .replace("{{enum_name_str_lowercase}}", &enum_name_str_lowercase);

    let enum_map_token: proc_macro2::TokenStream = match syn::parse_str(&enum_map) {
        Ok(token) => token,
        Err(_) => return input_and_compile_error(input_copy, syn::Error::new(enum_name.span(), "construct token stream error from string")),
    };

    quote! {
        #gen_from_token
        #to_string_token
        #lit_val_to_i32_token
        #to_i32_token
        #enum_option_token
        #enum_prim_token
        #enum_vec_token
        #enum_map_token
    }
    .into()
}

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
                        _ => Err(err_boxed_full_string(
                            ENUM_NOT_FOUND,
                            format!("enum field {} not found", f_name),
                        )),
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
