use quote::{format_ident, quote, ToTokens};
use syn::{AttrStyle, Data};

mod http;
mod nats;
mod api;

#[proc_macro_attribute]
pub fn api(attribute: proc_macro::TokenStream, api: proc_macro::TokenStream) -> proc_macro::TokenStream {
    api::api_impl(attribute, api)
}

#[proc_macro_derive(ApiRequest)]
pub fn api_request(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let Data::Struct(request_struct) = ast.data else {
        panic!("only structs are supported");
    };

    // let mut request_parameters = vec![];
    for field in request_struct.fields {
        let mut type_str = field.ty.to_token_stream().to_string();
        let mut comment = String::new();
        for attr in field.attrs {
            if let AttrStyle::Outer = attr.style && attr.meta.require_name_value().is_ok() {
                comment += &(attr.meta.require_name_value().unwrap().value.to_token_stream().to_string() + "\n");
            }
        }

        // ZJ-TODO: this is hacky
        let is_optional = type_str.contains("Option <");
        if is_optional {
            let option_idx = type_str.find("Option <").unwrap();
            let end_chevron_idx = type_str[option_idx..].find(">").unwrap();
            type_str = type_str[(option_idx + 8)..end_chevron_idx].to_string();
        }

        // let param_name = field.ident.to_token_stream().to_string();
        // let required = if is_optional { "utoipa::openapi::Required::False" } else { "utoipa::openapi::Required::True" };
        // let param_in = if is_optional { "utoipa::openapi::path::ParameterIn::Query" } else { "utoipa::openapi::path::ParameterIn::Path" };
        // let schema_type = match type_str.as_str() {
        //     "u32" | "u64" => "utoipa::openapi::schema::Type::Integer",
        //     "f32" | "f64" => "utoipa::openapi::schema::Type::Number",
        //     "String" | "&str" => "utoipa::openapi::schema::Type::String",
        //     "bool" => "utoipa::openapi::schema::Type::Boolean",
        //     _ => panic!("unsupported type in ApiRequest macro: {}", type_str),
        // };
        //
        // request_parameters.push(quote! {
        //     utoipa::openapi::path::ParameterBuilder::new()
        //         .name(#param_name)
        //         .description(Some(#comment))
        //         .required(#required)
        //         .parameter_in(#param_in)
        //         .schema(Some(
        //             utoipa::openapi::ObjectBuilder::new()
        //                 .schema_type(#schema_type)
        //         ))
        //         .build()
        // });
    }

    let struct_name = format_ident!("{}", ast.ident);

    quote! {
        impl IntoParams for #struct_name {
            fn into_params(_: impl Fn() -> Option<ParameterIn>) -> Vec<Parameter> {
                vec![
                    // #(#request_parameters),*
                ]
            }
        }
    }.into()
}