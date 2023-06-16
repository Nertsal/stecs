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
    /// The image to collect the fields into.
    /// If `None`, collects into a tuple instead.
    image_struct: Option<syn::Ident>,
    /// The fields (or components) to query.
    fields: Punctuated<FieldOpts, syn::Token![,]>,
}

#[derive(Debug)]
pub struct FieldOpts {
    /// The name of the field/component.
    name: syn::Ident,
    /// The optic to the access the field/component. Can be used to rename the field in the query, or to query from a nested storage.
    optic: Option<syn::Expr>,
}

// get!(units, id, { pos, tick })

impl Parse for StorageGetOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_of: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let id: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let image_struct: Option<syn::Ident> = input.parse()?;

        let fields;
        if image_struct.is_some() {
            braced!(fields in input);
        } else {
            parenthesized!(fields in input);
        };

        let fields: Punctuated<FieldOpts, syn::Token![,]> =
            fields.parse_terminated(FieldOpts::parse)?;

        Ok(Self {
            struct_of,
            image_struct,
            id,
            fields,
        })
    }
}

// pos
// tick: body.tick
// damage: damage.Some

impl Parse for FieldOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;

        let optic = if input.parse::<Option<syn::Token![:]>>()?.is_some() {
            Some(input.parse::<syn::Expr>()?)
        } else {
            None
        };

        Ok(Self { name, optic })
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

        let fields: Vec<_> = self
            .fields
            .iter()
            .map(|field| {
                let name = &field.name;
                quote! { #name }
            })
            .collect();

        let mut get_fields = match self.image_struct {
            Some(image) => quote! { Some(#image { #(#fields),* }) }, // struct
            None => quote! { Some(( #(#fields),* )) },               // tuple
        };

        let storage = &self.struct_of;
        let id = &self.id;
        for field in self.fields.iter().rev() {
            let name = &field.name;
            let storage = quote! { #storage.#name };
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
