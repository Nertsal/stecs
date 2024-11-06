use darling::export::syn::{self, parse::Parse, punctuated::Punctuated};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum Optic {
    Dynamic {
        ty: syn::Type,
        component: OpticComponent,
    },
    GetId,
    Access {
        storage: OpticStorage,
        component: OpticComponent,
    },
}

#[derive(Debug, Clone)]
pub enum OpticStorage {
    Identity,
    Field {
        name: syn::Ident,
        optic: Box<OpticStorage>,
    },
}

#[derive(Debug, Clone)]
pub enum OpticComponent {
    Identity,
    Field {
        name: syn::Ident,
        optic: Box<OpticComponent>,
    },
    Some(Box<OpticComponent>),
}

#[derive(Debug, Clone, Copy)]
enum Access {
    Owned,
    Borrow,
    BorrowMut,
}

impl Access {
    fn borrow(is_mut: bool) -> Self {
        if is_mut {
            Self::BorrowMut
        } else {
            Self::Borrow
        }
    }
}

impl Optic {
    /// Access the target component immutably.
    pub fn access(&self, id: TokenStream, archetype: TokenStream) -> TokenStream {
        self.access_impl(false, id, archetype)
    }

    /// Access the target component mutably.
    pub fn access_mut(&self, id: TokenStream, archetype: TokenStream) -> TokenStream {
        self.access_impl(true, id, archetype)
    }

