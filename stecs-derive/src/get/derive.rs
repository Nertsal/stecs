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

        let crate_name = crate::crate_name();

        let (fields, constructor) = self.image.prepare_fields_constructor();
        let mut get_fields = constructor;

        let storage = &self.struct_of;
        let storage = quote! { #storage.inner };
        let id = &self.id;
        for (name, is_mut, optic) in fields.into_iter().rev() {
            let name = &name.mangled;
            let access = if is_mut {
                optic.access_mut(true, &quote! { #id }, &storage)
            } else {
                optic.access(true, &quote! { #id }, &storage)
            };

            get_fields = match optic {
                Optic::GetId => quote! {
                    {
                        let #name = #access;
                        #get_fields
                    }
                },
                Optic::Access { .. } => {
                    // Option<T>
                    quote! {
                        match #access {
                            None => None,
                            Some(#name) => { #get_fields }
                        }
                    }
                }
            };
        }

        quote! {{
            use #crate_name::storage::{Storage, SparseStorage};
            #[allow(non_snake_case)]
            #get_fields
        }}
    }
}
