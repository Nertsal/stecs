use darling::export::syn::{self, parse::Parse, punctuated::Punctuated};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

// TODO: proper optics
#[derive(Debug, Clone)]
pub enum Optic {
    Id,
    Field { name: syn::Ident, optic: Box<Optic> },
    Some(Box<Optic>),
    Get(Box<Optic>),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    // #[error("At most one Get is allowed")]
    // TooManyGets,
}

impl Optic {
    /// Compose two optics sequentially.
    pub fn compose(self, tail: Self) -> Self {
        match self {
            Optic::Id => tail,
            Optic::Field { name, optic } => Optic::Field {
                name,
                optic: Box::new(optic.compose(tail)),
            },
            Optic::Some(optic) => Optic::Some(Box::new(optic.compose(tail))),
            Optic::Get(optic) => Optic::Get(Box::new(optic.compose(tail))),
        }
    }

    // /// The type of the component of the storage queried.
    // pub fn storage_type(&self, target_type: syn::Type) -> syn::Type {
    //     match self {
    //         Optic::Id => target_type,
    //         Optic::Field { optic, .. } => optic.storage_type(target_type),
    //         Optic::Some(optic) => {
    //             let ty = optic.storage_type(target_type);
    //             syn::Type::Verbatim(quote! { Option<#ty> })
    //         }
    //     }
    // }

    // /// Whether the last accessor is field.
    // pub fn ends_with_field(&self) -> bool {
    //     match self {
    //         Optic::Id => false,
    //         Optic::Field { optic, .. } => match &**optic {
    //             Optic::Id => true,
    //             _ => optic.ends_with_field(),
    //         },
    //         Optic::Some(optic) => optic.ends_with_field(),
    //     }
    // }

    /// Access the target component immutably.
    pub fn access(&self, id: &syn::Expr, source: TokenStream) -> TokenStream {
        self.access_impl(false, id, source)
    }

    /// Access the target component mutably.
    pub fn access_mut(&self, id: &syn::Expr, source: TokenStream) -> TokenStream {
        self.access_impl(true, id, source)
    }

    fn access_impl(&self, is_mut: bool, id: &syn::Expr, source: TokenStream) -> TokenStream {
        match self {
            Optic::Id => source,
            Optic::Field { name, optic } => optic.access_impl(is_mut, id, quote! { #source.#name }),
            Optic::Some(optic) => {
                let access_value = optic.access_impl(is_mut, id, quote! { value });
                if is_mut {
                    quote! {
                        match #source.as_mut() {
                            None => None,
                            Some(value) => Some(#access_value)
                        }
                    }
                } else {
                    quote! {
                        match #source.as_ref() {
                            None => None,
                            Some(value) => Some(#access_value)
                        }
                    }
                }
            }
            Optic::Get(optic) => {
                let access_value = optic.access_impl(is_mut, id, quote! { value });
                if is_mut {
                    quote! {
                        match #source.get_mut(#id) {
                            None => None,
                            Some(value) => Some(#access_value)
                        }
                    }
                } else {
                    quote! {
                        match #source.get(#id) {
                            None => None,
                            Some(value) => Some(#access_value)
                        }
                    }
                }
            }
        }
    }

    // /// Separated by `._get`.
    // pub fn parse_storage_component(s: &str) -> Result<(Option<Self>, Option<Self>), ParseError> {
    //     let parts = s.split("._get").collect::<Vec<_>>();
    //     match &parts[..] {
    //         [] => Ok((None, None)),
    //         [component] => Ok((None, Some(component.parse()?))),
    //         [storage, ""] => Ok((Some(storage.parse()?), None)),
    //         [storage, component] => Ok((Some(storage.parse()?), Some(component.parse()?))),
    //         _ => Err(ParseError::TooManyGets),
    //     }
    // }
}

impl std::str::FromStr for Optic {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut optic = Optic::Id;
        let s = s.trim();
        for accessor in s.split('.').rev() {
            optic = match accessor.trim() {
                "_id" => Optic::Id,
                "Some" => Optic::Some(Box::new(optic)),
                field => Optic::Field {
                    name: Ident::new_raw(field, proc_macro2::Span::call_site()),
                    optic: Box::new(optic),
                },
            };
        }
        Ok(optic)
    }
}

enum OpticPart {
    Id,
    Some,
    Field(syn::Ident),
    Get,
}

impl Parse for Optic {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parts = Punctuated::<OpticPart, syn::Token![.]>::parse_separated_nonempty(input)?;

        let mut optic = Optic::Id;
        let mut has_get = false;
        for part in parts.into_iter().rev() {
            optic = match part {
                OpticPart::Id => optic,
                OpticPart::Some => Optic::Some(Box::new(optic)),
                OpticPart::Field(name) => Optic::Field {
                    name,
                    optic: Box::new(optic),
                },
                OpticPart::Get => {
                    has_get = true;
                    Optic::Get(Box::new(optic))
                }
            };
        }

        if !has_get {
            optic = optic.compose(Optic::Get(Box::new(Optic::Id)));
        }

        Ok(optic)
    }
}

impl Parse for OpticPart {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let part = match ident.to_string().as_str() {
            "_id" => Self::Id,
            "Some" => Self::Some,
            "Get" => Self::Get,
            _ => Self::Field(ident),
        };
        Ok(part)
    }
}
