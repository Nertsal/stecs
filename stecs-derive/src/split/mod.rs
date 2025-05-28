mod archetype;
mod struct_ref;
mod struct_split;

use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

#[derive(FromDeriveInput)]
#[darling(supports(struct_named), attributes(split))]
pub struct SplitOpts {
    ident: syn::Ident,
    vis: syn::Visibility,
    data: ast::Data<(), FieldOpts>,
    generics: syn::Generics,
    debug: Option<()>,
    clone: Option<()>,
}

#[derive(FromField)]
#[darling(attributes(split))]
struct FieldOpts {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    nested: Option<()>,
}

struct Struct {
    crate_name: syn::Ident,
    name: syn::Ident,
    visibility: syn::Visibility,
    fields: Vec<Field>,
    generics: syn::Generics,
    derive_debug: bool,
    derive_to_owned: bool,
}

struct Field {
    name: syn::Ident,
    ty: syn::Type,
    nested: bool,
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("not a struct")]
    NotAStruct,
    #[error("field has no name")]
    NamelessField,
}

impl TryFrom<SplitOpts> for Struct {
    type Error = ParseError;

    fn try_from(value: SplitOpts) -> Result<Self, Self::Error> {
        let fields = value
            .data
            .take_struct()
            .ok_or(ParseError::NotAStruct)?
            .fields;
        let fields = fields
            .into_iter()
            .map(|field| {
                let name = field.ident.ok_or(ParseError::NamelessField)?;
                Ok(Field {
                    name,
                    ty: field.ty,
                    nested: field.nested.is_some(),
                })
            })
            .collect::<Result<Vec<Field>, ParseError>>()?;
        Ok(Self {
            crate_name: crate::crate_name(),
            name: value.ident,
            visibility: value.vis,
            fields,
            generics: value.generics,
            derive_debug: value.debug.is_some(),
            derive_to_owned: value.clone.is_some(),
        })
    }
}

impl SplitOpts {
    pub fn derive(self) -> TokenStream {
        let query = Struct::try_from(self).unwrap_or_else(|err| panic!("{err}"));
        query.derive()
    }
}

struct GenericUsage {
    generics: TokenStream,
    generics_family: TokenStream,
    generics_use: TokenStream,
    generics_family_use: TokenStream,
}

impl Struct {
    pub fn derive(self) -> TokenStream {
        // Validation
        if self.fields.is_empty() {
            panic!(
                "expected at least one field: cannot generate an archetype from an empty struct"
            );
        }
        if self.fields.iter().any(|field| field.name == "id") {
            panic!(
                "`id` is not allowed to be a field name, as it is used as a keyword inside queries"
            );
        }

        let struct_name = &self.name;
        let crate_name = crate::crate_name();

        // Split struct (SoA) name
        let struct_split_name = syn::Ident::new(
            &format!("{struct_name}Split"),
            proc_macro2::Span::call_site(),
        );
        // StructOf/Archetype (SoA with ids) name
        let struct_of_name = syn::Ident::new(
            &format!("{struct_name}StructOf"),
            proc_macro2::Span::call_site(),
        );

        // Generics
        let generic_family_name = quote! { __F }; // NOTE: mangled name to avoid conflicts
        let generic_usage = self.get_generic_usage(&generic_family_name);
        let GenericUsage {
            generics: _,
            generics_family,
            generics_use,
            generics_family_use,
        } = &generic_usage;

        let lifetime_ref_name = quote! { '__a }; // NOTE: mangled name to avoid conflicts

        // Reference structs
        let struct_ref_name =
            syn::Ident::new(&format!("{struct_name}Ref"), proc_macro2::Span::call_site());
        let struct_ref_mut_name = syn::Ident::new(
            &format!("{struct_name}RefMut"),
            proc_macro2::Span::call_site(),
        );
        let struct_references = self.generate_struct_references(
            &generic_usage,
            &lifetime_ref_name,
            &struct_ref_name,
            &struct_ref_mut_name,
        );

        // impl SplitFields
        let struct_split_fields = quote! {
            impl<#generics_family> #crate_name::archetype::SplitFields<#generic_family_name> for #struct_name<#generics_use> {
                type StructOf = #struct_of_name<#generics_family_use>;
                type Split = #struct_split_name<#generics_family_use>;
            }
        };

        // Struct split
        let struct_split = self.generate_struct_split(
            &generic_usage,
            &generic_family_name,
            &struct_split_name,
            &struct_ref_name,
            &struct_ref_mut_name,
        );

        // StructOf/Archetype
        let struct_archetype = self.generate_struct_archetype(
            &generic_usage,
            &generic_family_name,
            &struct_split_name,
            &struct_of_name,
            &struct_ref_name,
            &struct_ref_mut_name,
        );

        let mut generated = TokenStream::new();
        generated.append_all(struct_references);
        generated.append_all(struct_split_fields);
        generated.append_all(struct_split);
        generated.append_all(struct_archetype);
        generated
    }

    fn get_generic_usage(&self, generic_family_name: &TokenStream) -> GenericUsage {
        let crate_name = &self.crate_name;

        let params: Vec<_> = self.generics.params.iter().collect();
        let params_use: Vec<_> = params
            .iter()
            .map(|param| match param {
                syn::GenericParam::Type(param) => {
                    let ident = &param.ident;
                    quote! { #ident }
                }
                syn::GenericParam::Lifetime(param) => {
                    let ident = &param.lifetime;
                    quote! { #ident }
                }
                syn::GenericParam::Const(param) => {
                    let ident = &param.ident;
                    quote! { #ident }
                }
            })
            .collect();

        // Family generics, with an added StorageFamily generic
        let i = params
            .iter()
            .position(|param| !matches!(param, syn::GenericParam::Lifetime(_)))
            .unwrap_or(params.len());
        let mut params_family: Vec<_> = params.iter().map(|param| quote! { #param}).collect();
        params_family.insert(
            i,
            quote! { #generic_family_name: #crate_name::storage::StorageFamily },
        );

        let mut params_family_use = params_use.clone();
        params_family_use.insert(i, quote! { #generic_family_name });

        GenericUsage {
            generics: quote! { #(#params),* },
            generics_family: quote! { #(#params_family),* },
            generics_use: quote! { #(#params_use),*},
            generics_family_use: quote! { #(#params_family_use),* },
        }
    }
}
