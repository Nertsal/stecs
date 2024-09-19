use crate::optic::Optic;

use darling::export::syn::{
    self, braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug)]
pub struct StorageGetOpts {
    /// The structure of storages to query components from.
    pub struct_of: syn::Expr,
    /// Id of the entity to access.
    pub id: syn::Expr,
    /// The image (struct or tuple) to collect the components into.
    pub image: ImageOpts,
}

#[derive(Debug)]
pub enum ImageOpts {
    Struct {
        /// The image to collect the fields into.
        ident: syn::Ident,
        /// The fields (or components) to query.
        fields: Punctuated<StructFieldOpts, syn::Token![,]>,
    },
    Tuple {
        /// The fields (or components) to query.
        fields: Punctuated<TupleFieldOpts, syn::Token![,]>,
    },
}

#[derive(Debug)]
pub struct StructFieldOpts {
    /// The name of the field/component.
    pub name: syn::Ident,
    pub is_mut: bool,
    /// The optic to the access the field/component. Can be used to rename the field in the query, or to query from a nested storage or optional components.
    pub optic: Optic,
}

#[derive(Debug)]
pub struct TupleFieldOpts {
    pub is_mut: bool,
    /// The optic to the access the field/component. Can be used to rename the field in the query, or to query from a nested storage.
    pub optic: Optic,
}

// get!(units, id, { pos, tick })

impl Parse for StorageGetOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_of: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let id: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let image: ImageOpts = input.parse()?;

        Ok(Self {
            struct_of,
            id,
            image,
        })
    }
}

// Struct variant
// { pos, tick: body.tick, damage: damage.Get.Some }

// Tuple variant
// (pos, body.tick, damage.Get.Some)

impl Parse for ImageOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let image_struct: Option<syn::Ident> = input.parse()?;
        let image = match image_struct {
            Some(ident) => {
                // Struct
                let fields;
                braced!(fields in input);

                let fields =
                    Punctuated::<StructFieldOpts, syn::Token![,]>::parse_terminated(&fields)?;
                ImageOpts::Struct { ident, fields }
            }
            None => {
                // Tuple
                let fields;
                parenthesized!(fields in input);

                let fields: Punctuated<TupleFieldOpts, syn::Token![,]> =
                    fields.parse_terminated(TupleFieldOpts::parse)?;
                ImageOpts::Tuple { fields }
            }
        };
        Ok(image)
    }
}

// Struct variant
// pos
// tick: body.tick
// damage: damage.Get.Some

impl Parse for StructFieldOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let mut is_mut = false;

        let optic = if input.parse::<Option<syn::Token![:]>>()?.is_some() {
            input.parse::<syn::Token![&]>()?;
            if input.parse::<Option<syn::Token![mut]>>()?.is_some() {
                is_mut = true;
            }

            input.parse::<Optic>()?
        } else {
            // NOTE: `id` is treated specially to get the id of the entity,
            // so it is not allowed as a field inside Archetype's
            if name == "id" {
                Optic::GetId
            } else {
                let optic = Box::new(Optic::Get(Box::new(Optic::Id)));
                Optic::Field {
                    name: name.clone(),
                    optic,
                }
            }
        };

        Ok(Self {
            name,
            is_mut,
            optic,
        })
    }
}

// Tuple variant is same as struct without the field name
// pos
// body.tick
// damage.Some

impl Parse for TupleFieldOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut is_mut = false;
        input.parse::<syn::Token![&]>()?;
        if input.parse::<Option<syn::Token![mut]>>()?.is_some() {
            is_mut = true;
        }
        let optic: Optic = input.parse()?;
        Ok(Self { is_mut, optic })
    }
}

impl StorageGetOpts {
    pub fn get(self) -> TokenStream {
        // match units.pos.get(id) {
        //     None => None,
        //     Some(pos) => match units.tick.get(id) {
        //         None => None,
        //         Some(tick) => Struct { pos, tick },
        //     },
        // }

        // field: optic
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

        let field_names: Vec<_> = fields
            .iter()
            .map(|(name, _, _)| quote! { #name, })
            .collect();

        let mut get_fields = match self.image {
            ImageOpts::Struct { ident, .. } => quote! { Some(#ident { #(#field_names)* }) }, // struct
            ImageOpts::Tuple { .. } => quote! { Some(( #(#field_names)* )) }, // tuple
        };

        let storage = &self.struct_of;
        let id = &self.id;
        for (name, is_mut, optic) in fields.into_iter().rev() {
            let component = if is_mut {
                optic.access_mut(id, quote! { #storage })
            } else {
                optic.access(id, quote! { #storage })
            };

            get_fields = if optic.is_optional() {
                quote! {
                    match #component {
                        None => None,
                        Some(#name) => { #get_fields }
                    }
                }
            } else {
                quote! {
                    {
                        let #name = #component;
                        #get_fields
                    }
                }
            };
        }

        quote! {{
            #get_fields
        }}
    }
}
