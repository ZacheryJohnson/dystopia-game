mod satisfiable;

#[proc_macro_derive(Satisfiable)]
pub fn satisfiable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    satisfiable::satisfiable_impl(input)
}