use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{DataEnum, DeriveInput, Fields, Ident, Variant};

use crate::syn_utils::*;

use super::EnumDeriveBuilder;

impl EnumDeriveBuilder {
    pub fn to_network(ast: &DeriveInput, de: &DataEnum) -> proc_macro2::TokenStream {
        let enum_name = &ast.ident;
        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        // we need the repr size to build the arms
        // get the type inside #[repr()]
        let ty = SynUtils::repr_size(&ast.attrs);

        // if all variants are unit, the serialize_from() method is straightforward
        let code = if de.variants.iter().all(|x| x.fields == Fields::Unit) {
            if ty.is_none() {
                unimplemented!("repr size is mandatory on enum {}", enum_name);
            }
            let code = build_unit_arms(&ty.unwrap());

            quote! {
                impl #impl_generics ToNetworkOrder<W: Write> for #enum_name #ty_generics #where_clause {
                    fn serialize_to(&self, buffer: &mut W) -> std::io::Result<usize> {
                        #code
                    }
                }
            }
        } else {
            // in this case of a mixed enum, no mandatory #[repr] attribute
            let arms = de
                .variants
                .iter()
                .map(|v| build_variant_arm(enum_name, v, &ty));

            quote! {
                impl #impl_generics ToNetworkOrder<W: Write> for #enum_name #ty_generics #where_clause {
                    fn serialize_to(&self, buffer: &mut W) -> std::io::Result<usize> {
                        let mut length = 0usize;
                        match self {
                            #( #arms)*
                        }
                    }
                }
            }
        };

        code
    }
}

fn build_unit_arms(ty: &TokenStream) -> proc_macro2::TokenStream {
    match ty.to_string().as_str() {
        "u8" => quote!(buffer.write_u8(*self as u8)?; Ok(1)),
        "i8" => quote!(buffer.write_i8(*self as i8)?; Ok(1)),
        _ => {
            let method = format_ident!("write_{}", ty.to_string());
            quote!(buffer.#method::<BigEndian>(*self as #ty)?; Ok(std::mem::size_of::<#ty>()))
        }
    }
}

fn build_arms_from_literal(lit: &TokenStream, ty: &TokenStream) -> proc_macro2::TokenStream {
    match ty.to_string().as_str() {
        "u8" => quote!(buffer.write_u8(#lit as u8)?; Ok(1)),
        "i8" => quote!(buffer.write_i8(#lit as i8)?; Ok(1)),
        _ => {
            let method = format_ident!("write_{}", ty.to_string());
            quote!(buffer.#method::<BigEndian>(#lit as #ty)?; Ok(std::mem::size_of::<#ty>()))
        }
    }
}

// Test whether all enum variant are unit
// fn is_unit_only(_ast: &DeriveInput, de: &DataEnum) -> bool {
//     de.variants.iter().all(|v| matches!(v.fields, Fields::Unit))
// }

// Build the code for each variant arm
// Ex: if enum is:
//
// #[repr(u8)]
// enum Message {
//     Ok = 0,
//     Quit = 1,
//     Move { x: u16, y: u16 },
//     Write(String),
//     ChangeColor(u16, u16, u16),
// }
//
// then this function will build the arm for the variant passed as the 2nd parameter.
// Ex:
//
// Message::ChangeColor(f0, f1, f2) => {
//        let mut length = 0usize ;
//        length += ToNetworkOrder ::serialize_to(f0, buffer)?;
//        length += ToNetworkOrder ::serialize_to(f1, buffer)?;
//        length += ToNetworkOrder ::serialize_to(f2, buffer)?;
//        Ok(length)
// },
fn build_variant_arm(
    enum_name: &Ident,
    variant: &Variant,
    ty: &Option<TokenStream>,
) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;

    match &variant.fields {
        // unnamed variant like: ChangeColor(i32, i32, i32)
        Fields::Unnamed(_) => {
            let field_names = (0..variant.fields.len())
                .map(|i| Ident::new(&format!("f{}", i), Span::call_site()));

            let method_calls = field_names.clone().map(|f| {
                quote! {
                    length += ToNetworkOrder::serialize_to(#f, buffer)?;
                }
            });

            quote! {
                #enum_name::#variant_ident(#(#field_names),*) => {
                    #( #method_calls)*
                    Ok(length)
                },
            }
        }
        // named variant like: Move { x: i32, y: i32 }
        Fields::Named(_) => {
            let members = variant.fields.iter().map(|f| &f.ident);

            let method_calls = members.clone().map(|f| {
                quote! {
                    length += ToNetworkOrder::serialize_to(#f, buffer)?;
                }
            });

            quote! {
                #enum_name::#variant_ident{ #(#members),* } => {
                    let mut length = 0usize;
                    #( #method_calls)*
                    Ok(length)
                },
            }
        }
        // unit variant like: Quit = 1
        Fields::Unit => {
            if ty.is_none() {
                unimplemented!("repr size is mandatory on enum {}", enum_name);
            }

            let lit = variant.literal();
            let code = build_arms_from_literal(&lit, ty.as_ref().unwrap());

            quote!(
                #enum_name::#variant_ident => { #code }
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_arms() {
        let ty = quote!(u8);
        let res = build_unit_arms(&ty).to_string();
        assert_eq!(res, "buffer . write_u8 (* self as u8) ? ; Ok (1)");

        let ty = quote!(u32);
        let res = build_unit_arms(&ty).to_string();
        assert_eq!(res, "buffer . write_u32 :: < BigEndian > (* self as u32) ? ; Ok (std :: mem :: size_of :: < u32 > ())");
    }
}
