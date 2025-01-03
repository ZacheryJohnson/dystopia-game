use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{Data, PatPath};

#[proc_macro_derive(Abstractable)]
pub fn abstractable_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let enum_ident = ast.ident.to_owned();

    let Data::Enum(mut ast_enum) = ast.data else {
        panic!("#[derive(Abstractable)] is only defined for enums!");
    };
    let abstract_enum_name = syn::Ident::new(&format!("Abstract{}", enum_ident), Span::call_site());
    let mut abstract_to_real_lines = vec![];
    let mut constructor_fns = vec![];

    for variant in &mut ast_enum.variants {
        let field_names = variant
            .fields
            .iter()
            .map(|field| field.ident.to_owned().unwrap())
            .collect::<Vec<_>>();

        let mut default_field_init_lines = vec![];
        for field in &mut variant.fields {
            let field_name = field.ident.to_owned().unwrap();
            let field_type = &field.ty;
            let option_wrapped_field: syn::Type = syn::parse(quote! { Option<#field_type> }.into()).unwrap();
            field.ty = option_wrapped_field;

            default_field_init_lines.push(quote! { #field_name: None });
        }

        let variant_ident = &variant.ident;
        let abstract_binding_line = if field_names.is_empty() {
            quote! {}
        } else {
            quote! {
                { #(#field_names),* }
            }
        };

        let field_construction_line = quote! {
            #(#field_names: #field_names.unwrap_or_default()),*
        };

        abstract_to_real_lines.push(quote! {
            AbstractBelief::#variant_ident #abstract_binding_line => Belief::#variant_ident { #field_construction_line }
        });

        let fn_name_ident = format_ident!("{}", variant_ident);
        constructor_fns.push(if default_field_init_lines.is_empty() {
            quote! {
                pub fn #fn_name_ident() -> #abstract_enum_name {
                    #abstract_enum_name::#variant_ident
                }
            }
        } else {
            quote! {
                pub fn #fn_name_ident() -> #abstract_enum_name {
                    #abstract_enum_name::#variant_ident {#(#default_field_init_lines),*}
                }
            }
        });
    }

    let variants = ast_enum.variants.iter().collect::<Vec<_>>();

    let gen = quote! {
        pub enum #abstract_enum_name {
            #(#variants),*
        }

        impl #abstract_enum_name {
            pub fn to_belief(&self) -> #enum_ident { // ZJ-TODO: change to_belief -> to_<#enum_ident>
                match self {
                    #(#abstract_to_real_lines),*
                }
            }

            pub fn satisfied_by(&self, belief: #enum_ident) -> bool {
                matches!(self.to_belief(), belief)
            }

            #(#constructor_fns)*
        }
    };

    println!("{}", gen.to_string());

    gen.into()
}
