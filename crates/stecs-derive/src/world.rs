use crate::syn;

use std::collections::HashMap;

use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(FromDeriveInput)]
#[darling(supports(struct_named), attributes(world))]
pub struct WorldOpts {
    ident: syn::Ident,
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
    name: syn::Ident,
    // visibility: syn::Visibility,
    fields: Vec<Field>,
    // generics: syn::Generics,
}

struct Field {
    name: syn::Ident,
    // ty: syn::Type,
    groups: Vec<syn::LitStr>,
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("not a struct")]
    NotAStruct,
    #[error("field has no name")]
    NamelessField,
    #[error("`groups` attribute accepts only a list of string literals: {0}")]
    InvalidGroups(syn::Error),
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
                })
            })
            .collect::<Result<Vec<Field>, ParseError>>()?;
        Ok(Self {
            name: value.ident,
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
            return Err(syn::Error::new_spanned(ident, "unexpected field"));
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
        let groups = attr
            .parse_args::<Groups>()
            .map_err(ParseError::InvalidGroups)?;
        result.extend(groups.0.into_iter());
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
        let world = self.name;

        // impl Default for World
        let default = {
            let fields = self.fields.iter().map(|field| {
                let name = &field.name;
                quote! { #name: ::std::default::Default::default(), }
            });

            quote! {
                impl ::std::default::Default for #world {
                    fn default() -> Self {
                        Self {
                            #(#fields)*
                        }
                    }
                }
            }
        };

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

        quote! {
            #default
            #query_all
            #(#query_groups)*
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
        macro_rules! #name {
            ($world:expr, $args:tt) => {
                query!([#(#fields),*], $args)
            }
        }
    }
}
