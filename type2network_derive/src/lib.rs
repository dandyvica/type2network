use syn::{parse_macro_input, Data, DeriveInput};

use proc_macro::TokenStream;

mod r#struct;
use r#struct::{StructBuilderFunc, StructDeriveBuilder};

mod r#enum;
use r#enum::{EnumBuilderFunc, EnumDeriveBuilder};

mod syn_utils;

#[proc_macro_derive(ToNetwork)]
pub fn to_network(input: TokenStream) -> TokenStream {
    derive_helper(
        input,
        Some(EnumDeriveBuilder::to_network),
        StructDeriveBuilder::to_network,
        false,
    )
}

#[proc_macro_derive(FromNetwork, attributes(from_network))]
pub fn from_network(input: TokenStream) -> TokenStream {
    derive_helper(
        input,
        Some(EnumDeriveBuilder::from_network),
        StructDeriveBuilder::from_network,
        false,
    )
}

fn derive_helper(
    input: TokenStream,
    enum_builder: Option<EnumBuilderFunc>,
    struct_builder: StructBuilderFunc,
    print_code: bool,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let code: proc_macro2::TokenStream = match &ast.data {
        Data::Enum(de) => match enum_builder {
            Some(eb) => eb(&ast, de),
            None => unimplemented!("{} is not a struct", ast.ident.to_string()),
        },
        Data::Struct(ds) => struct_builder(&ast, ds),
        _ => unimplemented!("{} is neither a struct, nor an enum", ast.ident.to_string()),
    };

    if print_code {
        println!("{}", code);
    }

    code.into()
}