    fn access_impl(&self, is_mut: bool, id: TokenStream, archetype: TokenStream) -> TokenStream {
        match self {
            Optic::Dynamic { ty, component } => {
                let value_name = quote! { __value };
                let storage = if is_mut {
                    quote! { #archetype.r#dyn.get_mut::<#ty>(#id) }
                } else {
                    quote! { #archetype.r#dyn.get::<#ty>(#id) }
                };

                if component.is_identity() {
                    quote! { #storage }
                } else {
                    let access = component.access_impl(Access::Owned, quote! { #value_name });
                    quote! {{
                        let #value_name = #storage;
                        #access
                    }}
                }
            }
            Optic::GetId => id,
            Optic::Access { storage, component } => {
                let storage = storage.access(archetype);

                let getter = if is_mut {
                    quote! { get_mut }
                } else {
                    quote! { get }
                };

                if component.is_identity() {
                    quote! { #storage.#getter(#id) }
                } else {
                    let value_name = quote! { __value };
                    let access =
                        component.access_impl(Access::borrow(is_mut), quote! { #value_name });
                    quote! {
                        match #storage.#getter(#id) {
                            None => None,
                            Some(#value_name) => { Some(#access) }
                        }
                    }
                }
            }
        }
    }

    /// Access many entities (identified by `ids`) mutably.
    #[cfg(feature = "query_mut")]
    pub fn access_many_mut(&self, ids: TokenStream, archetype: TokenStream) -> TokenStream {
        match self {
            Optic::Dynamic { ty, component } => {
                let value_name = quote! { __value };
                let access = if component.is_identity() {
                    quote! {}
                } else {
                    let access = component.access_impl(Access::Owned, quote! { #value_name });
                    quote! { .map(|#value_name| #access) }
                };
                quote! {
                    unsafe { #archetype.r#dyn.get_many_mut::<#ty>(#ids) } #access
                }
            }
            Optic::GetId => ids,
            Optic::Access { storage, component } => {
                let storage = storage.access(archetype);

                let value_name = quote! { __value };
                let access = if component.is_identity() {
                    quote! {}
                } else {
                    let access = component.access_impl(Access::BorrowMut, quote! { #value_name });
                    quote! { .map(|#value_name| #access) }
                };

                quote! {
                    unsafe { #storage.get_many_unchecked_mut(#ids) } #access
                }
            }
        }
    }
}

impl OpticStorage {
    pub fn access(&self, archetype: TokenStream) -> TokenStream {
        match self {
            OpticStorage::Identity => archetype,
            OpticStorage::Field { name, optic } => optic.access(quote! { #archetype.#name }),
        }
    }
}

impl OpticComponent {
    /// Whether this optic is the `Identity`.
    pub fn is_identity(&self) -> bool {
        match self {
            OpticComponent::Identity => true,
            OpticComponent::Field { .. } => false,
            OpticComponent::Some(_) => false,
        }
    }

    /// Whether this optic is a prism (as opposed to being a lens),
    /// i.e. whether the access return an `Option<T>`.
    pub fn is_prism(&self) -> bool {
        match self {
            OpticComponent::Identity => false,
            OpticComponent::Field { optic, .. } => optic.is_prism(),
            OpticComponent::Some(_) => true,
        }
    }

    fn access_impl(&self, access: Access, entity: TokenStream) -> TokenStream {
        match self {
            OpticComponent::Identity => entity,
            OpticComponent::Field { name, optic } => {
                optic.access_impl(access, quote! { #entity.#name })
            }
            OpticComponent::Some(optic) => {
                let value_name = quote! { __value };
                let tail = optic.access_impl(access, quote! { #value_name });
                let tail = if optic.is_prism() {
                    tail
                } else {
                    quote! { Some(#tail) }
                };

                let convert = match access {
                    Access::Owned => quote! {},
                    Access::Borrow => quote! { .as_ref() },
                    Access::BorrowMut => quote! { .as_mut() },
                };

                quote! {
                    match #entity #convert {
                        None => None,
                        Some(#value_name) => { #tail }
                    }
                }
            }
        }
    }
}

struct OpticPartToken(syn::Ident, OpticPart);

enum OpticPart {
    // Id,
    GetId,
    Some,
    Ident(syn::Ident),
    Get,
}

impl Parse for Optic {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<Option<syn::Token![dyn]>>()?.is_some() {
            let ty: syn::Type = input.parse()?;

            let component = if input.parse::<Option<syn::Token![.]>>()?.is_some() {
                let parts =
                    Punctuated::<OpticPartToken, syn::Token![.]>::parse_separated_nonempty(input)?;
                let parts: Vec<_> = parts.into_iter().collect();
                build_component_optic(&parts)?
            } else {
                OpticComponent::Identity
            };

            return Ok(Optic::Dynamic { ty, component });
        }

        let parts = Punctuated::<OpticPartToken, syn::Token![.]>::parse_separated_nonempty(input)?;

        let parts: Vec<_> = parts.into_iter().collect();
        let empty = [];
        let mut slices = parts.split(|part| matches!(part.1, OpticPart::Get));
        let Some(storage_parts) = slices.next() else {
            unreachable!()
        };
        let component_parts = slices.next().unwrap_or(&empty);
        if slices.next().is_some() {
            let token = &parts[storage_parts.len() + 1 + component_parts.len()].0;
            return Err(syn::Error::new_spanned(
                token,
                "there may only be at most one `Get`",
            ));
        }

        // Storage part - before the first Get
        let mut storage = OpticStorage::Identity;
        let mut get_id = false;
        for OpticPartToken(token, part) in storage_parts.iter().rev() {
            if get_id {
                return Err(syn::Error::new_spanned(
                    token,
                    "`id` must be the only optic part",
                ));
            }

            storage = match part {
                // OpticPart::Id => {
                //     return Err(syn::Error::new_spanned(token, "explicit `_id` is not allowed"));
                // }
                OpticPart::GetId => {
                    if !matches!(storage, OpticStorage::Identity) {
                        return Err(syn::Error::new_spanned(
                            token,
                            "`id` must be the only optic part",
                        ));
                    }
                    get_id = true;
                    storage
                }
                OpticPart::Some => {
                    // TODO: maybe not (optional storages?)
                    return Err(syn::Error::new_spanned(
                        token,
                        "`Some` may only occur after `Get`",
                    ));
                }
                OpticPart::Ident(name) => OpticStorage::Field {
                    name: name.clone(),
                    optic: Box::new(storage),
                },
                OpticPart::Get => {
                    return Err(syn::Error::new_spanned(
                        token,
                        "there may only be one `Get`",
                    ))
                }
            };
        }

        if get_id {
            return Ok(Optic::GetId);
        }

        // Component part
        let component = build_component_optic(component_parts)?;

        Ok(Optic::Access { storage, component })
    }
}

fn build_component_optic(parts: &[OpticPartToken]) -> syn::Result<OpticComponent> {
    let mut component = OpticComponent::Identity;
    for OpticPartToken(token, part) in parts.iter().rev() {
        component = match part {
            // OpticPart::Id => {
            //     return Err(input.error("explicit `_id` is not allowed"));
            // }
            OpticPart::GetId => {
                return Err(syn::Error::new_spanned(
                    token,
                    "`id` must be the first and only optic part",
                ));
            }
            OpticPart::Some => OpticComponent::Some(Box::new(component)),
            OpticPart::Ident(name) => OpticComponent::Field {
                name: name.clone(),
                optic: Box::new(component),
            },
            OpticPart::Get => {
                return Err(syn::Error::new_spanned(
                    token,
                    "there may only be one `Get`",
                ))
            }
        };
    }
    Ok(component)
}

impl Parse for OpticPartToken {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse::<syn::Ident>()?;
        let part = match ident.to_string().as_str() {
            // "_id" => Self::Id,
            "id" => OpticPart::GetId,
            "Some" => OpticPart::Some,
            "Get" => OpticPart::Get,
            _ => OpticPart::Ident(ident.clone()),
        };
        Ok(OpticPartToken(ident, part))
    }
}
