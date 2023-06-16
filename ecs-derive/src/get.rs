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
    struct_of: syn::Expr,
    /// Id of the entity to access.
    id: syn::Expr,
    /// The image (struct or tuple) to collect the components into.
    image: ImageOpts,
}

#[derive(Debug)]
enum ImageOpts {
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
struct StructFieldOpts {
    /// The name of the field/component.
    name: syn::Ident,
    /// The optic to the access the field/component. Can be used to rename the field in the query, or to query from a nested storage.
    optic: syn::Expr,
}

#[derive(Debug)]
struct TupleFieldOpts {
    /// The optic to the access the field/component. Can be used to rename the field in the query, or to query from a nested storage.
    optic: syn::Expr,
}

// get!(units, id, { pos, tick })

impl Parse for StorageGetOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_of: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let id: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let image_struct: Option<syn::Ident> = input.parse()?;

        let image = match image_struct {
            Some(ident) => {
                // Struct
                let fields;
                braced!(fields in input);

                let fields: Punctuated<StructFieldOpts, syn::Token![,]> =
                    fields.parse_terminated(StructFieldOpts::parse)?;
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

        Ok(Self {
            struct_of,
            id,
            image,
        })
    }
}

// Struct variant
// pos
// tick: body.tick
// damage: damage.Some

impl Parse for StructFieldOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;

        let optic = if input.parse::<Option<syn::Token![:]>>()?.is_some() {
            input.parse::<syn::Expr>()?
        } else {
            syn::Expr::Verbatim(quote! { #name })
        };

        Ok(Self { name, optic })
    }
}

// Tuple variant is same as struct without the field name
// pos
// body.tick
// damage.Some

impl Parse for TupleFieldOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let optic = input.parse::<syn::Expr>()?;
        Ok(Self { optic })
    }
}

impl StorageGetOpts {
    pub fn get(self) -> TokenStream {
        // match units.inner.pos.get(id) {
        //     None => None,
        //     Some(pos) => units
        //         .inner
        //         .tick
        //         .get(id)
        //         .map(|tick| ::structx::structx! { pos, tick }),
        // }

        // field: optic
        let fields: Vec<(syn::Ident, syn::Expr)> = match &self.image {
            ImageOpts::Struct { fields, .. } => fields
                .iter()
                .map(|field| (field.name.clone(), field.optic.clone()))
                .collect(),
            ImageOpts::Tuple { fields } => fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let name =
                        syn::Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site());
                    (name, field.optic.clone())
                })
                .collect(),
        };

        let field_names: Vec<_> = fields.iter().map(|(name, _)| name).collect();

        let mut get_fields = match self.image {
            ImageOpts::Struct { ident, .. } => quote! { Some(#ident { #(#field_names),* }) }, // struct
            ImageOpts::Tuple { .. } => quote! { Some(( #(#field_names),* )) }, // tuple
        };

        let storage = &self.struct_of;
        let id = &self.id;
        for (name, optic) in fields.iter().rev() {
            let storage = quote! { #storage.#optic };
            get_fields = quote! {
                match #storage.get(#id) {
                    None => None,
                    Some(#name) => { #get_fields }
                }
            };
        }

        quote! {{
            #get_fields
        }}
    }
}
