use super::*;

#[derive(FromDeriveInput)]
// #[darling(attributes(query), supports(struct_named))]
pub struct QueryOpts {
    ident: syn::Ident,
    data: ast::Data<(), QueryField>,
    // base: syn::Ident,
}

#[derive(FromField)]
#[darling(attributes(query))]
struct QueryField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    component: Option<syn::Type>,
}

impl QueryOpts {
    pub fn derive(self) -> TokenStream {
        let Self {
            ident: query_name,
            data: query_data,
        } = self;

        let query_components_name = syn::Ident::new(
            &format!("{query_name}Components"),
            proc_macro2::Span::call_site(),
        );

        let query_fields = query_data
            .take_struct()
            .expect("Expected a struct with named fields")
            .fields;

        let is_mut = query_fields.iter().any(|field| {
            let syn::Type::Reference(ty) = &field.ty else {
                panic!("Expected a reference");
            };
            ty.mutability.is_some()
        });

        let (query_readonly, query_readonly_name) = if !is_mut {
            (quote! {}, query_name.clone())
        } else {
            let query_readonly_name = syn::Ident::new(
                &format!("{query_name}ReadOnly"),
                proc_macro2::Span::call_site(),
            );

            let fields = query_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let syn::Type::Reference(refer) = &field.ty else {
                    panic!("Expected a reference");
                };
                    let ty = &*refer.elem;
                    quote! { #name: &'a #ty, }
                })
                .collect::<Vec<_>>();

            (
                quote! {
                    #[derive(Debug)]
                    struct #query_readonly_name<'a> {
                        #(#fields)*
                    }
                },
                query_readonly_name,
            )
        };

        let struct_query = quote! {
            impl<'b, F: StorageFamily + 'static> StructQuery<F> for #query_name<'b> {
                type Components<'a> = #query_components_name<'a, F>;
            }
        };

        let components = {
            let fields = query_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let syn::Type::Reference(refer) = &field.ty else {
                        panic!("Expected a reference");
                    };
                    let mutable = refer.mutability.is_some();
                    let ty = field.component.as_ref().unwrap_or(&refer.elem);
                    if mutable {
                        quote! { #name: &'a mut F::Storage<#ty>, }
                    } else {
                        quote! { #name: &'a F::Storage<#ty>, }
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                struct #query_components_name<'a, F: StorageFamily> {
                    phantom_data: ::std::marker::PhantomData<F>,
                    #(#fields)*
                }
            }
        };

        let query_components = {
            let ids = query_fields
                .first()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        self.#name.ids()
                    }
                })
                .expect("Expected at least one field");

            let fields = query_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! { #name }
                })
                .collect::<Vec<_>>();

            let mut get = query_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let syn::Type::Reference(ty) = &field.ty else {
                        panic!("Expected reference fields");
                    };
                    let ty = &ty.elem;
                    quote! { let #name: &#ty = self.#name.get(id)?.get_component()?; }
                })
                .collect::<Vec<_>>();
            get.push(quote! {
                Some(Self::ItemReadOnly { #(#fields),* })
            });

            let mut get_mut = query_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let syn::Type::Reference(ty) = &field.ty else {
                        panic!("Expected reference fields");
                    };
                    let mutable = ty.mutability.is_some();
                    let ty = &ty.elem;
                    if mutable {
                        quote! { let #name: &mut #ty = self.#name.get_mut(id)?.get_component_mut()?; }
                    } else {
                        quote! { let #name: &#ty = self.#name.get(id)?.get_component()?; }
                    }
                })
                .collect::<Vec<_>>();
            get_mut.push(quote! {
                Some(Self::Item { #(#fields),* })
            });

            quote! {
                impl<'b, F: StorageFamily> QueryComponents<F> for #query_components_name<'b, F> {
                    type Item<'a> = #query_name<'a> where Self: 'a;
                    type ItemReadOnly<'a> = #query_readonly_name<'a> where Self: 'a;
                    fn ids(&self) -> F::IdIter {
                        #ids
                    }
                    fn get(&self, id: F::Id) -> Option<Self::ItemReadOnly<'_>> {
                        #(#get)*
                    }
                    fn get_mut(&mut self, id: F::Id) -> Option<Self::Item<'_>> {
                        #(#get_mut)*
                    }
                }
            }
        };

        let query_macro = {
            // Convert the query name to snake_case and append to "query_"
            let query_name_str = query_name.to_string();
            let mut words = query_name_str.split_inclusive(char::is_uppercase);
            let mut macro_name = "query_".to_string();
            macro_name += &words.next().unwrap().to_lowercase();
            macro_name += &words.next().unwrap().to_lowercase();
            for word in words {
                let letter = macro_name.pop().unwrap();
                macro_name += "_";
                macro_name += &letter.to_lowercase().to_string();
                macro_name += &word.to_lowercase();
            }
            let macro_name = syn::Ident::new(&macro_name, proc_macro2::Span::call_site());

            let fields = query_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let syn::Type::Reference(ty) = &field.ty else {
                        panic!("Expected reference fields");
                    };
                    let mutable = ty.mutability.is_some();
                    if mutable {
                        quote! { mut #name }
                    } else {
                        quote! { #name }
                    }
                })
                .collect::<Vec<_>>();

            let get_phantom_data = query_fields
                .first()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! { #name.phantom_data() }
                })
                .expect("Expected at least one field");

            quote! {
                macro_rules! #macro_name {
                    ($structof: expr) => {{
                        let phantom_data = $structof.inner.#get_phantom_data;
                        let components = ::ecs::query_components!(
                            $structof,
                            #query_components_name,
                            (#(#fields),*),
                            { phantom_data: phantom_data }
                        );
                        #query_name::query(components)
                    }}
                }
            }
        };

        let mut generated = TokenStream::new();
        generated.append_all(struct_query);
        generated.append_all(query_readonly);
        generated.append_all(components);
        generated.append_all(query_components);
        generated.append_all(query_macro);
        generated
    }
}
