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
        let span_start = input.span();

        let struct_ofs = if input.peek(syn::token::Bracket) {
            // Parse an array of struct_of's
            // [a, b, c]
            let list;
            let brackets = syn::bracketed!(list in input);
            let items =
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated(&list)?;
            if items.is_empty() {
                return Err(syn::Error::new(
                    brackets.span.join(),
                    "Expected at least one item to query from",
                ));
            }
            items.into_iter().collect()
        } else {
            // Parse a single struct_of
            let struct_of: syn::Expr = input.parse()?;
            vec![struct_of]
        };

        let _: syn::Token![,] = input.parse()?;

        let image: ImageOpts = input.parse()?;

        let foreign = match &image {
            ImageOpts::Struct { fields, .. } => fields
                .iter()
                .any(|field| matches!(field.optic, Optic::Foreign { .. })),
            ImageOpts::Tuple { fields } => fields
                .iter()
                .any(|field| matches!(field.optic, Optic::Foreign { .. })),
        };
        if foreign && struct_ofs.len() > 1 {
            let span = span_start.join(input.span()).unwrap_or(input.span());
            return Err(syn::Error::new(
                span,
                "`foreign` optics cannot be used with multiple archetypes",
            ));
        }

        #[cfg(not(feature = "query_mut"))]
        {
            // Mutability in queries turned off
            let is_mut = match &image {
                ImageOpts::Struct { fields, .. } => fields.iter().any(|field| field.is_mut),
                ImageOpts::Tuple { fields } => fields.iter().any(|field| field.is_mut),
            };
            if is_mut {
                // NOTE: sometimes in doc tests the `join` fails
                let span = span_start.join(input.span()).unwrap_or(input.span());
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
        let (fields, constructor) = self.image.prepare_fields_constructor();
        if fields.is_empty() {
            return quote! { ::std::iter::empty() };
        }

        let mut result = vec![];
        for storage in &self.struct_ofs {
            let mut query = vec![];

            // Get each field
            let id_expr = quote! { __ID }; // NOTE: mangled to avoid conflicts
            let ids_expr = quote! { ::stecs::storage::IdGenerator::ids(&#storage.ids) };
            query.extend(fields.iter().map(|(name, is_mut, optic)| {
                let name = &name.mangled;
                if *is_mut {
                    #[cfg(feature = "query_mut")]
                    {
                        let component =
                            optic.access_many_mut(ids_expr.clone(), quote! { #storage });
                        quote! { let #name = #component; }
                    }

                    #[cfg(not(feature = "query_mut"))]
                    panic!(
                        "The `query_mut` feature is disabled, so mutable queries are not supported"
                    );
                } else if matches!(optic, Optic::GetId) {
                    quote! {
                        let #name = #ids_expr;
                    }
                } else {
                    let component = optic.access(id_expr.clone(), quote! { #storage });
                    let optional = matches!(optic, Optic::Foreign { .. });
                    #[cfg(feature = "dynamic")]
                    let optional = optional || matches!(optic, Optic::Dynamic { .. });
                    if optional {
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
                    let one = |f| if f { 1 } else { 0 };
                    let optional = match optic {
                        #[cfg(feature = "dynamic")]
                        Optic::Dynamic { component, .. } => one(component.is_prism()),
                        Optic::GetId => 0,
                        Optic::Foreign { component, .. } => 1 + one(component.is_prism()),
                        Optic::Access { component, .. } => one(component.is_prism()),
                    };
                    if optional > 0 {
                        let name = &name.mangled;
                        let q: TokenStream = "?".repeat(optional).parse().unwrap();
                        quote! { let #name = #name #q; }
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

        quote! {
            {
                use ::stecs::storage::{IdGenerator, Storage};
                #[allow(non_snake_case)]
                #(#result)*
            }
        }
    }
}
