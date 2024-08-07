use syn::{DataStruct, DeriveInput, Fields, Ident};

pub struct StructDeriveBuilder;
pub type StructBuilderFunc = fn(&DeriveInput, &DataStruct) -> proc_macro2::TokenStream;

#[derive(Debug, Default)]
enum AttrKind {
    // when no #[from_network] attribute is given
    #[default]
    NoAttribute,

    // #[from_network(ignore)]
    NoAction,

    // #[from_network(with_fn(my_func))]
    Call(Ident),

    // #[from_network(with_code( let v = Vec::new(); ))]
    Block(proc_macro2::TokenStream),

    // #[from_network(debug)]
    Debug,
}

// Test whether the struct is a unit struct
fn is_unit(ds: &DataStruct) -> bool {
    matches!(ds.fields, Fields::Unit)
}

pub mod from;
pub mod to;
