use quote::quote;
use syn::{
    parenthesized, token, AttrStyle, Attribute, DataStruct, DeriveInput, Field, Fields, Ident,
    Index,
};

use crate::generics::process_generics;

pub struct StructDeriveBuilder;
pub type StructBuilder = fn(&DeriveInput, &DataStruct) -> proc_macro2::TokenStream;

//
#[derive(Debug, Default)]
enum AttrKind {
    #[default]
    NoAttribute,
    NoAction,
    Call(Ident),
    Block(proc_macro2::TokenStream),
}

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
                Some(_) => process_named_field(field.1),
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

fn process_named_field(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();

    // we only support 1 attribute
    if field.attrs.len() > 1 {
        unimplemented!("only support a single attribue on field {}", field_name);
    }

    // analyze attribute
    let kind = if field.attrs.len() == 1 {
        process_attr(&field.attrs[0])
    } else {
        AttrKind::NoAttribute
    };

    // no return code depending on attribute
    match kind {
        // no attribute provided: just deserialize the field as other ones
        AttrKind::NoAttribute => {
            quote! {
                FromNetworkOrder::deserialize_from(&mut self.#field_name, buffer)?;
            }
        }

        // #[deser(no)]: don't do anything
        AttrKind::NoAction => quote!(),

        // a function was provided to the deser attribute: so just call it
        // e.g.: #[deser(my_func)]
        AttrKind::Call(func) => {
            quote! {
                #func(self);
            }
        }

        // a block was provided
        // #[deser({ self.z = 0xFFFF })]
        AttrKind::Block(block) => quote!(#block),
    }
}

fn process_attr(attr: &Attribute) -> AttrKind {
    // outer attribute only
    if attr.style != AttrStyle::Outer {
        unimplemented!("attribute {:?} is not on outer attribute", attr);
    }

    // the only attribute we process is "deser"
    if !attr.path().is_ident("deser") {
        unimplemented!("only #[deser] is a valid attribute");
    }

    let mut kind = AttrKind::default();

    let _ = attr.parse_nested_meta(|meta| {
        // #[deser(no)]
        if meta.path.is_ident("no") {
            kind = AttrKind::NoAction;
            return Ok(());
        }

        // #[deser(with_fn(function))]
        if meta.path.is_ident("with_fn") {
            if meta.input.peek(token::Paren) {
                let content;
                parenthesized!(content in meta.input);

                let function: Ident = content.parse()?;
                kind = AttrKind::Call(function);

                return Ok(());
            }

            unimplemented!("malformed deser(with_fn) attribute")
        }

        // #[deser(with_block({ let x = 9; }))]
        if meta.path.is_ident("with_code") {
            if meta.input.peek(token::Paren) {
                let content;
                parenthesized!(content in meta.input);

                let block: proc_macro2::TokenStream = content.parse()?;
                kind = AttrKind::Block(block);

                return Ok(());
            }

            unimplemented!("deser attribute meta not supported")
        }

        Err(meta.error("unrecognized deser attribute"))
    });

    kind
}
