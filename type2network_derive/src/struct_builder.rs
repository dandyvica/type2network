use proc_macro2::TokenStream;
use quote::quote;
use syn::visit::Visit;
use syn::{DataStruct, DeriveInput, Fields, Index, ItemStruct};

pub struct StructDeriveBuilder;

pub type StructBuilder = fn(&DeriveInput, &DataStruct) -> proc_macro2::TokenStream;

impl StructDeriveBuilder {
    pub fn to_network(ast: &DeriveInput, ds: &DataStruct) -> proc_macro2::TokenStream {
        let name = &ast.ident;

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
            impl #impl_generics ToNetworkOrder for #name #ty_generics #where_clause {
                fn to_network_order<V: std::io::Write>(&self, buffer: &mut V) -> std::io::Result<usize> {
                    let mut length = 0usize;
                    #( #method_calls)*
                    Ok(length)
                }
            }
        }
    }

    pub fn from_network(ast: &DeriveInput, ds: &DataStruct) -> proc_macro2::TokenStream {
        let name = &ast.ident;

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

        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        quote! {
            impl #impl_generics FromNetworkOrder for #name #ty_generics #where_clause {
                fn from_network_order<'t2n>(&mut self, buffer: &mut std::io::Cursor<&'t2n [u8]>) -> std::io::Result<()> {
                    #( #method_calls)*
                    Ok(())
                }
            }
        }
    }


}

#[derive(Debug, Default)]
pub struct Visitor {
    // store the ToNetworkOrder trait code created from provided struct
    pub to_network_code: TokenStream,

    // store the FromNetworkOrder trait code created from provided struct
    pub from_network_code: TokenStream,
}

impl Visitor {
    // Build the visitor out of the AST
    // pub fn new(ast: &DeriveInput) -> Self {
    //     let mut visitor = Visitor::default();
    //     //visitor.name = Some(&ast.ident);
    //     visitor.visit_derive_input(&ast);

    //     visitor
    // }

    // build the ToNetwork trait fn
    pub fn to_network<'ast>(&self, node: &'ast ItemStruct) -> TokenStream {
        let name = &node.ident;

        let method_calls = node.fields.iter().enumerate().map(|field| {
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

        let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();

        quote! {
            impl #impl_generics ToNetworkOrder for #name #ty_generics #where_clause {
                fn to_network_order<V: Write>(&self, buffer: V) -> std::io::Result<usize> {
                    let mut length = 0usize;
                    #( #method_calls)*
                    Ok(length)
                }
            }
        }
    }

    // build the ToNetwork trait fn
    pub fn from_network<'ast>(&self, node: &'ast ItemStruct) -> TokenStream {
        let name = &node.ident;

        // call from_network_order() call for each field
        let method_calls = node.fields.iter().enumerate().map(|field| {
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

        let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();

        quote! {
            impl #impl_generics FromNetworkOrder for #name #ty_generics #where_clause {
                fn from_network_order<'t2n>(&mut self, buffer: &mut std::io::Cursor<&'t2n [u8]>) -> std::io::Result<()> {
                    #( #method_calls)*
                    Ok(())
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        println!("inside visit_item_struct()");
        // first check whether its a unit struct. In that case, no code is created
        if node.fields == Fields::Unit {
            return;
        }

        // create code for ToNetworkOrder
        self.to_network_code = self.to_network(node);

        // create code for FromNetworkOrder
        self.from_network_code = self.from_network(node);
    }
}
