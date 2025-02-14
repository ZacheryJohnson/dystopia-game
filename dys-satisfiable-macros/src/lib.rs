mod satisfiable;
mod unique;

#[proc_macro_derive(Satisfiable)]
pub fn satisfiable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    satisfiable::satisfiable_impl(input)
}

#[proc_macro_derive(UniqueKey, attributes(unique))]
pub fn unique(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unique::unique_impl(input)
}