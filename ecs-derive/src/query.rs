use crate::{get::ImageOpts, optic::Optic};

use darling::export::syn::{
    self,
    parse::{Parse, ParseStream},
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug)]
pub struct QueryOpts {
    /// The structure(s) of storages to query components from.
    struct_ofs: Vec<syn::Expr>,
    /// The image (struct or tuple) to collect the components into.
    image: ImageOpts,
}

// query!(units, { pos, tick })

impl Parse for QueryOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_ofs = if input.peek(syn::token::Bracket) {
            // Parse an array of struct_of's
            // [a, b, c]
            let list;
            syn::bracketed!(list in input);
            let items =
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated(&list)?;
            items.into_iter().collect()
        } else {
            // Parse a single struct_of
            let struct_of: syn::Expr = input.parse()?;
            vec![struct_of]
        };

        if struct_ofs.is_empty() {
            panic!("Expected at least one item to query from");
        }

        let _: syn::Token![,] = input.parse()?;

        let image: ImageOpts = input.parse()?;

        Ok(Self { struct_ofs, image })
    }
}

impl QueryOpts {
    pub fn query(self) -> TokenStream {
        let fields: Vec<(syn::Ident, bool, Optic)> = match &self.image {
            ImageOpts::Struct { fields, .. } => fields
                .iter()
                .map(|field| (field.name.clone(), field.is_mut, field.optic.clone()))
                .collect(),
            ImageOpts::Tuple { fields } => fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let name =
                        syn::Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site());
                    (name, field.is_mut, field.optic.clone())
                })
                .collect(),
        };
        if fields.is_empty() {
            return quote! { ::std::iter::empty() };
        }

        let field_names: Vec<_> = fields.iter().map(|(name, _, _)| quote! { #name }).collect();
        let constructor = match self.image {
            ImageOpts::Struct { ident, .. } => quote! { Some(#ident { #(#field_names),* }) }, // struct
            ImageOpts::Tuple { .. } => quote! { Some(( #(#field_names),* )) }, // tuple
        };

        let mut result = vec![];
        for storage in &self.struct_ofs {
            let first_field = &fields
                .iter()
                .find(|(_, _, optic)| !matches!(optic, Optic::GetId))
                .expect("at least one non-id field expected")
                .2;
            let first_storage = first_field.access_storage(quote! { #storage });
            let mut query = vec![quote! {
                // NOTE: weird name to avoid name conflicts with struct fields
                let _pls_dont_use_ids = #first_storage.ids().collect::<Vec<_>>();
            }];

            // Get each field
            let id_expr = syn::Expr::Verbatim(quote! { id });
            let ids_expr = syn::Expr::Verbatim(quote! { _pls_dont_use_ids.clone().into_iter() }); // TODO: avoid cloning
            query.extend(fields.iter().map(|(name, is_mut, optic)| {
                if *is_mut {
                    let component = optic.access_many_mut(&ids_expr, quote! { #storage });
                    quote! { let #name = #component; }
                } else {
                    let component = optic.access(&id_expr, quote! { #storage });
                    // TODO: avoid cloning
                    quote! { let #name = _pls_dont_use_ids.clone().into_iter().map(|id| #component); }
                }
            }));

            // Zip fields
            query.push(quote! {});
            let mut tail = fields.iter();
            if let Some((name, _, _)) = tail.next() {
                query.push(quote! { #name });
            }
            query.extend(tail.map(|(name, _, _)| {
                quote! { .zip(#name) }
            }));

            // Construct args for map
            let mut args = quote! {};
            let mut tail = fields.iter();
            if let Some((name, _, _)) = tail.next() {
                args = quote! { #name };
            }
            for (name, _, _) in tail {
                args = quote! { (#args, #name) };
            }

            // Filter only values that are Some
            let filtered = fields
                .iter()
                .map(|(name, _, optic)| {
                    if optic.is_optional() {
                        quote! { let #name = #name?; }
                    } else {
                        quote! {}
                    }
                })
                .collect::<Vec<_>>();

            // map
            query.push(quote! {
                .filter_map(|#args| {
                    #(#filtered)*
                    #constructor
                })
            });

            if result.is_empty() {
                result.push(quote! { { #(#query)* } });
            } else {
                result.push(quote! { .chain({ #(#query)* }) });
            }
        }

        quote! { { #(#result)* } }
    }
}
