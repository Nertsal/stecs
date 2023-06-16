use crate::{get::ImageOpts, optic::Optic};

use darling::export::syn::{
    self, braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
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

        Ok(Self { struct_of, image })
    }
}

impl QueryOpts {
    pub fn query(self) -> TokenStream {
        // query_components!(units, UnitComponents, (pos = pos, mut tick = body.tick), { phantom_data: Default::default() })

        quote! {{
        }}
    }
}
