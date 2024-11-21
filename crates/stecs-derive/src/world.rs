use crate::syn;

use std::collections::HashMap;

use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(FromDeriveInput)]
#[darling(supports(struct_named), attributes(world))]
pub struct WorldOpts {
    // ident: syn::Ident,
    // vis: syn::Visibility,
    data: ast::Data<(), FieldOpts>,
    // generics: syn::Generics,
}

#[derive(FromField)]
#[darling(attributes(groups), forward_attrs)]
struct FieldOpts {
    ident: Option<syn::Ident>,
    // ty: syn::Type,
    attrs: Vec<syn::Attribute>,
    // groups: Option<Vec<syn::LitStr>>,
}

struct Struct {
    // name: syn::Ident,
    // visibility: syn::Visibility,
    fields: Vec<Field>,
    // generics: syn::Generics,
}

struct Field {
    name: syn::Ident,
    // ty: syn::Type,
    groups: Vec<syn::LitStr>,
    foreign: Option<syn::Ident>,
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("not a struct")]
    NotAStruct,
    #[error("field has no name")]
    NamelessField,
    #[error("there may only be one `foreign` attribute per field")]
    TooManyForeign,
}

impl TryFrom<WorldOpts> for Struct {
    type Error = ParseError;

    fn try_from(value: WorldOpts) -> Result<Self, Self::Error> {
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
                    // ty: field.ty
                    groups: extract_groups(&field.attrs)?,
                    foreign: extract_foreign(&field.attrs)?,
                })
            })
            .collect::<Result<Vec<Field>, ParseError>>()?;
        Ok(Self {
            // name: value.ident,
            // visibility: value.vis,
            fields,
            // generics: value.generics,
        })
    }
}

struct Groups(syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]>);

impl syn::parse::Parse for Groups {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident != "groups" {
            return Err(syn::Error::new_spanned(ident, "expected `groups`"));
        }

        input.parse::<syn::Token![=]>()?;

        let groups;
        syn::bracketed!(groups in input);

        let groups =
            syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated(&groups)?;

        Ok(Self(groups))
    }
}

fn extract_groups(attrs: &[syn::Attribute]) -> Result<Vec<syn::LitStr>, ParseError> {
    let mut result = Vec::new();
    for attr in attrs {
        if let Ok(groups) = attr.parse_args::<Groups>() {
            result.extend(groups.0.into_iter());
        }
    }
    Ok(result)
}

struct Foreign(syn::Ident);

impl syn::parse::Parse for Foreign {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident != "foreign" {
            return Err(syn::Error::new_spanned(ident, "expected `foreign`"));
        }

        input.parse::<syn::Token![=]>()?;

        let foreign = input.parse()?;

        Ok(Self(foreign))
    }
}

fn extract_foreign(attrs: &[syn::Attribute]) -> Result<Option<syn::Ident>, ParseError> {
    let mut result = None;
    for attr in attrs {
        if let Ok(foreign) = attr.parse_args::<Foreign>() {
            if result.is_some() {
                return Err(ParseError::TooManyForeign);
            }
            result = Some(foreign.0);
        }
    }
    Ok(result)
}

impl WorldOpts {
    pub fn derive(self) -> TokenStream {
        let query = Struct::try_from(self).unwrap_or_else(|err| panic!("{err}"));
        query.derive()
    }
}

impl Struct {
    fn derive(self) -> TokenStream {
        let all_fields = self.fields.iter().map(|field| &field.name);
        let query_all = generate_query(quote! { query_all }, all_fields);

        let query_groups = {
            let mut groups: HashMap<String, Vec<&syn::Ident>> = HashMap::new();
            for field in &self.fields {
                for group in &field.groups {
                    groups.entry(group.value()).or_default().push(&field.name);
                }
            }
            groups.into_iter().map(|(group, fields)| {
                let name =
                    syn::Ident::new(&format!("query_{}", group), proc_macro2::Span::call_site());
                generate_query(name, fields)
            })
        };

        let query_foreign = {
            let mut foreign: HashMap<&syn::Ident, Vec<&syn::Ident>> = HashMap::new();
            for field in &self.fields {
                if let Some(group) = &field.foreign {
                    foreign.entry(group).or_default().push(&field.name);
                }
            }
            foreign.into_iter().map(|(archetype, foreign)| {
                let name = syn::Ident::new(
                    &format!("query_foreign_{}", archetype),
                    proc_macro2::Span::call_site(),
                );
                Some(quote! {
                    #[macro_export]
                    macro_rules! #name {
                        ($world:expr, $args:tt) => {
                            query_foreign!($world.#archetype, [#($world,#foreign),*], $args)
                        }
                    }
                })
            })
        };

        quote! {
            #query_all
            #(#query_groups)*
            #(#query_foreign)*
        }
    }
}

// macro_rules! query_all {
//     ($world:expr, $args:tt) => {
//         query!([$world.players, $world.enemies, $world.particles], $args)
//     };
// }
fn generate_query(
    name: impl ToTokens,
    fields: impl IntoIterator<Item = impl ToTokens>,
) -> TokenStream {
    let fields = fields.into_iter().map(|field| quote! { $world.#field });
    quote! {
        #[macro_export]
        macro_rules! #name {
            ($world:expr, $args:tt) => {
                query!([#(#fields),*], $args)
            }
        }
    }
}
