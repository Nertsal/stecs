use super::types::*;

use crate::optic::Optic;

use darling::export::syn::{
    self, braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

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
