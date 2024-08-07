use syn::{DataEnum, DeriveInput};

pub struct EnumDeriveBuilder;
pub type EnumBuilderFunc = fn(&DeriveInput, &DataEnum) -> proc_macro2::TokenStream;

pub mod from;
pub mod to;
