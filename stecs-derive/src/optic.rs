use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated};

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
    pub fn access(&self, checked: bool, id: &TokenStream, archetype: &TokenStream) -> TokenStream {
        self.access_impl(false, checked, id, archetype)
    }

    /// Access the target component mutably.
    pub fn access_mut(
        &self,
        checked: bool,
        id: &TokenStream,
        archetype: &TokenStream,
    ) -> TokenStream {
        self.access_impl(true, checked, id, archetype)
    }

    fn access_impl(
        &self,
        is_mut: bool,
        checked: bool,
        id: &TokenStream,
        archetype: &TokenStream,
    ) -> TokenStream {
        match self {
            Optic::GetId => quote! { #id },
            Optic::Access { storage, component } => {
                let storage = storage.access(archetype);

                let getter = if is_mut {
                    if checked {
                        quote! { get_mut }
                    } else {
                        quote! { get_unchecked_mut }
                    }
                } else if checked {
                    quote! { get }
                } else {
                    quote! { get_unchecked }
                };

                let mut get = quote! { #storage.#getter(#id) };
                if !checked {
                    get = quote! { unsafe { #get } };
                }

                if component.is_identity() {
                    get
                } else {
                    let value_name = quote! { __value };
                    let access = component.access_impl(checked, quote! { #value_name });
                    if checked {
                        quote! {
                            match #get {
                                None => None,
                                Some(#value_name) => { #access }
                            }
                        }
                    } else {
                        quote! {
                            {
                                let #value_name = #get;
                                #access
                            }
                        }
                    }
                }
            }
        }
    }
}

impl OpticStorage {
    pub fn access(&self, archetype: &TokenStream) -> TokenStream {
        match self {
            OpticStorage::Identity => quote! { #archetype },
            OpticStorage::Field { name, optic } => optic.access(&quote! { #archetype.#name }),
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

    fn access_impl(&self, is_checked: bool, entity: TokenStream) -> TokenStream {
        match self {
            OpticComponent::Identity => entity,
            OpticComponent::Field { name, optic } => {
                optic.access_impl(is_checked, quote! { #entity.#name })
            }
            OpticComponent::Some(optic) => {
                if optic.is_identity() {
                    if is_checked {
                        quote! { Some(#entity) }
                    } else {
                        quote! { #entity }
                    }
                } else {
                    let value_name = quote! { __value };
                    let tail = optic.access_impl(is_checked, quote! { #value_name });
                    let tail = if optic.is_prism() {
                        tail
                    } else {
                        quote! { Some(#tail) }
                    };

                    if is_checked {
                        quote! {{
                            let #value_name = #entity;
                            #tail
                        }}
                    } else {
                        quote! {
                            match #entity {
                                None => None,
                                Some(#value_name) => { #tail }
                            }
                        }
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
    Field(syn::Ident),
    Get,
}

impl Parse for Optic {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
                OpticPart::Field(name) => OpticStorage::Field {
                    name: name.clone(),
                    optic: Box::new(storage),
                },
                OpticPart::Get => {
                    return Err(syn::Error::new_spanned(
                        token,
                        "there can only be one `Get`",
                    ))
                }
            };
        }

        if get_id {
            return Ok(Optic::GetId);
        }

        // Component part
        let mut component = OpticComponent::Identity;
        for OpticPartToken(token, part) in component_parts.iter().rev() {
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
                OpticPart::Field(name) => OpticComponent::Field {
                    name: name.clone(),
                    optic: Box::new(component),
                },
                OpticPart::Get => {
                    return Err(syn::Error::new_spanned(
                        token,
                        "there can only be one `Get`",
                    ))
                }
            };
        }

        Ok(Optic::Access { storage, component })
    }
}

impl Parse for OpticPartToken {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let part = match ident.to_string().as_str() {
            // "_id" => Self::Id,
            "id" => OpticPart::GetId,
            "Some" => OpticPart::Some,
            "Get" => OpticPart::Get,
            _ => OpticPart::Field(ident.clone()),
        };
        Ok(OpticPartToken(ident, part))
    }
}
