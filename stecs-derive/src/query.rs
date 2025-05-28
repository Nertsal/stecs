use crate::{get::ImageOpts, optic::Optic};

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};

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
        let (fields, constructor) = self.image.prepare_fields_constructor();
        if fields.is_empty() {
            return quote! { ::std::iter::empty() };
        }

        let crate_name = crate::crate_name();

        let mut result = vec![];
        for storage in &self.struct_ofs {
            let ids_expr = quote! { #crate_name::storage::IdGenerator::ids(&#storage.ids) };
            let id_name = syn::Ident::new("__ID", proc_macro2::Span::call_site());
            let id_expr = quote! { #id_name };
            let storage = quote! { #storage.inner };

            // Get an entity by id
            let mut get_by_id = vec![];
            get_by_id.extend(fields.iter().map(|(name, is_mut, optic)| {
                let name = &name.mangled;

                let optional = match optic {
                    Optic::GetId => false,
                    Optic::Access { component, .. } => component.is_prism(),
                };
                let filter = if optional {
                    quote! { ? }
                } else {
                    quote! {}
                };

                if matches!(optic, Optic::GetId) {
                    quote! { let #name = #id_name; }
                } else if *is_mut {
                    #[cfg(not(feature = "query_mut"))]
                    panic!(
                        "The `query_mut` feature is disabled, so mutable queries are not supported"
                    );

                    #[cfg(feature = "query_mut")]
                    {
                        let component = optic.access_mut(false, &id_expr, &storage);

                        quote! {
                            let #name = #component #filter;
                            let #name = unsafe { #crate_name::UpcastLifetime::upcast(#name) };
                        }
                        // let #name = unsafe { &mut *std::ptr::from_mut(#name) };
                    }
                } else {
                    let component = optic.access(false, &id_expr, &storage);
                    quote! { let #name = #component #filter; }
                }
            }));

            // Iterate over id's
            let query = quote! {
                #ids_expr.filter_map(|#id_name| {
                    #(#get_by_id)*
                    #constructor
                })
            };

            if result.is_empty() {
                result.push(quote! { { #query } });
            } else {
                result.push(quote! { .chain({ #query }) });
            }
        }

        quote! {
            {
                use stecs::storage::Storage;
                #[allow(non_snake_case)]
                #(#result)*
            }
        }
    }
}
