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
        let (fields, constructor) = self.image.prepare_fields_constructor();
        if fields.is_empty() {
            return quote! { ::std::iter::empty() };
        }

        let mut result = vec![];
        for storage in &self.struct_ofs {
            let mut query = vec![];

            // Get each field
            let id_expr = quote! { _ECS_field_ID }; // NOTE: mangled to avoid conflicts
            let ids_expr = quote! { #storage.ids.ids() };
            query.extend(fields.iter().map(|(name, is_mut, optic)| {
                let name = &name.mangled;
                if *is_mut {
                    let component = optic.access_many_mut(ids_expr.clone(), quote! { #storage });
                    quote! { let #name = #component; }
                } else if matches!(optic, Optic::GetId) {
                    quote! {
                        let #name = #ids_expr;
                    }
                } else {
                    let component = optic.access(id_expr.clone(), quote! { #storage });
                    quote! {
                        let #name = #ids_expr.map(|#id_expr| {
                            let value = #component;
                            value.expect("`id` must be valid")
                        });
                    }
                }
            }));

            // Zip fields
            query.push(quote! {});
            let mut tail = fields.iter();
            if let Some((name, _, _)) = tail.next() {
                let name = &name.mangled;
                query.push(quote! { #name });
            }
            query.extend(tail.map(|(name, _, _)| {
                let name = &name.mangled;
                quote! { .zip(#name) }
            }));

            // Construct args for map
            let mut args = quote! {};
            let mut tail = fields.iter();
            if let Some((name, _, _)) = tail.next() {
                let name = &name.mangled;
                args = quote! { #name };
            }
            for (name, _, _) in tail {
                let name = &name.mangled;
                args = quote! { (#args, #name) };
            }

            // Filter only values that are Some
            let filtered = fields
                .iter()
                .map(|(name, _, optic)| {
                    let optional = if let Optic::Access { component, .. } = optic {
                        component.is_prism()
                    } else {
                        false
                    };
                    if optional {
                        let name = &name.mangled;
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
