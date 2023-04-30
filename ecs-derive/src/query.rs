use super::*;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct QueryOpts {
    ident: syn::Ident,
    data: ast::Data<(), FieldOpts>,
}

#[derive(FromField, Debug)]
#[darling(attributes(query))]
struct FieldOpts {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    component: Option<syn::Type>,
}

struct Query {
    name: syn::Ident,
    fields: Vec<Field>,
}

struct Field {
    name: syn::Ident,
    is_mutable: bool,
    ty: syn::Type,
    component: Option<syn::Type>,
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("not a struct")]
    NotAStruct,
    #[error("field has no name")]
    NamelessField,
    #[error("struct has no fields, expected at least 1")]
    ZeroFields,
    #[error("field `{name}` is neither a `&` or a `&mut`")]
    FieldNotRef { name: syn::Ident },
}

impl TryFrom<QueryOpts> for Query {
    type Error = ParseError;

    fn try_from(value: QueryOpts) -> Result<Self, Self::Error> {
        let fields = value
            .data
            .take_struct()
            .ok_or(ParseError::NotAStruct)?
            .fields;
        if fields.is_empty() {
            return Err(ParseError::ZeroFields);
        }
        let fields = fields
            .into_iter()
            .map(|field| {
                let name = field.ident.ok_or(ParseError::NamelessField)?;
                let syn::Type::Reference(refer) = field.ty else {
                    return Err(ParseError::FieldNotRef { name });
                };
                let is_mutable = refer.mutability.is_some();
                Ok(Field {
                    name,
                    is_mutable,
                    ty: *refer.elem,
                    component: field.component,
                })
            })
            .collect::<Result<Vec<Field>, ParseError>>()?;
        Ok(Self {
            name: value.ident,
            fields,
        })
    }
}

impl QueryOpts {
    pub fn derive(self) -> TokenStream {
        let query = Query::try_from(self).unwrap_or_else(|err| panic!("{err}"));
        query.derive()
    }
}

impl Query {
    pub fn derive(self) -> TokenStream {
        let Self {
            name: query_name,
            fields: query_fields,
        } = self;

        let query_components_name = syn::Ident::new(
            &format!("{query_name}Components"),
            proc_macro2::Span::call_site(),
        );

        let is_mut = query_fields.iter().any(|field| field.is_mutable);

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
                    let name = &field.name;
                    let ty = &field.ty;
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
                    let name = &field.name;
                    let mutable = field.is_mutable;
                    let ty = field.component.as_ref().unwrap_or(&field.ty);
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
                    let name = &field.name;
                    quote! { self.#name.ids() }
                })
                .expect("Expected at least one field");

            let fields = query_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    quote! { #name }
                })
                .collect::<Vec<_>>();

            let mut get = query_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty;
                    quote! { let #name: &#ty = self.#name.get(id)?.get_component()?; }
                })
                .collect::<Vec<_>>();
            get.push(quote! {
                Some(Self::ItemReadOnly { #(#fields),* })
            });

            let mut get_mut = query_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty;
                    if field.is_mutable {
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
                    let name = &field.name;
                    if field.is_mutable {
                        quote! { mut #name }
                    } else {
                        quote! { #name }
                    }
                })
                .collect::<Vec<_>>();

            let get_phantom_data = query_fields
                .first()
                .map(|field| {
                    let name = &field.name;
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
