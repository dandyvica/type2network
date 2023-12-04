use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataEnum, DeriveInput};

use syn_utils::*;

use super::EnumDeriveBuilder;

impl EnumDeriveBuilder {
    pub fn from_network(ast: &DeriveInput, _de: &DataEnum) -> proc_macro2::TokenStream {
        let enum_name = &ast.ident;
        let enum_string = enum_name.to_string();
        let ty = SynUtils::repr_size(&ast.attrs)
            .unwrap_or_else(|| unimplemented!("repr size is mandatory on enum {}", enum_name));

        let value_expr = build_value(&ty);

        quote! {
            impl<'a> FromNetworkOrder<'a> for #enum_name {
                fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
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
