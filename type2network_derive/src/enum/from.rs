use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, DataEnum, DeriveInput};

use crate::syn_utils::*;

use super::EnumDeriveBuilder;

impl EnumDeriveBuilder {
    pub fn from_network(ast: &DeriveInput, _de: &DataEnum) -> proc_macro2::TokenStream {
        let enum_name = &ast.ident;
        let enum_string = enum_name.to_string();

        // which trait does the enum implement ? From or TryFrom or none of these ?
        let implemented_trait = get_from_or_tryfrom(&ast.attrs);

        let ty = SynUtils::repr_size(&ast.attrs)
            .unwrap_or_else(|| unimplemented!("repr size is mandatory on enum {}", enum_name));

        let value_expr = build_value(&ty);

        // the implementation of FromNetworkOrder depends on whether From or TryFrom is implemented
        match implemented_trait {
            TryFromOrFrom::From => quote! {
                impl<'fromnet> FromNetworkOrder<'fromnet> for #enum_name {
                    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'fromnet [u8]>) -> std::io::Result<()> {
                        #value_expr
                        *self = <#enum_name>::from(value);
                        Ok(())
                    }
                }
            },
            TryFromOrFrom::TryFrom => quote! {
                impl<'fromnet> FromNetworkOrder<'fromnet> for #enum_name {
                    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'fromnet [u8]>) -> std::io::Result<()> {
                        #value_expr
                        match <#enum_name>::try_from(value) {
                            Ok(ct) => {
                                *self = ct;
                                Ok(())
                            }
                            _ => Err(std::io::Error::other(format!("error converting value '{}' to enum type {}", value, #enum_string))),
                        }
                    }
                }
            },
            TryFromOrFrom::None => panic!(
                "at least, '{}' should implement From or TryFrom trait",
                enum_string
            ),
        }
    }
}

fn build_value(ty: &TokenStream) -> proc_macro2::TokenStream {
    match ty.to_string().as_str() {
        "u8" => quote!(let value = buffer.read_u8()?;),
        "i8" => quote!(let value = buffer.read_i8()?;),
        _ => {
            let method = format_ident!("read_{}", ty.to_string());
            quote!(let value = buffer.#method::<BigEndian>()?;)
        }
    }
}

// FromNetwork for enums makes it mandatory to impl either From or TryFrom
// This is hinted using the #[from_network(From)] ou #[from_network(TryFrom)] outer attribute
enum TryFromOrFrom {
    From,
    TryFrom,
    None,
}

fn get_from_or_tryfrom(attrs: &[Attribute]) -> TryFromOrFrom {
    let mut result = TryFromOrFrom::None;

    // loop through attributes
    for attr in attrs {
        // we found the #[tonetwork] attributes
        if attr.path().is_ident("from_network") {
            attr.parse_nested_meta(|meta| {
                // #[from_network(From)]
                if meta.path.is_ident("From") {
                    result = TryFromOrFrom::From;
                    return Ok(());
                }

                // #[from_network(TryFrom)]
                if meta.path.is_ident("TryFrom") {
                    result = TryFromOrFrom::TryFrom;
                    return Ok(());
                }

                // neither From nor TryFrom was found
                Ok(())
            })
            .unwrap();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value() {
        let ty = quote!(u8);
        let res = build_value(&ty).to_string();
        assert_eq!(res, "let value = buffer . read_u8 () ? ;");

        let ty = quote!(u64);
        let res = build_value(&ty).to_string();
        assert_eq!(res, "let value = buffer . read_u64 :: < BigEndian > () ? ;");
    }
}
