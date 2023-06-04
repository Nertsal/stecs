use darling::export::syn::{
    self, braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr,
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug)]
pub struct QueryOpts {
    /// The structure to query components from.
    struct_of: syn::Expr,
    /// Image type to query into.
    image: syn::Ident,
    fields: Punctuated<FieldOpts, syn::Token![,]>,
    extra_fields: Punctuated<Field, syn::Token![,]>,
}

#[derive(Debug)]
struct FieldOpts {
    is_mut: bool,
    name: syn::Ident,
    accessor: syn::Expr,
}

#[derive(Debug)]
struct Field {
    name: syn::Ident,
    expr: Option<syn::Expr>,
}

impl Parse for QueryOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_of: syn::Expr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let image: syn::Ident = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let fields;
        parenthesized!(fields in input);
        let fields: Punctuated<FieldOpts, syn::Token![,]> =
            fields.parse_terminated(FieldOpts::parse)?;

        let extra_fields: Punctuated<Field, syn::Token![,]> = if input.peek(syn::Token![,]) {
            let _: syn::Token![,] = input.parse()?;
            let fields;
            braced!(fields in input);
            fields.parse_terminated(Field::parse)?
        } else {
            Punctuated::new()
        };

        Ok(Self {
            struct_of,
            image,
            fields,
            extra_fields,
        })
    }
}

// [mut] <name> = .<field.a?.b>
impl Parse for FieldOpts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mutability: Option<syn::Token![mut]> = input.parse()?;

        let name: syn::Ident = input.parse()?;
        let _: syn::Token![=] = input.parse()?;

        let _: syn::Token![.] = input.parse()?;
        let accessor: syn::Expr = input.parse()?;

        Ok(Self {
            is_mut: mutability.is_some(),
            name,
            accessor,
        })
    }
}

// name: expr
impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let colon: Option<syn::Token![:]> = input.parse()?;
        let expr: Option<syn::Expr> = if colon.is_some() {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self { name, expr })
    }
}

impl QueryOpts {
    pub fn query(self) -> TokenStream {
        let structof = self.struct_of;
        let fields = self
            .fields
            .into_iter()
            .map(|field| {
                let access = field.accessor;
                Field {
                    name: field.name,
                    expr: Some(if field.is_mut {
                        Expr::Verbatim(quote! { &mut #structof.#access })
                    } else {
                        Expr::Verbatim(quote! { &#structof.#access })
                    }),
                }
            })
            .chain(self.extra_fields)
            .map(|field| {
                let name = field.name;
                if let Some(expr) = field.expr {
                    quote! { #name: #expr, }
                } else {
                    quote! { #name, }
                }
            })
            .collect::<Vec<_>>();

        let image = self.image;
        quote! {{
            use ::ecs::Storage;
            #image {
                #(#fields)*
            }
        }}
    }
}
