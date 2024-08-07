use quote::quote;
use syn::{DataStruct, DeriveInput, Index};

use crate::r#struct::is_unit;

use super::StructDeriveBuilder;

impl StructDeriveBuilder {
    pub fn to_network(ast: &DeriveInput, ds: &DataStruct) -> proc_macro2::TokenStream {
        // no code is created for unit structs
        if is_unit(ds) {
            return quote!();
        }

        let struct_name = &ast.ident;

        let method_calls = ds.fields.iter().enumerate().map(|field| {
            match &field.1.ident {
                // case of a struct with named fields
                Some(field_name) => {
                    quote! {
                        length += ToNetworkOrder::serialize_to(&self.#field_name, buffer)?;
                    }
                }
                // case of a tuple struct
                None => {
                    let index = Index::from(field.0);
                    quote! {
                        length += ToNetworkOrder::serialize_to(&self.#index, buffer)?;
                    }
                }
            }
        });

        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        quote! {
            impl #impl_generics ToNetworkOrder for #struct_name #ty_generics #where_clause {
                fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
                    let mut length = 0usize;
                    #( #method_calls)*
                    Ok(length)
                }
            }
        }
    }
}
