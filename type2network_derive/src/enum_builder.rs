use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, DataEnum, DeriveInput, Fields, Ident, Variant};

pub struct EnumDeriveBuilder;
pub type EnumBuilder = fn(&DeriveInput, &DataEnum) -> proc_macro2::TokenStream;

impl EnumDeriveBuilder {
    pub fn to_network(ast: &DeriveInput, de: &DataEnum) -> proc_macro2::TokenStream {
        let enum_name = &ast.ident;
        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        // if all variants are unit, the serialize_from() method is straightforward
        let code = if de.variants.iter().all(|x| x.fields == Fields::Unit) {
            let code_to = process_attr_to(enum_name, &ast.attrs);

            quote! {
                impl #impl_generics ToNetworkOrder for #enum_name #ty_generics #where_clause {
                    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
                        #code_to
                    }
                }
            }
        } else {
            let arms = de
                .variants
                .iter()
                .map(|v| EnumDeriveBuilder::build_variant_arm(enum_name, v));

            quote! {
                impl #impl_generics ToNetworkOrder for #enum_name #ty_generics #where_clause {
                    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
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

    pub fn from_network(ast: &DeriveInput, _de: &DataEnum) -> proc_macro2::TokenStream {
        let enum_name = &ast.ident;
        let enum_string = enum_name.to_string();
        let value_expr = process_attr_from(enum_name, &ast.attrs);

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
    fn build_variant_arm(enum_name: &Ident, variant: &Variant) -> proc_macro2::TokenStream {
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
                quote! {
                    #enum_name::#variant_ident => {
                        let size = std::mem::size_of_val(self);
                        match size {
                            1 => buffer.write_u8(#enum_name::#variant_ident as u8)?,
                            2 => buffer.write_u16::<BigEndian>(#enum_name::#variant_ident as u16)?,
                            4 => buffer.write_u32::<BigEndian>(#enum_name::#variant_ident as u32)?,
                            8 => buffer.write_u64::<BigEndian>(#enum_name::#variant_ident as u64)?,
                            _ => unimplemented!("size of variant is not supported"),
                        }

                        Ok(size)
                    },
                }
            }
        }
    }
}

//process the #[repr] attribute for all different cases for the ToNetwork trait
fn process_attr_to(ident: &Ident, attrs: &[Attribute]) -> proc_macro2::TokenStream {
    let mut ty = proc_macro2::TokenStream::default();

    for attr in attrs {
        if attr.path().is_ident("repr") {
            let _ = attr.parse_nested_meta(|meta| {
                // #[repr(u8)]
                if meta.path.is_ident("u8") {
                    ty = quote!(buffer.write_u8(*self as u8)?; Ok(1));
                    return Ok(());
                }
                // #[repr(u16)]
                if meta.path.is_ident("u16") {
                    ty = quote!(buffer.write_u16::<BigEndian>(*self as u16)?; Ok(2));
                    return Ok(());
                }
                // #[repr(u32)]
                if meta.path.is_ident("u32") {
                    ty = quote!(buffer.write_u32::<BigEndian>(*self as u32)?; Ok(4));
                    return Ok(());
                }
                // #[repr(u64)]
                if meta.path.is_ident("u64") {
                    ty = quote!(buffer.write_u64::<BigEndian>(*self as u64)?; Ok(8));
                    return Ok(());
                }
                // #[repr(i8)]
                if meta.path.is_ident("i8") {
                    ty = quote!(buffer.write_i8(*self as i8)?; Ok(1));
                    return Ok(());
                }
                // #[repr(i16)]
                if meta.path.is_ident("i16") {
                    ty = quote!(buffer.write_i16::<BigEndian>(*self as i16)?; Ok(2));
                    return Ok(());
                }
                // #[repr(i32)]
                if meta.path.is_ident("i32") {
                    ty = quote!(buffer.write_i32::<BigEndian>(*self as i32)?; Ok(4));
                    return Ok(());
                }
                // #[repr(i64)]
                if meta.path.is_ident("64") {
                    ty = quote!(buffer.write_i64::<BigEndian>(*self as iu64)?; Ok(8));
                    return Ok(());
                }

                unimplemented!("unsupported repr() in enum {}", ident.to_string());
            });
        }
    }

    ty
}

//process the #[repr] attribute for all different cases for the FromNetwork trait
fn process_attr_from(ident: &Ident, attrs: &[Attribute]) -> proc_macro2::TokenStream {
    let mut ty = proc_macro2::TokenStream::default();

    for attr in attrs {
        if attr.path().is_ident("repr") {
            let _ = attr.parse_nested_meta(|meta| {
                // #[repr(u8)]
                if meta.path.is_ident("u8") {
                    ty = quote!(let value = buffer.read_u8()?;);
                    return Ok(());
                }
                // #[repr(u16)]
                if meta.path.is_ident("u16") {
                    ty = quote!(let value = buffer.read_u16::<BigEndian>()?;);
                    return Ok(());
                }
                // #[repr(u32)]
                if meta.path.is_ident("u32") {
                    ty = quote!(let value = buffer.read_u32::<BigEndian>()?;);
                    return Ok(());
                }
                // #[repr(u64)]
                if meta.path.is_ident("u64") {
                    ty = quote!(let value = buffer.read_u64::<BigEndian>()?;);
                    return Ok(());
                }
                // #[repr(i8)]
                if meta.path.is_ident("i8") {
                    ty = quote!(let value = buffer.read_i8()?;);
                    return Ok(());
                }
                // #[repr(i16)]
                if meta.path.is_ident("i16") {
                    ty = quote!(let value = buffer.read_i16()?;);
                    return Ok(());
                }
                // #[repr(i32)]
                if meta.path.is_ident("i32") {
                    ty = quote!(let value = buffer.read_i32()?;);
                    return Ok(());
                }
                // #[repr(i64)]
                if meta.path.is_ident("64") {
                    ty = quote!(let value = buffer.read_64()?;);
                    return Ok(());
                }

                unimplemented!("unsupported repr() in enum {}", ident.to_string());
            });
        }
    }

    ty
}
