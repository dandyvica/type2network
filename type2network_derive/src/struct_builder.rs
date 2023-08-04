use proc_macro2::Span;
use quote::quote;
use syn::{DataStruct, DeriveInput, Fields, Index, Lifetime, LifetimeParam, GenericParam};

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
                        length += ToNetworkOrder::to_network_order(&self.#field_name, buffer)?;
                    }
                }
                // case of a tuple struct
                None => {
                    let index = Index::from(field.0);
                    quote! {
                        length += ToNetworkOrder::to_network_order(&self.#index, buffer)?;
                    }
                }
            }
        });

        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        quote! {
            impl #impl_generics ToNetworkOrder for #struct_name #ty_generics #where_clause {
                fn to_network_order(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
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

        // call from_network_order() call for each field
        let method_calls = ds.fields.iter().enumerate().map(|field| {
            match &field.1.ident {
                // case of a struct with named fields
                Some(field_name) => {
                    quote! {
                        FromNetworkOrder::from_network_order(&mut self.#field_name, buffer)?;
                    }
                }
                // case of a tuple struct
                None => {
                    let index = Index::from(field.0);
                    quote! {
                        FromNetworkOrder::from_network_order(&mut self.#index, buffer)?;
                    }
                }
            }
        });

        let (_, ty_generics, where_clause) = ast.generics.split_for_impl();
        
        // need to add the 'a lifetime using this trick. Ref. https://users.rust-lang.org/t/add-lifetime-to/97988/5
        let mut gen_clone = ast.generics.clone();
        let lt = Lifetime::new("'a", Span::mixed_site());
        let ltp = LifetimeParam::new(lt);
        gen_clone.params.push(GenericParam::from(ltp));
        let (impl_generics2, _, _) = gen_clone.split_for_impl();

        quote! {
            impl #impl_generics2 FromNetworkOrder<'a> for #struct_name #ty_generics #where_clause {
                fn from_network_order(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
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