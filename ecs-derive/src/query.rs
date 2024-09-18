use crate::{get::ImageOpts, optic::Optic};

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

        Ok(Self { struct_of, image })
    }
}

impl QueryOpts {
    pub fn query(self) -> TokenStream {
        // let is_mut = match &self.image {
        //     ImageOpts::Struct { fields, .. } => fields.iter().any(|field| field.is_mut),
        //     ImageOpts::Tuple { fields } => fields.iter().any(|field| field.is_mut),
        // };

        // if !is_mut {
        //     // units
        //     //     .ids()
        //     //     .flat_map(|id| ::ecs::get!(units, id, (pos, body.tick)))
        //     let get = crate::get::StorageGetOpts {
        //         struct_of: self.struct_of.clone(),
        //         id: syn::Expr::Verbatim(quote! { id }),
        //         image: self.image,
        //     }
        //     .get();

        //     let storage = &self.struct_of;
        //     return quote! {{
        //         #storage.ids().flat_map(|id| { #get }.map(|item| (id, item)))
        //     }};
        // }

        // let ids = world.units.ids().collect::<Vec<_>>();
        // let health = world.units.health.get_many_mut(ids.clone().into_iter());
        // let damage = ids.clone().into_iter().map(|id| world.units.damage.get(id));
        // ids.into_iter()
        //     .zip(health)
        //     .zip(damage)
        //     .filter_map(|((id, health), damage)| {
        //         let health = health?;
        //         let damage = damage?;
        //         Some((id, (health, damage)))
        //     })

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

        let field_names: Vec<_> = fields
            .iter()
            .map(|(name, _, _)| quote! { #name, })
            .collect();

        let constructor = match self.image {
            ImageOpts::Struct { ident, .. } => quote! { Some((id, #ident { #(#field_names)* })) }, // struct
            ImageOpts::Tuple { .. } => quote! { Some(( id, #(#field_names)* )) }, // tuple
        };

        let storage = &self.struct_of;
        let mut query = vec![quote! { let ids = #storage.ids().collect::<Vec<_>>(); }];

        // Get each field
        let id_expr = syn::Expr::Verbatim(quote! { id });
        query.extend(fields.iter().map(|(name, is_mut, optic)| {
            // TODO: avoid cloning
            if *is_mut {
                let component = optic.access_mut(&id_expr, quote! { #storage });
                quote! { let #name = #component }
            } else {
                let component = optic.access(&id_expr, quote! { #storage });
                quote! { let #name = ids.clone().into_iter().map(|id| #component); }
            }
        }));

        let q = quote! { #(#query)* };
        panic!("{}", q);
    }
}
