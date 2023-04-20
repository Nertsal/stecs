use super::*;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct StructOfOpts {
    ident: syn::Ident,
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

        let struct_split_fields = quote! {
            impl SplitFields for #struct_name {
                type StructOf<F: StorageFamily> = #struct_of_name<F>;
            }
        };

        let struct_of = {
            let struct_of_fields = struct_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().expect("Expected named fields");
                    let ty = &field.ty;
                    quote! {
                        #name: F::Storage<#ty>
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                struct #struct_of_name<F: StorageFamily> {
                    #(#struct_of_fields),*
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

            quote! {
                impl<F: StorageFamily> Archetype for #struct_of_name<F> {
                    type Item = #struct_name;
                    type Family = F;

                    fn insert(&mut self, value: Self::Item) -> ArchetypeId<Self> {
                        #(#insert)*
                    }

                    fn remove(&mut self, id: ArchetypeId<Self>) -> Option<Self::Item> {
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

        let struct_of_id_holder = {
            let ids = struct_fields
                .first()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        self.#name.ids()
                    }
                })
                .expect("Expected at least on field");

            quote! {
                impl<F: StorageFamily> IdHolder for #struct_of_name<F> {
                    type Id = ArchetypeId<Self>;
                    type IdIter = F::IdIter;
                    fn ids(&self) -> Self::IdIter {
                        #ids
                    }
                }
            }
        };

        let mut generated = TokenStream::new();
        generated.append_all(struct_split_fields);
        generated.append_all(struct_of);
        generated.append_all(struct_of_archetype);
        generated.append_all(struct_of_default);
        generated.append_all(struct_of_id_holder);
        generated
    }
}
