use quote::quote;
use syn::{
    parenthesized, token, AttrStyle, Attribute, DataStruct, DeriveInput, Field, Ident, Index,
};

use crate::{r#struct::is_unit, syn_utils::add_lifetime};

use super::{AttrKind, StructDeriveBuilder};

impl StructDeriveBuilder {
    pub fn from_network(ast: &DeriveInput, ds: &DataStruct) -> proc_macro2::TokenStream {
        // no code is created for unit structs
        if is_unit(ds) {
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

        // add lifetime specific to our trait ('a)
        let (_, ty_generics, where_clause) = ast.generics.split_for_impl();
        let gen_clone = add_lifetime(ast);
        let (new_impl_generics, _, _) = gen_clone.split_for_impl();

        quote! {
            impl #new_impl_generics FromNetworkOrder<'a> for #struct_name #ty_generics #where_clause {
                fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
                    #( #method_calls)*
                    Ok(())
                }
            }
        }
    }
}

// in case of a named field, process potential attribute and inject code
fn process_named_field(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();

    //find the attribute containing #[deser]
    let from_attr = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("from_network"));

    // analyze attribute
    let kind = if let Some(deser) = from_attr {
        process_attr(deser)
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

        // #[deser(ignore)]: don't do anything
        AttrKind::NoAction => quote!(),

        // a function was provided to the deser attribute: so just call it
        // e.g.: #[deser(my_func)]
        AttrKind::Call(func) => {
            quote! {
                #func(self)?;
            }
        }

        // a block was provided
        // #[deser({ self.z = 0xFFFF })]
        AttrKind::Block(block) => quote!(
            #block
            FromNetworkOrder::deserialize_from(&mut self.#field_name, buffer)?;
        ),

        // debug is requested
        // #[deser(debug)]
        AttrKind::Debug => quote!(
            FromNetworkOrder::deserialize_from(&mut self.#field_name, buffer)?;
            dbg!(self.#field_name);
        ),
    }
}

// process the #[from_network] attribute for all different cases
fn process_attr(attr: &Attribute) -> AttrKind {
    // outer attribute only
    if attr.style != AttrStyle::Outer {
        unimplemented!("attribute {:?} is not on outer attribute", attr);
    }

    // the only attribute we process is "deser"
    if !attr.path().is_ident("from_network") {
        unimplemented!("only #[from_network] is a valid attribute");
    }

    let mut kind = AttrKind::default();

    let _ = attr.parse_nested_meta(|meta| {
        // #[deser(ignore)]
        if meta.path.is_ident("ignore") {
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

            unimplemented!("malformed from_network(with_fn) attribute")
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

            unimplemented!("#from_network attribute meta not supported")
        }

        // #[deser(debug)]
        if meta.path.is_ident("debug") {
            kind = AttrKind::Debug;
            return Ok(());
        }

        Err(meta.error("unrecognized #from_network attribute"))
    });

    kind
}
