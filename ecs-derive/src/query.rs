use crate::optic::{self, Optic};
use crate::syn;

use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

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
    /// Override the component type, for when it cannot be inferred correctly.
    component: Option<syn::Type>,
    /// Alias for `optic = "<nested>.<field>._get"`.
    storage: Option<String>,
    /// Query all fields of the nested storage.
    nested: Option<()>,
    optic: Option<String>,
}

struct Query {
    name: syn::Ident,
    fields: Vec<Field>,
}

struct Field {
    name: syn::Ident,
    owner: FieldOwner,
    is_mutable: bool,
    ty: syn::Type,
    ty_readonly: syn::Type,
    ty_qualified: syn::Type,
    ty_qualified_readonly: syn::Type,
    storage_type: syn::Type,
    storage: Optic,
    component: Optic,
}

enum FieldOwner {
    Owned,
    Borrowed,
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
    #[error("optic has invalid syntax: {0}")]
    OpticParse(#[from] optic::ParseError),
    #[error("cannot have both nested and storage optic specified")]
    NestedWithStorage,
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
                let (ty, ty_readonly, ty_qualified, ty_qualified_readonly, owner, storage_type) =
                    if field.nested.is_none() {
                        (
                            (*refer.elem).clone(),
                            (*refer.elem).clone(),
                            (*refer.elem).clone(),
                            *refer.elem,
                            FieldOwner::Borrowed,
                            field
                                .component
                                .clone()
                                .map(|ty| syn::Type::Verbatim(quote! { F::Storage<#ty> })),
                        )
                    } else {
                        let ty = *refer.elem;
                        if is_mutable {
                            (
                                syn::Type::Verbatim(
                                    quote! { <#ty as ::ecs::StructRef>::RefMut<'_> },
                                ),
                                syn::Type::Verbatim(quote! { <#ty as ::ecs::StructRef>::Ref<'_> }),
                                syn::Type::Verbatim(
                                    quote! { <#ty as ::ecs::StructRef>::RefMut<'a> },
                                ),
                                syn::Type::Verbatim(quote! { <#ty as ::ecs::StructRef>::Ref<'a> }),
                                FieldOwner::Owned,
                                Some(syn::Type::Verbatim(
                                    quote! { <#ty as ::ecs::SplitFields<F>>::StructOf },
                                )),
                            )
                        } else {
                            (
                                syn::Type::Verbatim(quote! { <#ty as ::ecs::StructRef>::Ref<'_> }),
                                syn::Type::Verbatim(quote! { <#ty as ::ecs::StructRef>::Ref<'_> }),
                                syn::Type::Verbatim(quote! { <#ty as ::ecs::StructRef>::Ref<'a> }),
                                syn::Type::Verbatim(quote! { <#ty as ::ecs::StructRef>::Ref<'a> }),
                                FieldOwner::Owned,
                                Some(syn::Type::Verbatim(
                                    quote! { <#ty as ::ecs::SplitFields<F>>::StructOf },
                                )),
                            )
                        }
                    };

                // Parse the provided optic
                let (mut storage, mut component) = if let Some(optic) = field.optic {
                    Optic::parse_storage_component(&optic)?
                } else {
                    (None, None)
                };

                // `storage` acts as a convenient storage-only optic composed with the field name accessor
                // component optic can still be specified in `optic`
                if let Some(optic) = field.storage {
                    if storage.is_some() {
                        return Err(ParseError::NestedWithStorage);
                    }
                    let optic: Optic = optic.parse()?;
                    storage = Some(optic.compose(Optic::Field {
                        name: name.clone(),
                        optic: Box::new(Optic::Id),
                    }));
                }

                if let Some(comp_ty) = &field.component {
                    if component.is_none() && ty != *comp_ty {
                        // The component and the queried field have different types
                        // Try to automatically access the field in the component
                        component = Some(Optic::Field {
                            name: name.clone(),
                            optic: Box::new(Optic::Id),
                        });
                    }
                }

                // By default, try to access the storage by the name of the field
                let storage = storage.unwrap_or_else(|| Optic::Field {
                    name: name.clone(),
                    optic: Box::new(Optic::Id),
                });

                // By default, the component is queried as a whole
                let component = component.unwrap_or(Optic::Id);

                // Guess the storage type by checking the component optic
                let storage_type = storage_type.unwrap_or_else(|| {
                    let ty = component.storage_type(ty.clone());
                    syn::Type::Verbatim(quote! { F::Storage<#ty> })
                });

                Ok(Field {
                    name,
                    owner,
                    is_mutable,
                    ty,
                    ty_readonly,
                    ty_qualified,
                    ty_qualified_readonly,
                    storage_type,
                    storage,
                    component,
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

        // Original variant (nested storages are rewritten)
        let (query_mutable, query_mutable_name) = {
            let query_mutable_name = syn::Ident::new(
                &format!("{query_name}Query"),
                proc_macro2::Span::call_site(),
            );

            let fields = query_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty_qualified;
                    match field.owner {
                        FieldOwner::Owned => quote! { #name: #ty, },
                        FieldOwner::Borrowed => {
                            if field.is_mutable {
                                quote! { #name: &'a mut #ty, }
                            } else {
                                quote! { #name: &'a #ty, }
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();

            (
                quote! {
                    #[derive(Debug)]
                    #[allow(dead_code)]
                    struct #query_mutable_name<'a> {
                        #(#fields)*
                    }
                },
                query_mutable_name,
            )
        };

        // Read-only variant
        let (query_readonly, query_readonly_name) = if !is_mut {
            (quote! {}, query_mutable_name.clone())
        } else {
            let query_readonly_name = syn::Ident::new(
                &format!("{query_name}ReadOnly"),
                proc_macro2::Span::call_site(),
            );

            let fields = query_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty_qualified_readonly;
                    match field.owner {
                        FieldOwner::Owned => quote! { #name: #ty, },
                        FieldOwner::Borrowed => {
                            quote! { #name: &'a #ty, }
                        }
                    }
                })
                .collect::<Vec<_>>();

            (
                quote! {
                    #[derive(Debug)]
                    #[allow(dead_code)]
                    struct #query_readonly_name<'a> {
                        #(#fields)*
                    }
                },
                query_readonly_name,
            )
        };

        // impl StructQuery
        let struct_query = quote! {
            impl<'b, F: ::ecs::StorageFamily + 'static> ::ecs::StructQuery<F> for #query_mutable_name<'b> {
                type Components<'a> = #query_components_name<'a, F>;
            }
        };

        // Components structure to hold references to the storages
        let components = {
            let fields = query_fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let mutable = field.is_mutable;
                    let ty = &field.storage_type;
                    if mutable {
                        quote! { #name: &'a mut #ty, }
                    } else {
                        quote! { #name: &'a #ty, }
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                struct #query_components_name<'a, F: ::ecs::StorageFamily + 'a> {
                    phantom_data: ::std::marker::PhantomData<F>,
                    #(#fields)*
                }
            }
        };

        // impl QueryComponents
        let query_components = {
            // TODO: check
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
                    let access = field.component.access();
                    let ty = &field.ty_readonly;
                    let ty = if let FieldOwner::Owned = field.owner {
                        quote! { #ty }
                    } else {
                        quote! { &#ty }
                    };
                    if field.component.ends_with_field() {
                        quote! { let #name: #ty = &self.#name.get(id)?#access; }
                    } else {
                        quote! { let #name: #ty = self.#name.get(id)?#access; }
                    }
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
                    let ty = if let FieldOwner::Owned = field.owner {
                        quote! { #ty }
                    } else if field.is_mutable {
                        quote! { &mut #ty }
                    } else {
                        quote! { & #ty }
                    };
                    if field.is_mutable {
                        let access = field.component.access_mut();
                        if field.component.ends_with_field() {
                            quote! { let #name: #ty = &mut self.#name.get_mut(id)?#access; }
                        } else {
                            quote! { let #name: #ty = self.#name.get_mut(id)?#access; }
                        }
                    } else {
                        let access = field.component.access();
                        if field.component.ends_with_field() {
                            quote! { let #name: #ty = &self.#name.get(id)?#access; }
                        } else {
                            quote! { let #name: #ty = self.#name.get(id)?#access; }
                        }
                    }
                })
                .collect::<Vec<_>>();
            get_mut.push(quote! {
                Some(Self::Item { #(#fields),* })
            });

            quote! {
                impl<'b, F: ::ecs::StorageFamily> ::ecs::QueryComponents<F> for #query_components_name<'b, F> {
                    type Item<'a> = #query_mutable_name<'a> where Self: 'a;
                    type ItemReadOnly<'a> = #query_readonly_name<'a> where Self: 'a;
                    fn ids(&self) -> F::IdIter {
                        use ::ecs::Storage;
                        #ids
                    }
                    fn get(&self, id: F::Id) -> Option<Self::ItemReadOnly<'_>> {
                        use ::ecs::Storage;
                        #(#get)*
                    }
                    fn get_mut(&mut self, id: F::Id) -> Option<Self::Item<'_>> {
                        use ::ecs::Storage;
                        #(#get_mut)*
                    }
                }
            }
        };

        // Macro to query the components from a StructOf
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
                    let access = field.storage.access();
                    if field.is_mutable {
                        quote! { mut #name = #access }
                    } else {
                        quote! { #name = #access }
                    }
                })
                .collect::<Vec<_>>();

            let get_phantom_data = query_fields
                .first()
                .map(|field| {
                    let access = field.storage.access();
                    quote! { #access }
                })
                .expect("Expected at least one field");

            quote! {
                macro_rules! #macro_name {
                    ($structof: expr) => {{
                        #[allow(unused_imports)]
                        use ::ecs::Storage; // Might (or not) be used for phantom data
                        let phantom_data = $structof.inner #get_phantom_data.phantom_data();
                        let components = ::ecs::query_components!(
                            $structof.inner,
                            #query_components_name,
                            (#(#fields),*),
                            { phantom_data }
                        );
                        <#query_mutable_name as ::ecs::StructQuery<_>>::query(components)
                    }}
                }
            }
        };

        let mut generated = TokenStream::new();
        generated.append_all(query_mutable);
        generated.append_all(query_readonly);
        generated.append_all(struct_query);
        generated.append_all(components);
        generated.append_all(query_components);
        generated.append_all(query_macro);
        generated
    }
}
