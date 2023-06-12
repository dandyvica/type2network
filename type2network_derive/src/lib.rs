//use syn::visit::{self, Visit};
use syn::{parse_macro_input, Data, DeriveInput};

use proc_macro::TokenStream;
//use quote::{quote, ToTokens};

mod struct_builder;
use struct_builder::{StructDeriveBuilder, StructBuilder};

mod enum_builder;
use enum_builder::EnumDeriveBuilder;


#[proc_macro_derive(ToNetwork)]
pub fn to_network(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let code: proc_macro2::TokenStream = match &ast.data {
        Data::Enum(de) => EnumDeriveBuilder::to_network(&ast, de),
        Data::Struct(ds) => StructDeriveBuilder::to_network(&ast, ds),
        _ => unimplemented!("{} is neither a struct, nor an enum", ast.ident.to_string()),
    };

    // println!("code ============> '{}'", code);

    // quote!(#code).into()
    //derive_helper(input, StructDeriveBuilder::to_network)
    code.into()
}

#[proc_macro_derive(FromNetwork)]
pub fn from_network(input: TokenStream) -> TokenStream {
    // let ast = parse_macro_input!(input as DeriveInput);

    // let code = match &ast.data {
    //     Data::Enum(_) => unimplemented!("{} is not a struct", ast.ident.to_string()),
    //     Data::Struct(ds) => StructDeriveBuilder::from_network(&ast, ds),
    //     _ => unimplemented!("{} is neither a struct, nor an enum", ast.ident.to_string()),
    // };

    // println!("code ============> '{}'", code);

    // quote!(#code).into()

    derive_helper(input, StructDeriveBuilder::from_network)
}

fn derive_helper(input: TokenStream, struct_builder: StructBuilder) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let code: proc_macro2::TokenStream = match &ast.data {
        Data::Enum(_) => unimplemented!("{} is not a struct", ast.ident.to_string()),
        Data::Struct(ds) => struct_builder(&ast, ds),
        _ => unimplemented!("{} is neither a struct, nor an enum", ast.ident.to_string()),
    };

    //println!("code ============> '{}'", code);

    code.into()
}

// #[proc_macro_derive(FromNetwork)]
// pub fn from_network(input: TokenStream) -> TokenStream {
//     let ast = parse_macro_input!(input as DeriveInput);
//     let visitor = AstVisitor::new(&ast);

//     let code = match &ast.data {
//         Data::Enum(_) => inject_enum_from_network(&visitor),
//         Data::Struct(_) => inject_struct_from_network(&visitor),
//         _ => unimplemented!(
//             "{} is neither a struct, nor an enum",
//             visitor.name.unwrap().to_string()
//         ),
//     };

//     //println!("{}", code);

//     quote!(#code).into()
// }
