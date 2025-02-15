use proc_macro::TokenStream;
use std::collections::HashMap;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::Data;

pub fn unique_impl(input: TokenStream) -> TokenStream {
    let parsed: syn::DeriveInput = syn::parse(input).unwrap();

    let Data::Enum(ast) = parsed.data else {
        panic!("Only enums are supported");
    };

    let enum_ident = &parsed.ident;

    let mut variant_to_unique_idents: HashMap<Ident, Vec<proc_macro2::TokenStream>> = HashMap::new();
    let mut variant_to_hasher_token_streams: HashMap<Ident, Vec<proc_macro2::TokenStream>> = HashMap::new();

    for variant in &ast.variants {
        let variant_ident = &variant.ident;
        variant_to_unique_idents.insert(variant_ident.to_owned(), vec![]);
        variant_to_hasher_token_streams.insert(variant_ident.to_owned(), vec![]);

        for field in &variant.fields {
            let mut has_unique_attr = false;
            for attr in &field.attrs {
                if let Some(segment) = attr.meta.path().segments.first() {
                    if segment.ident == "unique" {
                        has_unique_attr = true;
                    }
                }
            }

            if has_unique_attr {
                let ident = field.ident.as_ref().unwrap();
                variant_to_unique_idents.get_mut(variant_ident).unwrap().push(field.ident.to_owned().unwrap().into_token_stream());
                variant_to_hasher_token_streams.get_mut(variant_ident).unwrap().push(quote! {
                    hasher.write_u64(#ident.to_owned() as u64);
                });
            }
        }
    }

    let mut variant_mapping: Vec<proc_macro2::TokenStream> = vec![];
    for (variant, idents) in variant_to_unique_idents {
        let hasher_tokens = variant_to_hasher_token_streams.get(&variant).unwrap();
        variant_mapping.push(quote! {
            #enum_ident::#variant { #(#idents,)* .. } => {
                let mut hasher = AHasher::default();
                hasher.write(stringify!(#variant).as_bytes());
                #(#hasher_tokens)*
                hasher.finish()
            }
        });
    }

    let gen = quote!{
        use std::hash::Hasher;
        use ahash::AHasher;

        impl Uniqueness for #enum_ident {
            fn unique_key(&self) -> u64 {
                match self {
                    #(#variant_mapping)*,
                    _ => 0,
                }
            }
        }
    };

    gen.into()
}