use crate::get::ImageOpts;

use darling::export::syn::{
    self,
    parse::{Parse, ParseStream},
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug)]
pub struct QueryOpts {
    /// The structure of storages to query components from.
    struct_of: syn::Expr,
    /// The image (struct or tuple) to collect the components into.
    image: ImageOpts,
}

// query!(units, { pos, tick })

impl Parse for QueryOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_of: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let image: ImageOpts = input.parse()?;

        let is_mut = match &image {
            ImageOpts::Struct { fields, .. } => fields.iter().any(|field| field.is_mut),
            ImageOpts::Tuple { fields } => fields.iter().any(|field| field.is_mut),
        };
        if is_mut {
            panic!("mutability in queries is not supported");
        }

        Ok(Self { struct_of, image })
    }
}

impl QueryOpts {
    pub fn query(self) -> TokenStream {
        // units
        //     .ids()
        //     .into_iter()
        //     .flat_map(|id| ::ecs::get!(units, id, (pos, body.tick)))
        let get = crate::get::StorageGetOpts {
            struct_of: self.struct_of.clone(),
            id: syn::Expr::Verbatim(quote! { id }),
            image: self.image,
        }
        .get();

        let storage = &self.struct_of;
        quote! {{
            #storage.ids().into_iter().flat_map(|id| { #get })
        }}
    }
}
