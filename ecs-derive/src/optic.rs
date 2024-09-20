use darling::export::syn::{self, parse::Parse, punctuated::Punctuated};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum Optic {
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
                    let value_name = quote! { _ECS_value };
                    let access = component.access_impl(is_mut, quote! { #value_name });
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
    pub fn access_many_mut(&self, ids: TokenStream, archetype: TokenStream) -> TokenStream {
        match self {
            Optic::GetId => ids,
            Optic::Access { storage, component } => {
                let storage = storage.access(archetype);

                let value_name = quote! { _ECS_value };
                let access = if component.is_identity() {
                    quote! {}
                } else {
                    let access = component.access_impl(true, quote! { #value_name });
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

    fn access_impl(&self, is_mut: bool, entity: TokenStream) -> TokenStream {
        match self {
            OpticComponent::Identity => entity,
            OpticComponent::Field { name, optic } => {
                optic.access_impl(is_mut, quote! { #entity.#name })
            }
            OpticComponent::Some(optic) => {
                let value_name = quote! { _ECS_value };
                let tail = optic.access_impl(is_mut, quote! { #value_name });
                let tail = if optic.is_prism() {
                    tail
                } else {
                    quote! { Some(#tail) }
                };

                let convert = if is_mut {
                    quote! { as_mut() }
                } else {
                    quote! { as_ref() }
                };

                quote! {
                    match #entity.#convert {
                        None => None,
                        Some(#value_name) => { #tail }
                    }
                }
            }
        }
    }
}

enum OpticPart {
    // Id,
    GetId,
    Some,
    Field(syn::Ident),
    Get,
}

impl Parse for Optic {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parts = Punctuated::<OpticPart, syn::Token![.]>::parse_separated_nonempty(input)?;
        // TODO: fix error spans when calling input.error()

        let parts: Vec<_> = parts.into_iter().collect();
        let empty = [];
        let mut slices = parts.split(|part| matches!(part, OpticPart::Get));
        let Some(storage_parts) = slices.next() else {
            unreachable!()
        };
        let component_parts = slices.next().unwrap_or(&empty);

        // Storage part - before the first Get
        let mut storage = OpticStorage::Identity;
        let mut get_id = false;
        for part in storage_parts.iter().rev() {
            if get_id {
                return Err(input.error("`id` must be the only optic part"));
            }

            storage = match part {
                // OpticPart::Id => {
                //     return Err(input.error("explicit `_id` is not allowed"));
                // }
                OpticPart::GetId => {
                    if !matches!(storage, OpticStorage::Identity) {
                        return Err(input.error("`id` must be the only optic part"));
                    }
                    get_id = true;
                    storage
                }
                OpticPart::Some => {
                    // TODO: maybe not
                    return Err(input.error("`Some` may only occur after `Get`"));
                }
                OpticPart::Field(name) => OpticStorage::Field {
                    name: name.clone(),
                    optic: Box::new(storage),
                },
                OpticPart::Get => return Err(input.error("there can only be one `Get`")),
            };
        }

        if get_id {
            return Ok(Optic::GetId);
        }

        // Component part
        let mut component = OpticComponent::Identity;
        for part in component_parts.iter().rev() {
            component = match part {
                // OpticPart::Id => {
                //     return Err(input.error("explicit `_id` is not allowed"));
                // }
                OpticPart::GetId => {
                    return Err(input.error("`id` must be the first and only optic part"));
                }
                OpticPart::Some => OpticComponent::Some(Box::new(component)),
                OpticPart::Field(name) => OpticComponent::Field {
                    name: name.clone(),
                    optic: Box::new(component),
                },
                OpticPart::Get => return Err(input.error("there can only be one `Get`")),
            };
        }

        Ok(Optic::Access { storage, component })
    }
}

impl Parse for OpticPart {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let part = match ident.to_string().as_str() {
            // "_id" => Self::Id,
            "id" => Self::GetId,
            "Some" => Self::Some,
            "Get" => Self::Get,
            _ => Self::Field(ident),
        };
        Ok(part)
    }
}
