use super::*;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct StructOfOpts {
    ident: syn::Ident,
    vis: syn::Visibility,
    data: ast::Data<(), StructOfField>,
}

#[derive(FromField)]
struct StructOfField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

impl StructOfOpts {
    pub fn derive(self) -> TokenStream {
        let Self {
            ident: struct_name,
            vis,
            data: struct_data,
        } = self;

        let struct_fields = struct_data
            .take_struct()
            .expect("StructOf only works for structs")
            .fields;

        let struct_of_name = syn::Ident::new(
            &format!("{struct_name}StructOf"),
            proc_macro2::Span::call_site(),
        );

        let struct_ref_name =
            syn::Ident::new(&format!("{struct_name}Ref"), proc_macro2::Span::call_site());
        let struct_ref = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    quote! { pub #name: &'a #ty, }
                })
                .collect::<Vec<_>>();

            quote! {
                #[derive(Debug)]
                #vis struct #struct_ref_name<'a> {
                    #(#fields)*
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
                    let name = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    quote! { pub #name: &'a mut #ty, }
                })
                .collect::<Vec<_>>();

            quote! {
                #[derive(Debug)]
                #vis struct #struct_ref_mut_name<'a> {
                    #(#fields)*
                }
            }
        };

        let struct_split_fields = quote! {
            impl<F: StorageFamily> SplitFields<F> for #struct_name {
                type StructOf = #struct_of_name<F>;
            }
        };

        let struct_of = {
            let struct_of_fields = struct_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().expect("Expected named fields");
                    let ty = &field.ty;
                    quote! {
                        pub #name: F::Storage<#ty>
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                #vis struct #struct_of_name<F: StorageFamily> {
                    #(#struct_of_fields),*
                }
            }
        };

        let struct_of_impl = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! { #name, }
                })
                .collect::<Vec<_>>();

            let mut get = struct_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
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
                    let name = field.ident.as_ref().unwrap();
                    quote! { let #name = self.#name.get_mut(id)?; }
                })
                .collect::<Vec<_>>();
            get_mut.push(quote! {
                Some(#struct_ref_mut_name {
                    #(#fields)*
                })
            });

            quote! {
                impl<F: StorageFamily> #struct_of_name<F> {
                    pub fn get(&self, id: F::Id) -> Option<#struct_ref_name<'_>> {
                        #(#get)*
                    }

                    pub fn get_mut(&mut self, id: F::Id) -> Option<#struct_ref_mut_name<'_>> {
                        #(#get_mut)*
                    }

                    pub fn iter(&self) -> impl Iterator<Item = (F::Id, #struct_ref_name<'_>)> {
                        self.ids().filter_map(|id| self.get(id).map(move |item| (id, item)))
                    }

                    // TODO
                    // pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (F::Id, #struct_ref_mut_name<'a>)> + 'a {
                    //     self.ids().filter_map(|id| self.get_mut(id).map(move |item| (id, item)))
                    // }
                }
            }
        };

        let struct_of_archetype = {
            let mut insert = struct_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        let id = self.#name.insert(value.#name);
                    }
                })
                .collect::<Vec<_>>();
            insert.push(quote! { id });

            let mut remove = struct_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        let #name = self.#name.remove(id)?;
                    }
                })
                .collect::<Vec<_>>();
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {#name}
                })
                .collect::<Vec<_>>();
            remove.push(quote! { Some( #struct_name { #(#fields),* } )});

            let ids = struct_fields
                .first()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        self.#name.ids()
                    }
                })
                .expect("Expected at least one field");

            quote! {
                impl<F: StorageFamily> Archetype<F> for #struct_of_name<F> {
                    type Item = #struct_name;
                    fn ids(&self) -> F::IdIter {
                        #ids
                    }
                    fn insert(&mut self, value: Self::Item) -> F::Id {
                        #(#insert)*
                    }
                    fn remove(&mut self, id: F::Id) -> Option<Self::Item> {
                        #(#remove)*
                    }
                }
            }
        };

        let struct_of_default = {
            let fields = struct_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        #name: Default::default()
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                impl<F: StorageFamily> Default for #struct_of_name<F> {
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
        generated.append_all(struct_of);
        generated.append_all(struct_of_impl);
        generated.append_all(struct_of_archetype);
        generated.append_all(struct_of_default);
        generated
    }
}
