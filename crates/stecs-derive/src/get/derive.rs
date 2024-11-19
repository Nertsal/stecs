use crate::optic::Optic;

use super::types::StorageGetOpts;

use proc_macro2::TokenStream;
use quote::quote;

impl StorageGetOpts {
    pub fn get(self) -> TokenStream {
        // match units.pos.get(id) {
        //     None => None,
        //     Some(pos) => match units.tick.get(id) {
        //         None => None,
        //         Some(tick) => Struct { pos, tick },
        //     },
        // }

        let storage = &self.struct_of;
        let id = &self.id;
        let generation = self.image.prepare_fields_constructor(storage);
        let mut get_fields = generation.constructor;

        for field in generation.fields.into_iter().rev() {
            let name = &field.name.mangled;
            let access = if field.is_mut {
                field.optic.access_mut(quote! { #id }, quote! { #storage })
            } else {
                field.optic.access(quote! { #id }, quote! { #storage })
            };

            get_fields = match field.optic {
                #[cfg(feature = "dynamic")]
                Optic::Dynamic { .. } | Optic::Extension { .. } => quote! {
                    {
                        let #name = #access;
                        #get_fields
                    }
                },
                Optic::GetId => quote! {
                    {
                        let #name = #access;
                        #get_fields
                    }
                },
                Optic::Access { component, .. } => {
                    if component.is_prism() {
                        // Option<Option<T>>
                        quote! {
                            match #access {
                                None => None,
                                Some(None) => None,
                                Some(Some(#name)) => { #get_fields }
                            }
                        }
                    } else {
                        // Option<T>
                        quote! {
                            match #access {
                                None => None,
                                Some(#name) => { #get_fields }
                            }
                        }
                    }
                }
            };
        }

        get_fields.extend(generation.get_extensions);

        quote! {{
            #[allow(non_snake_case)]
            #get_fields
        }}
    }
}
