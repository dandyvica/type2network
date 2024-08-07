// this will help managing enum variants
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    Attribute, DeriveInput, Expr, Fields, FieldsNamed, FieldsUnnamed, GenericParam,
    Generics, Lifetime, LifetimeParam, Variant,
};

#[allow(dead_code)]
pub(super) trait VariantHelper {
    // true if the variant is a unit-like variant
    // Ok = 1
    fn is_unit(&self) -> bool;

    // Some(_) if variant has named fields
    // Move {
    //     x: u16,
    //     y: u16,
    // },
    fn is_named(&self) -> Option<&FieldsNamed>;

    // Some(_) if variant has unnamed fields
    // ChangeColor(u16, u16, u16),
    fn is_unnamed(&self) -> Option<&FieldsUnnamed>;

    // check whether the variant has the attribute 'attr'
    fn has_attribute<'a>(&'a self, attr: &str) -> Option<&'a Attribute>;

    // get the literal value of a unit variant
    fn literal(&self) -> TokenStream;
}

impl VariantHelper for Variant {
    fn is_unit(&self) -> bool {
        self.fields == Fields::Unit
    }

    fn is_named(&self) -> Option<&FieldsNamed> {
        if let Fields::Named(f) = &self.fields {
            return Some(f);
        }
        None
    }

    fn is_unnamed(&self) -> Option<&FieldsUnnamed> {
        if let Fields::Unnamed(f) = &self.fields {
            return Some(f);
        }
        None
    }

    fn has_attribute<'a>(&'a self, attr: &str) -> Option<&'a Attribute> {
        self.attrs.iter().find(|a| a.path().is_ident(attr))
    }

    fn literal(&self) -> TokenStream {
        // extract the litteral value of the variant. Ex: Ok = 0
        let value = self.discriminant.as_ref().unwrap_or_else(|| {
            unimplemented!("discriminant for variant {} is not a litteral", self.ident)
        });

        value.1.to_token_stream()
    }
}

// gather all global function under this umbrella
pub(super) struct SynUtils;

impl SynUtils {
    // return the internals of the repr attribute
    // #[repr(u8)] => Some(u8)
    pub fn repr_size(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
        let mut ty = None;

        for attr in attrs {
            if attr.path().is_ident("repr") {
                if let Ok(expr) = attr.parse_args::<Expr>() {
                    ty = Some(expr.to_token_stream());
                }
            }
        }

        ty
    }
}

// this helper function manage lifetimes and generic parameters for the FromNetworkOrder trait
pub(super) fn add_lifetime(ast: &DeriveInput) -> Generics {
    // need to add the 'a lifetime using this trick. Ref. https://users.rust-lang.org/t/add-lifetime-to/97988/5
    let mut gen_clone = ast.generics.clone();
    let lt = Lifetime::new("'fromnet", Span::mixed_site());
    let ltp = LifetimeParam::new(lt);

    //don't add the 'a lifetime if already there
    let is_lt_present = ast.generics.lifetimes().find(|l| **l == ltp);

    if is_lt_present.is_none() {
        gen_clone.params.push(GenericParam::from(ltp));
    }

    gen_clone
}

// pub(super) enum TraitTypeParam {
//     Reader,
//     Writer,
// }

// // add the WRITER type param and bound to the generics coming from what's derived
// // to be able to build defintion like: impl <WRITER : std::io::Write> ToNetworkOrder<WRITER> for PointStruct <WRITER>
// pub(super) fn add_writer_param(
//     gen: &Generics,
//     param_type: TraitTypeParam,
//     add_lifetime: bool,
// ) -> Generics {
//     // need to clone because ast is not mutable when calling this function
//     let mut gen_clone = gen.clone();

//     // define type param WRITER
//     let tp = match param_type {
//         TraitTypeParam::Reader => {
//             let mut tp = TypeParam::from(Ident::new("READER", Span::mixed_site()));
//             tp.bounds.push(parse_quote!(std::io::Read));
//             tp
//         }
//         TraitTypeParam::Writer => {
//             let mut tp = TypeParam::from(Ident::new("WRITER", Span::mixed_site()));
//             tp.bounds.push(parse_quote!(std::io::Write));
//             tp
//         }
//     };

//     // add lifetime if requested. This is used for the FromNetworkOrder trait
//     if add_lifetime {
//         let lt = Lifetime::new("'fromnet", Span::mixed_site());
//         let ltp = LifetimeParam::new(lt);
//         gen_clone.params.push(GenericParam::from(ltp));
//     }

//     // add param
//     gen_clone.params.push(GenericParam::Type(tp));

//     gen_clone
// }

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn enum_opcode_reserved() {
        let e: syn::ItemEnum = parse_quote!(
            #[repr(u16)]
            enum OpCode {
                Query = 0,  //[RFC1035]
                IQuery = 1, // (Inverse Query, OBSOLETE)	[RFC3425]
                Status = 2, // [RFC1035]
                Unassigned = 3,
                Notify = 4, // [RFC1996]
                Update = 5, // [RFC2136]
                DOS = 6,    // DNS Stateful Operations (DSO)	[RFC8490]
            }
        );

        let repr_size = SynUtils::repr_size(&e.attrs);
        assert!(repr_size.is_some());
        assert_eq!(&repr_size.unwrap().to_string(), "u16");
    }

    #[test]
    fn enum_message() {
        let e: syn::ItemEnum = parse_quote!(
            enum Message {
                Ok = 0,

                #[foo]
                Quit = 1,
                Move {
                    x: u16,
                    y: u16,
                },
                Write(String),
                ChangeColor(u16, u16, u16),
            }
        );

        for v in e.variants.iter() {
            match v.ident.to_string().as_str() {
                "Ok" => {
                    assert!(v.is_unit());
                    assert!(v.has_attribute("foo").is_none());
                    assert_eq!(v.literal().to_string(), "0");
                }
                "Quit" => {
                    assert!(v.is_unit());
                    assert!(v.has_attribute("foo").is_some());
                    assert_eq!(v.literal().to_string(), "1");
                }
                "Move" => {
                    assert!(v.is_named().is_some());
                    assert!(!v.has_attribute("foo").is_some());
                }
                "Write" => {
                    assert!(v.is_unnamed().is_some());
                    assert!(!v.has_attribute("foo").is_some());
                }
                _ => (),
            }
        }
    }
}
