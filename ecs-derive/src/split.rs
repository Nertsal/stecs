use crate::syn;

use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct SplitOpts {
    ident: syn::Ident,
    vis: syn::Visibility,
    data: ast::Data<(), FieldOpts>,
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
        } = self;

        let struct_of_name = syn::Ident::new(
            &format!("{struct_name}StructOf"),
            proc_macro2::Span::call_site(),
        );

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

            quote! {
                #[derive(Debug)]
                #vis struct #struct_ref_name<'a> {
                    #(#fields)*
                }

                impl #struct_ref_name<'_> {
                    #struct_cloned
                }
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

            quote! {
                #[derive(Debug)]
                #vis struct #struct_ref_mut_name<'a> {
                    #(#fields)*
                }

                impl #struct_ref_mut_name<'_> {
                    #struct_cloned
                }
            }
        };

        let struct_split_fields = quote! {
            impl<F: ::ecs::storage::StorageFamily> ::ecs::archetype::SplitFields<F> for #struct_name {
                type StructOf = #struct_of_name<F>;
            }
        };

        let struct_ref_impl = quote! {
            impl ::ecs::archetype::StructRef for #struct_name {
                type Ref<'a> = #struct_ref_name<'a>;
                type RefMut<'a> = #struct_ref_mut_name<'a>;
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
                #vis struct #struct_of_name<F: ::ecs::storage::StorageFamily> {
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
                impl<F: ::ecs::storage::StorageFamily> Clone for #struct_of_name<F>
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
                impl<F: ::ecs::storage::StorageFamily> #struct_of_name<F> {
                    pub fn phantom_data(&self, ) -> ::std::marker::PhantomData<F> {
                        ::std::default::Default::default()
                    }

                    pub fn get(&self, id: F::Id) -> Option<#struct_ref_name<'_>> {
                        use ::ecs::storage::Storage;
                        #(#get)*
                    }

                    pub fn get_mut(&mut self, id: F::Id) -> Option<#struct_ref_mut_name<'_>> {
                        use ::ecs::storage::Storage;
                        #(#get_mut)*
                    }

                    // TODO: impl IntoIterator
                    pub fn into_iter(mut self) -> impl Iterator<Item = (F::Id, #struct_name)> where F: 'static {
                        use ::ecs::archetype::Archetype;
                        self.ids().into_iter().filter_map(move |id| self.remove(id).map(move |item| (id, item)))
                    }

                    pub fn iter(&self) -> impl Iterator<Item = (F::Id, #struct_ref_name<'_>)> {
                        use ::ecs::archetype::Archetype;
                        self.ids().into_iter().filter_map(|id| self.get(id).map(move |item| (id, item)))
                    }

                    // TODO
                    // pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (F::Id, #struct_ref_mut_name<'a>)> + 'a {
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
                impl<F: ::ecs::storage::StorageFamily> ::ecs::archetype::Archetype<F> for #struct_of_name<F> {
                    type Item = #struct_name;
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
                impl<F: ::ecs::storage::StorageFamily> Default for #struct_of_name<F> {
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
