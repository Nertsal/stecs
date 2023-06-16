use darling::{
    export::syn::{self, parse_macro_input},
    FromDeriveInput,
};

mod components;
mod get;
mod optic;
mod query;
mod split;
// mod zip;

#[proc_macro_derive(SplitFields, attributes(split))]
pub fn derive_split_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    match split::SplitOpts::from_derive_input(&input) {
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

#[proc_macro]
pub fn storage_get(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as get::StorageGetOpts);
    input.get().into()
}

// #[proc_macro]
// pub fn storage_zip(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let input = parse_macro_input!(tokens as zip::StorageZipOpts);
//     input.zip().into()
// }
