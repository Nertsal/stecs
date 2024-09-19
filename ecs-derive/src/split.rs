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
    to_owned: bool,
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
            to_owned: value.clone.is_some(),
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
            to_owned: struct_to_owned,
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

        let to_owned_constraints = struct_generics
            .params
            .iter()
            .map(|generic| match generic {
                syn::GenericParam::Lifetime(_) => quote! {},
                syn::GenericParam::Type(param) => {
                    let name = &param.ident;
                    quote! { #name: ::std::clone::Clone, }
                }
                syn::GenericParam::Const(_) => quote! {},
            })
            .collect::<Vec<_>>();

        let struct_to_owneded = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! { #name: self.#name.clone(), }
                })
                .collect::<Vec<_>>();

            quote! {
                pub fn clone(&self) -> #struct_name<#generics_use>
                where #(#to_owned_constraints)*
                {
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
                #vis struct #struct_ref_name<'a, #generics_use> {
                    #(#fields)*
                }
            };
            let to_owned = if struct_to_owned {
                quote! {
                    impl<#generics> #struct_ref_name<'_, #generics_use> {
                        #struct_to_owneded
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                #derive
                #struct_ref
                #to_owned
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
                #vis struct #struct_ref_mut_name<'a, #generics_use> {
                    #(#fields)*
                }
            };
            let to_owned = if struct_to_owned {
                quote! {
                    impl<#generics> #struct_ref_mut_name<'_, #generics_use> {
                        #struct_to_owneded
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                #derive
                #struct_ref
                #to_owned
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

            let mut iter_mut = Vec::new();
            let mut get_many_mut = Vec::new();
            if fields.is_empty() {
                // No fields
                iter_mut.push(quote! { ::std::iter::empty() });
                get_many_mut.push(quote! { ::std::iter::empty() });
            } else {
                // Collect fields
                iter_mut = struct_fields
                    .iter()
                    .map(|field| {
                        let name = &field.name;
                        // TODO: Avoid cloning
                        quote! { let #name = self.#name.get_many_mut(ids.clone().into_iter()); }
                    })
                    .collect();

                // Zip fields
                let zip = std::iter::once(quote! { ids.into_iter() }).chain(
                    struct_fields.iter().map(|field| {
                        let name = &field.name;
                        quote! { .zip(#name) }
                    }),
                );
                iter_mut.extend(zip);

                // Construct the arguments for the lambda function
                let mut args = quote! { id };
                for field in struct_fields.iter().map(|field| &field.name) {
                    args = quote! { (#args, #field) };
                }

                // Filter fields to only Some
                let filter_fields = struct_fields
                    .iter()
                    .map(|field| {
                        let name = &field.name;
                        quote! { let #name = #name?; }
                    })
                    .collect::<Vec<_>>();

                get_many_mut = iter_mut.clone();

                // Construct the lambda function
                iter_mut.push(quote! {
                    .filter_map(|#args| {
                        #(#filter_fields)*
                        Some((
                            id,
                            #struct_ref_mut_name {
                                #(#fields)*
                            }
                        ))
                    })
                });

                get_many_mut.push(quote! {
                    .map(|#args| {
                        #(#filter_fields)*
                        Some(#struct_ref_mut_name {
                            #(#fields)*
                        }
                        )
                    })
                });
            }

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

                    pub fn iter(&self) -> impl Iterator<Item = (F::Id, #struct_ref_name<'_, #generics_use>)> {
                        use ::ecs::archetype::Archetype;
                        self.ids().filter_map(|id| self.get(id).map(move |item| (id, item)))
                    }

                    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (F::Id, #struct_ref_mut_name<'a, #generics_use>)> + 'a {
                        use ::ecs::archetype::Archetype;
                        let ids: Vec<_> = self.ids().collect(); // TODO: avoid allocation
                        #(#iter_mut)*
                    }

                    pub fn get_many_mut<'a>(
                        &'a mut self,
                        ids: impl Iterator<Item = F::Id>,
                    ) -> impl Iterator<Item = Option<#struct_ref_mut_name<'a, #generics_use>>> {
                        let ids: Vec<_> = ids.collect(); // TODO: avoid allocation
                        #(#get_many_mut)*
                    }
                }

                impl<#generics_family> IntoIterator for #struct_of_name<#generics_family_use> {
                    type Item = (F::Id, #struct_name<#generics_use>);
                    type IntoIter = ::ecs::archetype::ArchetypeIntoIter<F, #struct_of_name<#generics_family_use>>;

                    fn into_iter(self) -> Self::IntoIter {
                        ::ecs::archetype::ArchetypeIntoIter::new(self)
                    }
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
                    fn ids(&self) -> impl Iterator<Item = F::Id> {
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
