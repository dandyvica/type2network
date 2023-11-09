use quote::quote;
use syn::{DataStruct, DeriveInput, Fields, Index};

use crate::generics::process_generics;

pub struct StructDeriveBuilder;
pub type StructBuilder = fn(&DeriveInput, &DataStruct) -> proc_macro2::TokenStream;

impl StructDeriveBuilder {
    pub fn to_network(ast: &DeriveInput, ds: &DataStruct) -> proc_macro2::TokenStream {
        // no code is created for unit structs
        if StructDeriveBuilder::is_unit(ds) {
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

    pub fn from_network(ast: &DeriveInput, ds: &DataStruct) -> proc_macro2::TokenStream {
        // no code is created for unit structs
        if StructDeriveBuilder::is_unit(ds) {
            return quote!();
        }

        let struct_name = &ast.ident;

        // call deserialize_from() call for each field
        let method_calls = ds.fields.iter().enumerate().map(|field| {
            match &field.1.ident {
                // case of a struct with named fields
                Some(field_name) => {
                    quote! {
                        FromNetworkOrder::deserialize_from(&mut self.#field_name, buffer)?;
                    }
                }
                // case of a tuple struct
                None => {
                    let index = Index::from(field.0);
                    quote! {
                        FromNetworkOrder::deserialize_from(&mut self.#index, buffer)?;
                    }
                }
            }
        });

        let (_, ty_generics, where_clause) = ast.generics.split_for_impl();
        let gen_clone = process_generics(ast);
        let (impl_generics2, _, _) = gen_clone.split_for_impl();

        quote! {
            impl #impl_generics2 FromNetworkOrder<'a> for #struct_name #ty_generics #where_clause {
                fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
                    #( #method_calls)*
                    Ok(())
                }
            }
        }
    }

    // Test whether the struct is a unit struct
    fn is_unit(ds: &DataStruct) -> bool {
        matches!(ds.fields, Fields::Unit)
    }
}
