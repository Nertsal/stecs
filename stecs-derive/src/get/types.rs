use crate::optic::Optic;

use darling::export::syn::{self, punctuated::Punctuated};
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
                                &format!("__{}", field.name),
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
                    let name =
                        syn::Ident::new(&format!("__field{}", i), proc_macro2::Span::call_site());
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
