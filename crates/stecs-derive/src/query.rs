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
        let _span_start = input.span();

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

        #[cfg(not(feature = "query_mut"))]
        {
            // Mutability in queries turned off
            let is_mut = match &image {
                ImageOpts::Struct { fields, .. } => fields.iter().any(|field| field.is_mut),
                ImageOpts::Tuple { fields } => fields.iter().any(|field| field.is_mut),
            };
            if is_mut {
                // NOTE: sometimes in doc tests the `join` fails
                let span = _span_start.join(input.span()).unwrap_or(input.span());
                return Err(syn::Error::new(
                    span,
                    "enable the `query_mut` feature flag to allow mutable queries",
                ));
            }
        }

        Ok(Self { struct_ofs, image })
    }
}

impl QueryOpts {
    pub fn query(self) -> TokenStream {
        let mut result = vec![];

        for storage in &self.struct_ofs {
            let generation = self.image.prepare_fields_constructor(storage);
            if generation.fields.is_empty() {
                // NOTE: can return because the fields are all the same for each structof
                return quote! { ::std::iter::empty() };
            }

            let mut query = generation.get_extensions;

            // Get each field
            let id_expr = quote! { __ID }; // NOTE: mangled to avoid conflicts
            let ids_expr = quote! { ::stecs::storage::IdGenerator::ids(&#storage.ids) };
            query.extend(generation.fields.iter().map(|field| {
                let name = &field.name.mangled;
                if field.is_mut {
                    #[cfg(not(feature = "query_mut"))]
                    panic!(
                        "The `query_mut` feature is disabled, so mutable queries are not supported"
                    );

                    #[cfg(feature = "query_mut")]
                    {
                        let storage = quote! { #storage };
                        #[cfg(feature = "dynamic")]
                        let storage = field
                            .extension
                            .as_ref()
                            .map_or(storage, |ext| quote! { #ext });
                        let component = field.optic.access_many_mut(ids_expr.clone(), storage);
                        quote! { let #name = #component; }
                    }
                } else if matches!(field.optic, Optic::GetId) {
                    quote! {
                        let #name = #ids_expr;
                    }
                } else {
                    let storage = quote! { #storage };
                    #[cfg(feature = "dynamic")]
                    let storage = field
                        .extension
                        .as_ref()
                        .map_or(storage, |ext| quote! { #ext });
                    let component = field.optic.access(id_expr.clone(), quote! { #storage });
                    if field.optic.is_dynamic() {
                        quote! {
                            let #name = #ids_expr.map(|#id_expr| {
                                #component
                            });
                        }
                    } else {
                        quote! {
                            let #name = #ids_expr.map(|#id_expr| {
                                let value = #component;
                                value.expect("`id` must be valid")
                            });
                        }
                    }
                }
            }));

            // Zip fields
            query.push(quote! {});
            let mut tail = generation.fields.iter();
            if let Some(field) = tail.next() {
                let name = &field.name.mangled;
                query.push(quote! { #name });
            }
            query.extend(tail.map(|field| {
                let name = &field.name.mangled;
                quote! { .zip(#name) }
            }));

            // Construct args for map
            let mut args = quote! {};
            let mut tail = generation.fields.iter();
            if let Some(field) = tail.next() {
                let name = &field.name.mangled;
                args = quote! { #name };
            }
            for field in tail {
                let name = &field.name.mangled;
                args = quote! { (#args, #name) };
            }

            // Filter only values that are Some
            let filtered = generation
                .fields
                .iter()
                .map(|field| {
                    let optional = match &field.optic {
                        #[cfg(feature = "dynamic")]
                        Optic::Dynamic { component, .. } => component.is_prism(),
                        #[cfg(feature = "dynamic")]
                        Optic::Extension { component, .. } => component.is_prism(),
                        Optic::GetId => false,
                        Optic::Access { component, .. } => component.is_prism(),
                    };
                    if optional {
                        let name = &field.name.mangled;
                        quote! { let #name = #name?; }
                    } else {
                        quote! {}
                    }
                })
                .collect::<Vec<_>>();

            // map
            let constructor = generation.constructor;
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

        quote! {
            {
                use ::stecs::storage::Storage;
                #[allow(non_snake_case)]
                #(#result)*
            }
        }
    }
}
