use super::*;

#[derive(FromDeriveInput)]
#[darling(attributes(query), supports(struct_named))]
pub struct QueryOpts {
    ident: syn::Ident,
    data: ast::Data<(), QueryField>,
    structof: syn::Ident,
}

#[derive(FromField)]
struct QueryField {
    ident: Option<syn::Ident>,
}

impl QueryOpts {
    pub fn derive(self) -> TokenStream {
        let Self {
            ident: query_name,
            data: query_data,
            structof: structof_name,
        } = self;

        let query_fields = query_data
            .take_struct()
            .expect("Expected a struct with named fields")
            .fields;

        let struct_query = quote! {
            impl<'a> StructQuery for #query_name<'a> {
                type Item<'b> = #query_name<'b>;
            }
        };

        let queryable = {
            let mut get = query_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! {
                        let #name = self.#name.get(id)?;
                    }
                })
                .collect::<Vec<_>>();
            let fields = query_fields
                .iter()
                .map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! { #name }
                })
                .collect::<Vec<_>>();
            get.push(quote! {
                Some(#query_name { #(#fields),* })
            });

            quote! {
                impl<'a, F: StorageFamily> Queryable<#query_name<'a>> for #structof_name<F> {
                    fn get(&self, id: Self::Id) -> Option<<#query_name<'a> as StructQuery>::Item<'_>> {
                        #(#get)*
                    }
                }
            }
        };

        let mut generated = TokenStream::new();
        generated.append_all(struct_query);
        generated.append_all(queryable);
        generated
    }
}
