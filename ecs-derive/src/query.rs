use super::*;

#[derive(FromDeriveInput)]
#[darling(attributes(query), supports(struct_named))]
pub struct QueryOpts {
    ident: syn::Ident,
    data: ast::Data<(), QueryField>,
    base: syn::Ident,
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
            base: base_name,
        } = self;

        let query_fields = query_data
            .take_struct()
            .expect("Expected a struct with named fields")
            .fields;

        let mut get = query_fields
            .iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap();
                quote! {
                    let #name = struct_of.#name.get(id)?;
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
            Some(Self::Item { #(#fields),* })
        });

        quote! {
            impl<'a, F: StorageFamily> StructQuery<F> for #query_name<'a> {
                type Base = #base_name;
                type Item<'b> = #query_name<'b>;
                fn get(
                    struct_of: &<Self::Base as SplitFields<F>>::StructOf,
                    id: F::Id,
                ) -> Option<Self::Item<'_>> {
                    #(#get)*
                }
            }
        }
    }
}
