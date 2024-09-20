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

        let (fields, constructor) = self.image.prepare_fields_constructor();
        let mut get_fields = constructor;

        let storage = &self.struct_of;
        let id = &self.id;
        for (name, is_mut, optic) in fields.into_iter().rev() {
            let name = &name.mangled;
            let access = if is_mut {
                optic.access_mut(quote! { #id }, quote! { #storage })
            } else {
                optic.access(quote! { #id }, quote! { #storage })
            };

            get_fields = match optic {
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

        quote! {{
            #get_fields
        }}
    }
}
