use darling::{
    export::syn::{self, parse_macro_input},
    FromDeriveInput,
};

mod components;
mod optic;
mod query;
mod struct_of;

#[proc_macro_derive(StructOf, attributes(structof))]
pub fn derive_struct_of(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    match struct_of::StructOpts::from_derive_input(&input) {
        Ok(input) => input.derive().into(),
        Err(e) => e.write_errors().into(),
    }
}

#[proc_macro_derive(StructQuery, attributes(query))]
pub fn derive_query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    match query::QueryOpts::from_derive_input(&input) {
        Ok(input) => input.derive().into(),
        Err(e) => e.write_errors().into(),
    }
}

#[proc_macro]
pub fn query_components(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as components::QueryOpts);
    input.query().into()
}
