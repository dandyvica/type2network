use proc_macro2::Span;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, Ident, Variant};

pub struct EnumDeriveBuilder;

impl EnumDeriveBuilder {
    pub fn to_network(ast: &DeriveInput, de: &DataEnum) -> proc_macro2::TokenStream {
        let enum_name = &ast.ident;

        println!("is_unit_only={}", EnumDeriveBuilder::is_unit_only(ast, de));

        let arms = de
            .variants
            .iter()
            .map(|v| EnumDeriveBuilder::build_variant_arm(enum_name, v));

        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        let code = quote! {
            impl #impl_generics ToNetworkOrder for #enum_name #ty_generics #where_clause {
                fn to_network_order(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
                    match self {
                        #( #arms)*
                    }
                }
            }
        };

        println!("{}", code);

        code
    }

    // Test whether all enum variant are unit
    fn is_unit_only(_ast: &DeriveInput, de: &DataEnum) -> bool {
        de.variants.iter().all(|v| matches!(v.fields, Fields::Unit))
    }

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
    //        length += ToNetworkOrder ::to_network_order(f0, buffer)?;
    //        length += ToNetworkOrder ::to_network_order(f1, buffer)?;
    //        length += ToNetworkOrder ::to_network_order(f2, buffer)?;
    //        Ok(length)
    // },
    fn build_variant_arm(enum_name: &Ident, variant: &Variant) -> proc_macro2::TokenStream {
        let variant_ident = &variant.ident;

        match &variant.fields {
            // unnamed variant like: ChangeColor(i32, i32, i32)
            Fields::Unnamed(_) => {
                let field_names = (0..variant.fields.len())
                    .map(|i| Ident::new(&format!("f{}", i), Span::call_site()));

                let method_calls = field_names.clone().map(|f| {
                    quote! {
                        length += ToNetworkOrder::to_network_order(#f, buffer)?;
                    }
                });

                quote! {
                    #enum_name::#variant_ident(#(#field_names),*) => {
                        let mut length = 0usize;
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
                        length += ToNetworkOrder::to_network_order(#f, buffer)?;
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
                quote! {
                    #enum_name::#variant_ident => {
                        let value = #enum_name::#variant_ident;
                        let size = std::mem::size_of_val(&value);
                        match size {
                            1 => buffer.write_u8(value as u8)?,
                            2 => buffer.write_u16::<BigEndian>(value as u16)?,
                            4 => buffer.write_u32::<BigEndian>(value as u32)?,
                            8 => buffer.write_u64::<BigEndian>(value as u64)?,
                            _ => unimplemented!("size of variant is not supported"),
                        }

                        Ok(size)
                    },
                }
            }
        }
    }
}
