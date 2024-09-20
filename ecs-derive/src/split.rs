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

        if struct_fields.iter().any(|field| field.name == "id") {
            panic!(
                "`id` is not allowed to be a field name, as it is used as a keyword inside queries"
            );
        }

        let struct_of_name = syn::Ident::new(
            &format!("{struct_name}StructOf"),
            proc_macro2::Span::call_site(),
        );

        let generic_family_name = quote! { __F }; // NOTE: mangled name to avoid conflicts
        let (generics, generics_family, generics_use, generics_family_use) = {
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
            params_family.insert(
                i,
                quote! { #generic_family_name: ::ecs::storage::StorageFamily },
            );

            let mut params_family_use = params_use.clone();
            params_family_use.insert(i, quote! { #generic_family_name });

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
        let lifetime_ref_name = quote! { '__a }; // NOTE: mangled name to avoid conflicts
        let struct_ref = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty;
                    let ty = if field.nested {
                        quote! { <#ty as ::ecs::archetype::StructRef>::Ref<#lifetime_ref_name> }
                    } else {
                        quote! { &#lifetime_ref_name #ty }
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
                #vis struct #struct_ref_name<#lifetime_ref_name, #generics_use> {
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
                        quote! { <#ty as ::ecs::archetype::StructRef>::RefMut<#lifetime_ref_name> }
                    } else {
                        quote! { &#lifetime_ref_name mut #ty }
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
                #vis struct #struct_ref_mut_name<#lifetime_ref_name, #generics_use> {
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
            impl<#generics_family> ::ecs::archetype::SplitFields<#generic_family_name> for #struct_name<#generics_use> {
                type StructOf = #struct_of_name<#generics_family_use>;
            }
        };

        let struct_ref_impl = {
            let lifename = quote! { #lifetime_ref_name };
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
            let mut fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty;
                    let ty = if field.nested {
                        quote! { <#ty as ::ecs::archetype::SplitFields<#generic_family_name>>::StructOf }
                    } else {
                        quote! { #generic_family_name::Storage<#ty> }
                    };
                    quote! {
                        pub #name: #ty,
                    }
                })
                .collect::<Vec<_>>();
            fields.push(quote! { pub ids: #generic_family_name::Storage<()>, });

            quote! {
                #vis struct #struct_of_name<#generics_family> {
                    #(#fields)*
                }
            }
        };

        let struct_of_clone = {
            let mut constraints = struct_fields
                .iter()
                .map(|field| {
                    let ty = &field.ty;
                    if field.nested {
                        quote! { <#ty as ::ecs::archetype::SplitFields<#generic_family_name>>::StructOf: Clone }
                    } else {
                        quote! { #generic_family_name::Storage<#ty>: Clone }
                    }
                })
                .collect::<Vec<_>>();
            constraints.push(quote! { #generic_family_name::Storage<()>: Clone });

            let mut clone = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! { #name: self.#name.clone(), }
                })
                .collect::<Vec<_>>();
            clone.push(quote! { ids: self.ids.clone(), });

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
                        quote! {
                            let #name = unsafe { self.#name.get_many_unchecked_mut(self.ids.ids()) };
                        }
                    })
                    .collect();

                // Zip fields
                let zip = std::iter::once(quote! { self.ids.ids() }).chain(
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

                // Construct the lambda function
                iter_mut.push(quote! {
                    .filter_map(|#args| {
                        Some((
                            id,
                            #struct_ref_mut_name {
                                #(#fields)*
                            }
                        ))
                    })
                });

                // Get many mut

                let ids_expr = quote! { __ids };

                // Collect fields
                let get_fields = struct_fields.iter().map(|field| {
                    let name = &field.name;
                    quote! {
                        let #name = unsafe { self.#name.get_many_unchecked_mut(#ids_expr.clone()) };
                    }
                });
                get_many_mut.extend(get_fields);

                // Zip fields
                let mut zip = struct_fields.iter().map(|field| &field.name);
                if let Some(name) = zip.next() {
                    get_many_mut.push(quote! { #name });
                }
                for name in zip {
                    get_many_mut.push(quote! { .zip(#name) });
                }

                // Construct the arguments for the lambda function
                let mut args = quote! {};
                let mut args_iter = struct_fields.iter().map(|field| &field.name);
                if let Some(field) = args_iter.next() {
                    args = quote! { #field }
                }
                for field in args_iter {
                    args = quote! { (#args, #field) };
                }

                // Construct the lambda function
                get_many_mut.push(quote! {
                    .map(|#args| {
                        #struct_ref_mut_name {
                            #(#fields)*
                        }
                    })
                });
            }

            quote! {
                impl<#generics_family> #struct_of_name<#generics_family_use> {
                    pub fn new(&self) -> Self {
                        Self::default()
                    }

                    pub fn phantom_data(&self) -> ::std::marker::PhantomData<#generic_family_name> {
                        ::std::default::Default::default()
                    }

                    pub fn get(&self, id: #generic_family_name::Id) -> Option<#struct_ref_name<'_, #generics_use>> {
                        use ::ecs::storage::Storage;
                        #(#get)*
                    }

                    pub fn get_mut(&mut self, id: #generic_family_name::Id) -> Option<#struct_ref_mut_name<'_, #generics_use>> {
                        use ::ecs::storage::Storage;
                        #(#get_mut)*
                    }

                    pub fn iter(&self) -> impl Iterator<Item = (#generic_family_name::Id, #struct_ref_name<'_, #generics_use>)> {
                        use ::ecs::archetype::Archetype;
                        self.ids().filter_map(|id| self.get(id).map(move |item| (id, item)))
                    }

                    pub fn iter_mut<#lifetime_ref_name>(&#lifetime_ref_name mut self) -> impl Iterator<Item = (#generic_family_name::Id, #struct_ref_mut_name<#lifetime_ref_name, #generics_use>)> + #lifetime_ref_name {
                        use ::ecs::archetype::Archetype;
                        #(#iter_mut)*
                    }

                    pub unsafe fn get_many_unchecked_mut<#lifetime_ref_name>(
                        &#lifetime_ref_name mut self,
                        __ids: impl Iterator<Item = #generic_family_name::Id> + Clone,
                    ) -> impl Iterator<Item = #struct_ref_mut_name<#lifetime_ref_name, #generics_use>> {
                        #(#get_many_mut)*
                    }
                }

                impl<#generics_family> IntoIterator for #struct_of_name<#generics_family_use> {
                    type Item = (#generic_family_name::Id, #struct_name<#generics_use>);
                    type IntoIter = ::ecs::archetype::ArchetypeIntoIter<#generic_family_name, #struct_of_name<#generics_family_use>>;

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
                        self.#name.insert(value.#name);
                    }
                })
                .collect::<Vec<_>>();
            insert.push(quote! { let id = self.ids.insert(()); });
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
            remove.push(quote! { self.ids.remove(id)?; });
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! {#name}
                })
                .collect::<Vec<_>>();
            remove.push(quote! { Some( #struct_name { #(#fields),* } )});

            quote! {
                impl<#generics_family> ::ecs::archetype::Archetype<#generic_family_name> for #struct_of_name<#generics_family_use> {
                    type Item = #struct_name<#generics_use>;
                    fn ids(&self) -> impl Iterator<Item = #generic_family_name::Id> {
                        use ::ecs::storage::Storage;
                        self.ids.ids()
                    }
                    fn insert(&mut self, value: Self::Item) -> #generic_family_name::Id {
                        use ::ecs::storage::Storage;
                        #(#insert)*
                    }
                    fn remove(&mut self, id: #generic_family_name::Id) -> Option<Self::Item> {
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
                            ids: Default::default(),
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
