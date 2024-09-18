use darling::export::syn::{self, parse::Parse, punctuated::Punctuated};
use proc_macro2::TokenStream;
use quote::quote;

// TODO: rewrite get! to use pre/post optics instead, and then implement query!

#[derive(Debug, Clone)]
pub enum PreOptic {
    Id,
    Field {
        name: syn::Ident,
        optic: Box<PreOptic>,
    },
}

#[derive(Debug, Clone)]
pub enum PostOptic {
    Id,
    Some(Box<PostOptic>),
}

#[derive(Debug, Clone)]
pub struct OpticN {
    pub pre: PreOptic,
    pub post: PostOptic,
}

impl PreOptic {
    /// Access the storage of the target component.
    pub fn access(&self, source: TokenStream) -> TokenStream {
        match self {
            PreOptic::Id => source,
            PreOptic::Field { name, optic } => optic.access(quote! { #source.#name }),
        }
    }
}

impl PostOptic {
    /// Whether the optic can fail to find the value.
    fn is_optional(&self) -> bool {
        match self {
            PostOptic::Id => false,
            PostOptic::Some(_) => true,
        }
    }

    /// Access the targetted part of the component.
    pub fn access(&self, is_mut: bool, source: TokenStream) -> TokenStream {
        match self {
            PostOptic::Id => source,
            PostOptic::Some(optic) => {
                let access_value = optic.access(is_mut, quote! { value });
                let access_value = if optic.is_optional() {
                    access_value
                } else {
                    quote! { Some(#access_value) }
                };
                if is_mut {
                    quote! {
                        match #source.as_mut() {
                            None => None,
                            Some(value) => { #access_value }
                        }
                    }
                } else {
                    quote! {
                        match #source.as_ref() {
                            None => None,
                            Some(value) => { #access_value }
                        }
                    }
                }
            }
        }
    }
}

// TODO: proper optics
#[derive(Debug, Clone)]
pub enum Optic {
    Id,
    Field { name: syn::Ident, optic: Box<Optic> },
    Some(Box<Optic>),
    Get(Box<Optic>),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {}

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

    /// Whether the optic can fail to find the value.
    fn is_optional(&self) -> bool {
        match self {
            Optic::Id => false,
            Optic::Field { optic, .. } => optic.is_optional(),
            Optic::Some(_) => true,
            Optic::Get(_) => true,
        }
    }

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
                let access_value = if optic.is_optional() {
                    access_value
                } else {
                    quote! { Some(#access_value) }
                };
                if is_mut {
                    quote! {
                        match #source.as_mut() {
                            None => None,
                            Some(value) => { #access_value }
                        }
                    }
                } else {
                    quote! {
                        match #source.as_ref() {
                            None => None,
                            Some(value) => { #access_value }
                        }
                    }
                }
            }
            Optic::Get(optic) => {
                let access_value = optic.access_impl(is_mut, id, quote! { value });
                let access_value = if optic.is_optional() {
                    access_value
                } else {
                    quote! { Some(#access_value) }
                };
                if is_mut {
                    quote! {
                        match #source.get_mut(#id) {
                            None => None,
                            Some(value) => { #access_value }
                        }
                    }
                } else {
                    quote! {
                        match #source.get(#id) {
                            None => None,
                            Some(value) => { #access_value }
                        }
                    }
                }
            }
        }
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
