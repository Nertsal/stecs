use darling::export::syn;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

// TODO: proper optics
#[derive(Debug)]
pub enum Optic {
    Id,
    Field { name: syn::Ident, optic: Box<Optic> },
    Some(Box<Optic>),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Optic must start with a dot to indicate field access")]
    ExpectedDot,
    #[error("At most one _get is allowed")]
    TooManyGets,
}

impl Optic {
    /// The type of the component of the storage queried.
    pub fn storage_type(&self, target_type: syn::Type) -> syn::Type {
        match self {
            Optic::Id => target_type,
            Optic::Field { optic, .. } => optic.storage_type(target_type),
            Optic::Some(optic) => {
                let ty = optic.storage_type(target_type);
                syn::Type::Verbatim(quote! { Option<#ty> })
            }
        }
    }

    /// Access the target component immutably.
    pub fn access(&self) -> TokenStream {
        self.access_impl(false)
    }

    /// Access the target component mutably.
    pub fn access_mut(&self) -> TokenStream {
        self.access_impl(true)
    }

    fn access_impl(&self, is_mut: bool) -> TokenStream {
        match self {
            Optic::Id => quote! {},
            Optic::Field { name, optic } => {
                let access = optic.access_impl(is_mut);
                quote! { .#name #access }
            }
            Optic::Some(optic) => {
                let access = optic.access_impl(is_mut);
                if is_mut {
                    quote! { .as_mut()? #access }
                } else {
                    quote! { .as_ref()? #access }
                }
            }
        }
    }

    /// Separated by `._get`.
    pub fn parse_storage_component(s: &str) -> Result<(Option<Self>, Option<Self>), ParseError> {
        let parts = s.split("._get").collect::<Vec<_>>();
        match &parts[..] {
            [] => Ok((None, None)),
            [component] => Ok((None, Some(component.parse()?))),
            [storage, component] => Ok((Some(storage.parse()?), Some(component.parse()?))),
            _ => Err(ParseError::TooManyGets),
        }
    }
}

impl std::str::FromStr for Optic {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut optic = Optic::Id;
        let s = s.trim().strip_prefix('.').ok_or(ParseError::ExpectedDot)?;
        for accessor in s.split('.').rev() {
            optic = match accessor.trim() {
                "_id" => Optic::Id,
                "_Some" => Optic::Some(Box::new(optic)),
                field => Optic::Field {
                    name: Ident::new_raw(field, proc_macro2::Span::call_site()),
                    optic: Box::new(optic),
                },
            };
        }
        Ok(optic)
    }
}
