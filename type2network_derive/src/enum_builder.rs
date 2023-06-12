use quote::{quote, format_ident, ToTokens};
use syn::{DataStruct, DeriveInput, Fields, Index, ItemStruct, DataEnum, Variant, Ident, FieldsNamed, FieldsUnnamed};

pub struct EnumDeriveBuilder;

impl EnumDeriveBuilder {
    pub fn to_network(ast: &DeriveInput, de: &DataEnum) -> proc_macro2::TokenStream {
        let name = &ast.ident;

        for v in de.variants.iter() {
           println!("ident={} variant={} len={}", name, v.ident, v.fields.len());

            explore_variant(v);

            //println!("discriminants **************> {:?}", v.discriminant);
        }

        // let method_calls = de.variants.iter().map(|var| {
        //     quote! {
        //         #name::#var(x) => ToNetworkOrder::to_network_order(x, buffer),
        //     }
        // });        


        quote!(
            // impl ToNetworkOrder for #name {
            //     fn to_network_order(&self, buffer: &mut Vec<u8>) -> Result<usize, Error> {
            //         match self {
            //             #( #method_calls)*
            //         }
            //     }
            // }
        )

        
    }





}

fn explore_variant(var: &Variant)  {

    println!("var={}", var.to_token_stream());
    let variant_code = var.to_token_stream();

    match &var.fields {
        Fields::Unnamed(unnamed) => println!("variant {} is unnamed", var.ident),
        Fields::Named(named) => {
            let method_calls = generate_named(named);
            let code = quote!(
                #variant_code => {
                    let mut length = 0usize;
                    #method_calls
                    Ok(length)
                }
            );
            println!("generated code = {}", code);
        },
        Fields::Unit => println!("variant {} is unit", var.ident),
    }
}

fn generate_named(named: &FieldsNamed) -> proc_macro2::TokenStream {
    println!("generate_named => {}", named.to_token_stream());

    let method_calls = named.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        quote! {
            length += ToNetworkOrder::to_network_order(#field_name, buffer)?;
        }
    });

    quote!(
        #( #method_calls)*
    )
}

fn generate_unnamed(unnamed: &FieldsUnnamed) -> proc_macro2::TokenStream {
    let method_calls = unnamed.unnamed.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        quote! {
            length += ToNetworkOrder::to_network_order(#field_name, buffer)?;
        }
    });

    quote!(
        #( #method_calls)*
    )
}