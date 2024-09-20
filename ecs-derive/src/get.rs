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
pub struct FieldName {
    original: syn::Ident,
    pub mangled: syn::Ident,
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

        if input.parse::<Option<syn::Token![&]>>()?.is_none() {
            // Only `id` is allowed to be queries as-value
            let span = input.span();
            let optic: Optic = input.parse()?;
            if !matches!(optic, Optic::GetId) {
                return Err(syn::Error::new(
                    span,
                    "only `id` is allowed to be queried as-value. are you missing a `&`?",
                ));
            }
            return Ok(Self {
                is_mut: false,
                optic,
            });
        }

        if input.parse::<Option<syn::Token![mut]>>()?.is_some() {
            is_mut = true;
        }
        let optic: Optic = input.parse()?;
        Ok(Self { is_mut, optic })
    }
}

impl ImageOpts {
    /// Prepare fields for code generation and the constructor for the image.
    pub fn prepare_fields_constructor(&self) -> (Vec<(FieldName, bool, Optic)>, TokenStream) {
        let fields: Vec<_> = match &self {
            ImageOpts::Struct { fields, .. } => fields
                .iter()
                .map(|field| {
                    (
                        // NOTE: mangled name to avoid conflicts
                        FieldName {
                            original: field.name.clone(),
                            mangled: syn::Ident::new(
                                &format!("_ECS_field_{}", field.name),
                                proc_macro2::Span::call_site(),
                            ),
                        },
                        field.is_mut,
                        field.optic.clone(),
                    )
                })
                .collect(),
            ImageOpts::Tuple { fields } => fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    // NOTE: mangled name to avoid conflicts
                    let name = syn::Ident::new(
                        &format!("_ECS_field_{}", i),
                        proc_macro2::Span::call_site(),
                    );
                    (
                        FieldName {
                            original: name.clone(),
                            mangled: name,
                        },
                        field.is_mut,
                        field.optic.clone(),
                    )
                })
                .collect(),
        };

        let constructor = match self {
            ImageOpts::Struct { ident, .. } => {
                let fields = fields
                    .iter()
                    .map(|(FieldName { original, mangled }, _, _)| quote! { #original: #mangled });
                quote! { Some(#ident { #(#fields),* }) }
            }
            ImageOpts::Tuple { .. } => {
                let fields = fields.iter().map(|(name, _, _)| &name.mangled);
                quote! { Some(( #(#fields),* )) }
            }
        };

        (fields, constructor)
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

        let (fields, constructor) = self.image.prepare_fields_constructor();
        let mut get_fields = constructor;

        let storage = &self.struct_of;
        let id = &self.id;
        for (name, is_mut, optic) in fields.into_iter().rev() {
            let name = &name.mangled;
            let component = if is_mut {
                optic.access_mut(id, quote! { #storage })
            } else {
                optic.access(id, quote! { #storage })
            };

            get_fields = if optic.is_optional_many() {
                // Get + Prism -> Option<Option<T>>
                quote! {
                    match #component {
                        None => None,
                        Some(None) => None,
                        Some(Some(#name)) => { #get_fields }
                    }
                }
            } else if optic.is_optional() {
                // Get + Lens -> Option<T>
                quote! {
                    match #component {
                        None => None,
                        Some(#name) => { #get_fields }
                    }
                }
            } else {
                // Lens -> Option<T>
                // just `id`
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
