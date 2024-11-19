use std::collections::HashMap;

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

pub struct FieldGeneration {
    pub name: FieldName,
    pub is_mut: bool,
    pub optic: Optic,
    #[cfg(feature = "dynamic")]
    pub extension: Option<syn::Ident>,
}

pub struct ImageGeneration {
    pub fields: Vec<FieldGeneration>,
    pub get_extensions: Vec<TokenStream>,
    pub constructor: TokenStream,
}

impl ImageOpts {
    /// Prepare fields for code generation and the constructor for the image.
    pub fn prepare_fields_constructor(&self, storage: &syn::Expr) -> ImageGeneration {
        let fields: Vec<_> = match &self {
            ImageOpts::Struct { fields, .. } => fields
                .iter()
                .map(|field| {
                    FieldGeneration {
                        // NOTE: mangled name to avoid conflicts
                        name: FieldName {
                            original: field.name.clone(),
                            mangled: syn::Ident::new(
                                &format!("__{}", field.name),
                                proc_macro2::Span::call_site(),
                            ),
                        },
                        is_mut: field.is_mut,
                        optic: field.optic.clone(),
                        #[cfg(feature = "dynamic")]
                        extension: field.optic.get_extension(),
                    }
                })
                .collect(),
            ImageOpts::Tuple { fields } => fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    // NOTE: mangled name to avoid conflicts
                    let name =
                        syn::Ident::new(&format!("__field{}", i), proc_macro2::Span::call_site());
                    FieldGeneration {
                        name: FieldName {
                            original: name.clone(),
                            mangled: name,
                        },
                        is_mut: field.is_mut,
                        optic: field.optic.clone(),
                        #[cfg(feature = "dynamic")]
                        extension: field.optic.get_extension(),
                    }
                })
                .collect(),
        };

        #[cfg(not(feature = "dynamic"))]
        let get_extensions = vec![];
        #[cfg(feature = "dynamic")]
        let get_extensions = {
            let mut extensions: HashMap<&syn::Type, bool> = Default::default();
            for field in &fields {
                if let Optic::Extension { ty, .. } = &field.optic {
                    let value = extensions.entry(ty).or_insert(false);
                    *value = *value || field.is_mut;
                }
            }

            extensions
                .into_iter()
                .map(|(ty, is_mut)| {
                    let name = extension_name(ty);
                    if is_mut {
                        quote! {
                            let mut __EXT_empty = <#ty as ::stecs::archetype::SplitFields<_>>::Split::default();
                            let #name = #storage.r#dyn.get_ext_mut::<#ty>().unwrap_or(&mut __EXT_empty);
                        }
                    } else {
                        quote! {
                            let __EXT_empty = <#ty as ::stecs::archetype::SplitFields<_>>::Split::default();
                            let #name = #storage.r#dyn.get_ext::<#ty>().unwrap_or(&__EXT_empty);
                        }
                    }
                })
                .collect()
        };

        let constructor = match self {
            ImageOpts::Struct { ident, .. } => {
                let fields = fields.iter().map(|field| {
                    let original = &field.name.original;
                    let mangled = &field.name.mangled;
                    quote! { #original: #mangled }
                });
                quote! { Some(#ident { #(#fields),* }) }
            }
            ImageOpts::Tuple { .. } => {
                let fields = fields.iter().map(|field| &field.name.mangled);
                quote! { Some(( #(#fields),* )) }
            }
        };

        ImageGeneration {
            fields,
            get_extensions,
            constructor,
        }
    }
}

impl Optic {
    #[cfg(feature = "dynamic")]
    pub fn get_extension(&self) -> Option<syn::Ident> {
        match self {
            Optic::Extension { ty, .. } => Some(extension_name(ty)),
            _ => None,
        }
    }
}

fn extension_name(ty: &syn::Type) -> syn::Ident {
    syn::Ident::new(
        &format!("__EXT_{}", quote! { #ty }),
        proc_macro2::Span::call_site(),
    )
}
