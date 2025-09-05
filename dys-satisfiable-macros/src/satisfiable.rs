use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::Data;

pub fn satisfiable_impl(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let Data::Enum(mut ast_enum) = ast.data else {
        panic!("#[derive(Satisfiable)] is only defined for enums!");
    };

    let concrete_ident = &ast.ident;
    for variant in &mut ast_enum.variants {
        for field in &mut variant.fields {
            let field_type = &field.ty;
            let satisfiable_wrapped_field: syn::Type = syn::parse(quote! { SatisfiableField<#field_type> }.into()).unwrap();
            field.ty = satisfiable_wrapped_field;
        }
    }

    let mut variants = ast_enum.variants.iter().collect::<Vec<_>>();

    let mut builder_structs = vec![];
    let mut builder_struct_accessors_from_parent = vec![];

    let cloneable_test_trait_ident = format_ident!("{}SatisfiabilityTest", concrete_ident);

    for variant in &mut variants {
        let variant_ident = &variant.ident;
        let builder_struct_name = format_ident!("Satisfiable{}{}Builder", concrete_ident, variant_ident);

        let mut builder_struct_fields = vec![];
        let mut builder_struct_field_names = vec![];
        let mut builder_struct_field_tests = vec![];
        let mut builder_struct_accessors = vec![];
        for field in &variant.fields {
            let field_ident = &field.ident;
            let field_type = &field.ty;

            builder_struct_fields.push(quote! { #field_ident: #field_type });
            builder_struct_field_names.push(quote! { #field_ident });
            builder_struct_field_tests.push(quote! { result &= self.#field_ident.satisfied_by(&#field_ident); });
            builder_struct_accessors.push(quote! {
                pub fn #field_ident(mut self, value: #field_type) -> Self {
                    self.#field_ident = value;
                    self
                }
            });
        }

        let has_fields_ident = if variant.fields.is_empty() {
            quote! {}
        } else {
            quote! {{..}}
        };

        builder_structs.push(quote! {
            #[derive(Clone, Debug, Default)]
            pub struct #builder_struct_name {
                #(#builder_struct_fields),*
            }

            impl #builder_struct_name {
                #(#builder_struct_accessors)*
            }

            impl SatisfiabilityTest for #builder_struct_name {
                type ConcreteT = #concrete_ident;
                fn is_same_variant(&self, concrete: &#concrete_ident) -> bool {
                    matches!(concrete, #concrete_ident::#variant_ident #has_fields_ident)
                }

                fn satisfied_by(&self, concrete: #concrete_ident) -> bool {
                    let #concrete_ident::#variant_ident { #(#builder_struct_field_names),* } = concrete else {
                        return false;
                    };

                    let mut result: bool = true;
                    #(#builder_struct_field_tests)*
                    result
                }
            }

            impl #cloneable_test_trait_ident for #builder_struct_name {}
        });

        builder_struct_accessors_from_parent.push(quote! {
           pub fn #variant_ident() -> #builder_struct_name { #builder_struct_name::default() }
        });
    }

    let tester_struct_ident = format_ident!("{}Test", concrete_ident);
    let satisfiable_struct_name = format_ident!("Satisfiable{}", concrete_ident);

    let generated = quote! {
        use dyn_clone::DynClone;

        pub trait #cloneable_test_trait_ident: DynClone + std::fmt::Debug + SatisfiabilityTest<ConcreteT=#concrete_ident> {}

        dyn_clone::clone_trait_object!(#cloneable_test_trait_ident);

        #[derive(Clone, Debug)]
        pub struct #tester_struct_ident(Box<dyn #cloneable_test_trait_ident>);
        impl #tester_struct_ident {
            pub fn new(test: impl #cloneable_test_trait_ident + 'static) -> Self {
                #tester_struct_ident(Box::new(test))
            }
        }

        impl<T> From<T> for #tester_struct_ident
            where T: #cloneable_test_trait_ident + 'static
        {
            fn from(value: T) -> Self {
                Self(Box::new(value))
            }
        }

        impl SatisfiabilityTest for #tester_struct_ident {
            type ConcreteT = #concrete_ident;

            fn is_same_variant(&self, concrete: &#concrete_ident) -> bool {
                self.0.is_same_variant(concrete)
            }

            fn satisfied_by(&self, concrete: Self::ConcreteT) -> bool {
                self.0.satisfied_by(concrete)
            }
        }

        impl SatisfiabilityTest for #concrete_ident {
            type ConcreteT = #concrete_ident;

            fn is_same_variant(&self, concrete: &#concrete_ident) -> bool {
                std::mem::discriminant(self) == std::mem::discriminant(concrete)
            }

            fn satisfied_by(&self, concrete: #concrete_ident) -> bool {
                self == &concrete
            }
        }

        #(#builder_structs)*

        pub struct #satisfiable_struct_name;

        impl #satisfiable_struct_name {
            #(#builder_struct_accessors_from_parent)*
        }
    };

    generated.into()
}