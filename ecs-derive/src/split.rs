use crate::syn;

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
    name: syn::Ident,
    visibility: syn::Visibility,
    fields: Vec<Field>,
    generics: syn::Generics,
    debug: bool,
    clone: bool,
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
            name: value.ident,
            visibility: value.vis,
            fields,
            generics: value.generics,
            debug: value.debug.is_some(),
            clone: value.clone.is_some(),
        })
    }
}

impl SplitOpts {
    pub fn derive(self) -> TokenStream {
        let query = Struct::try_from(self).unwrap_or_else(|err| panic!("{err}"));
        query.derive()
    }
}

impl Struct {
    pub fn derive(self) -> TokenStream {
        let Self {
            name: struct_name,
            visibility: vis,
            fields: struct_fields,
            generics: struct_generics,
            debug: struct_debug,
            clone: struct_clone,
        } = self;

        let struct_of_name = syn::Ident::new(
            &format!("{struct_name}StructOf"),
            proc_macro2::Span::call_site(),
        );

        let (generics, generics_family, generics_use, generics_family_use) = {
            // TODO: mangle names
            let params: Vec<_> = struct_generics.params.iter().collect();
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
            params_family.insert(i, quote! { F: ::ecs::storage::StorageFamily });

            let mut params_family_use = params_use.clone();
            params_family_use.insert(i, quote! { F });

            (
                quote! { #(#params),* },
                quote! { #(#params_family),* },
                quote! { #(#params_use),*},
                quote! { #(#params_family_use),* },
            )
        };

        let struct_cloned = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! { #name: self.#name.clone(), }
                })
                .collect::<Vec<_>>();

            quote! {
                pub fn clone(&self) -> #struct_name {
                    #struct_name {
                        #(#fields)*
                    }
                }
            }
        };

        let struct_ref_name =
            syn::Ident::new(&format!("{struct_name}Ref"), proc_macro2::Span::call_site());
        let struct_ref = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty;
                    let ty = if field.nested {
                        quote! { <#ty as ::ecs::archetype::StructRef>::Ref<'a> }
                    } else {
                        quote! { &'a #ty }
                    };
                    quote! { pub #name: #ty, }
                })
                .collect::<Vec<_>>();

            let derive = if struct_debug {
                quote! { #[derive(Debug)] }
            } else {
                quote! {}
            };
            let struct_ref = quote! {
                #vis struct #struct_ref_name<'a, #generics> {
                    #(#fields)*
                }
            };
            let clone = if struct_clone {
                quote! {
                    impl #struct_ref_name<'_, #generics> {
                        #struct_cloned
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                #derive
                #struct_ref
                #clone
            }
        };

        let struct_ref_mut_name = syn::Ident::new(
            &format!("{struct_name}RefMut"),
            proc_macro2::Span::call_site(),
        );
        let struct_ref_mut = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty;
                    let ty = if field.nested {
                        quote! { <#ty as ::ecs::archetype::StructRef>::RefMut<'a> }
                    } else {
                        quote! { &'a mut #ty }
                    };
                    quote! { pub #name: #ty, }
                })
                .collect::<Vec<_>>();

            let derive = if struct_debug {
                quote! { #[derive(Debug)] }
            } else {
                quote! {}
            };
            let struct_ref = quote! {
                #vis struct #struct_ref_mut_name<'a, #generics> {
                    #(#fields)*
                }
            };
            let clone = if struct_clone {
                quote! {
                    impl #struct_ref_mut_name<'_, #generics> {
                        #struct_cloned
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                #derive
                #struct_ref
                #clone
            }
        };

        let struct_split_fields = quote! {
            impl<#generics_family> ::ecs::archetype::SplitFields<F> for #struct_name<#generics_use> {
                type StructOf = #struct_of_name<#generics_family_use>;
            }
        };

        let struct_ref_impl = {
            let lifename = quote! { 'a };
            let impl_generics: Vec<_> = struct_generics
                .params
                .iter()
                .map(|param| match param {
                    syn::GenericParam::Type(param) => {
                        let ident = &param.ident;
                        quote! { #ident }
                    }
                    syn::GenericParam::Lifetime(_) => lifename.clone(),
                    syn::GenericParam::Const(param) => {
                        let ident = &param.ident;
                        quote! { #ident }
                    }
                })
                .collect();
            let impl_generics = quote! { #(#impl_generics),* };

            quote! {
                impl<#generics> ::ecs::archetype::StructRef for #struct_name<#generics_use> {
                    type Ref<#lifename> = #struct_ref_name<#lifename, #impl_generics>;
                    type RefMut<#lifename> = #struct_ref_mut_name<#lifename, #impl_generics>;
                }
            }
        };

        let struct_of = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty;
                    let ty = if field.nested {
                        quote! { <#ty as ::ecs::archetype::SplitFields<F>>::StructOf }
                    } else {
                        quote! { F::Storage<#ty> }
                    };
                    quote! {
                        pub #name: #ty,
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                #vis struct #struct_of_name<#generics_family> {
                    #(#fields)*
                }
            }
        };

        let struct_of_clone = {
            let constraints = struct_fields
                .iter()
                .map(|field| {
                    let ty = &field.ty;
                    if field.nested {
                        quote! { <#ty as ::ecs::archetype::SplitFields<F>>::StructOf: Clone }
                    } else {
                        quote! { F::Storage<#ty>: Clone }
                    }
                })
                .collect::<Vec<_>>();

            let clone = struct_fields.iter().map(|field| {
                let name = &field.name;
                quote! { #name: self.#name.clone(), }
            });

            quote! {
                impl<#generics_family> Clone for #struct_of_name<#generics_family_use>
                where
                    #(#constraints),*
                {
                    fn clone(&self) -> Self {
                        Self {
                            #(#clone)*
                        }
                    }
                }
            }
        };

        let struct_of_impl = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! { #name, }
                })
                .collect::<Vec<_>>();

            let mut get = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! { let #name = self.#name.get(id)?; }
                })
                .collect::<Vec<_>>();
            get.push(quote! {
                Some(#struct_ref_name {
                    #(#fields)*
                })
            });

            let mut get_mut = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! { let #name = self.#name.get_mut(id)?; }
                })
                .collect::<Vec<_>>();
            get_mut.push(quote! {
                Some(#struct_ref_mut_name {
                    #(#fields)*
                })
            });

            quote! {
                impl<#generics_family> #struct_of_name<#generics_family_use> {
                    pub fn new(&self) -> Self {
                        Self::default()
                    }

                    pub fn phantom_data(&self) -> ::std::marker::PhantomData<F> {
                        ::std::default::Default::default()
                    }

                    pub fn get(&self, id: F::Id) -> Option<#struct_ref_name<'_, #generics_use>> {
                        use ::ecs::storage::Storage;
                        #(#get)*
                    }

                    pub fn get_mut(&mut self, id: F::Id) -> Option<#struct_ref_mut_name<'_, #generics_use>> {
                        use ::ecs::storage::Storage;
                        #(#get_mut)*
                    }

                    // TODO: impl IntoIterator
                    pub fn into_iter(mut self) -> impl Iterator<Item = (F::Id, #struct_name<#generics_use>)> where F: 'static {
                        use ::ecs::archetype::Archetype;
                        self.ids().into_iter().filter_map(move |id| self.remove(id).map(move |item| (id, item)))
                    }

                    pub fn iter(&self) -> impl Iterator<Item = (F::Id, #struct_ref_name<'_, #generics_use>)> {
                        use ::ecs::archetype::Archetype;
                        self.ids().into_iter().filter_map(|id| self.get(id).map(move |item| (id, item)))
                    }

                    // TODO
                    // pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (F::Id, #struct_ref_mut_name<'a, #generics_use>)> + 'a {
                    //     use ::ecs::archetype::Archetype;
                    //     self.ids().filter_map(|id| self.get_mut(id).map(move |item| (id, item)))
                    // }
                }
            }
        };

        let struct_of_archetype = {
            let mut insert = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! {
                        let id = self.#name.insert(value.#name);
                    }
                })
                .collect::<Vec<_>>();
            insert.push(quote! { id });

            let mut remove = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! {
                        let #name = self.#name.remove(id)?;
                    }
                })
                .collect::<Vec<_>>();
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! {#name}
                })
                .collect::<Vec<_>>();
            remove.push(quote! { Some( #struct_name { #(#fields),* } )});

            let ids = struct_fields
                .first()
                .map(|field| {
                    let name = &field.name;
                    quote! {
                        self.#name.ids()
                    }
                })
                .expect("Expected at least one field");

            quote! {
                impl<#generics_family> ::ecs::archetype::Archetype<F> for #struct_of_name<#generics_family_use> {
                    type Item = #struct_name<#generics_use>;
                    fn ids(&self) -> ::std::collections::BTreeSet<F::Id> {
                        use ::ecs::storage::Storage;
                        #ids
                    }
                    fn insert(&mut self, value: Self::Item) -> F::Id {
                        use ::ecs::storage::Storage;
                        #(#insert)*
                    }
                    fn remove(&mut self, id: F::Id) -> Option<Self::Item> {
                        use ::ecs::storage::Storage;
                        #(#remove)*
                    }
                }
            }
        };

        let struct_of_default = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! {
                        #name: Default::default()
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                impl<#generics_family> Default for #struct_of_name<#generics_family_use> {
                    fn default() -> Self {
                        Self {
                            #(#fields),*
                        }
                    }
                }
            }
        };

        let mut generated = TokenStream::new();
        generated.append_all(struct_split_fields);
        generated.append_all(struct_ref);
        generated.append_all(struct_ref_mut);
        generated.append_all(struct_ref_impl);
        generated.append_all(struct_of);
        generated.append_all(struct_of_clone);
        generated.append_all(struct_of_impl);
        generated.append_all(struct_of_archetype);
        generated.append_all(struct_of_default);
        generated
    }
}
