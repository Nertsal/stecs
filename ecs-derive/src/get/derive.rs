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
            let component = if is_mut {
                optic.access_mut(id, quote! { #storage })
            } else {
                optic.access(id, quote! { #storage })
            };

            get_fields = if optic.is_optional_many() {
                // Get + Prism -> Option<Option<T>>
                quote! {
                    match #component {
                        None => None,
                        Some(None) => None,
                        Some(Some(#name)) => { #get_fields }
                    }
                }
            } else if optic.is_optional() {
                // Get + Lens -> Option<T>
                quote! {
                    match #component {
                        None => None,
                        Some(#name) => { #get_fields }
                    }
                }
            } else {
                // Lens -> Option<T>
                // just `id`
                quote! {
                    {
                        let #name = #component;
                        #get_fields
                    }
                }
            };
        }

        quote! {{
            #get_fields
        }}
    }
}
