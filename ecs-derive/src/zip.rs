use darling::export::syn::{
    self,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug)]
pub struct StorageZipOpts {
    /// The structure of storages to query components from.
    struct_of: syn::Expr,
    /// The fields (or components) to query.
    fields: Punctuated<FieldOpts, syn::Token![,]>,
}

#[derive(Debug)]
pub struct FieldOpts {
    /// The name of the field/component.
    name: syn::Ident,
    /// The path to the field/component. Can be used to rename the field in the query, or to query from a nested storage.
    path: Option<syn::Expr>,
}

impl Parse for StorageZipOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_of: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let fields: Punctuated<FieldOpts, syn::Token![,]> =
            input.parse_terminated(FieldOpts::parse)?;

        Ok(Self { struct_of, fields })
    }
}

impl Parse for FieldOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;

        let path = if input.parse::<Option<syn::Token![,]>>()?.is_some() {
            Some(input.parse::<syn::Expr>()?)
        } else {
            None
        };

        Ok(Self { name, path })
    }
}

impl StorageZipOpts {
    pub fn zip(self) -> TokenStream {
        // quote! {
        //     struct
        // }

        todo!()

        // let structof = self.struct_of;
        // let fields = self
        //     .fields
        //     .into_iter()
        //     .map(|field| {
        //         let access = field.accessor;
        //         Field {
        //             name: field.name,
        //             expr: Some(if field.is_mut {
        //                 Expr::Verbatim(quote! { &mut #structof.#access })
        //             } else {
        //                 Expr::Verbatim(quote! { &#structof.#access })
        //             }),
        //         }
        //     })
        //     .chain(self.extra_fields)
        //     .map(|field| {
        //         let name = field.name;
        //         if let Some(expr) = field.expr {
        //             quote! { #name: #expr, }
        //         } else {
        //             quote! { #name, }
        //         }
        //     })
        //     .collect::<Vec<_>>();

        // let image = self.image;
        // quote! {{
        //     use ::ecs::storage::Storage;
        //     #image {
        //         #(#fields)*
        //     }
        // }}
    }
}
