use crate::syn;

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
#[darling(attributes(split))]
struct FieldOpts {
    ident: Option<syn::Ident>,
    // ty: syn::Type,
    // groups: Option<Vec<()>>,
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
    // groups: Vec<()>,
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("not a struct")]
    NotAStruct,
    #[error("field has no name")]
    NamelessField,
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

        quote! {
            #default
            #query_all
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
