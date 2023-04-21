use darling::{ast, export::syn, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

mod query;
mod struct_of;

#[proc_macro_derive(StructOf)]
pub fn derive_struct_of(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    match struct_of::StructOfOpts::from_derive_input(&input) {
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
