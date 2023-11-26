use proc_macro2::Span;
use syn::{DeriveInput, GenericParam, Generics, Lifetime, LifetimeParam};

// this helper function manage lifetimes and generic parameters for the FromNetworkOrder trait
pub fn process_generics(ast: &DeriveInput) -> Generics {
    // need to add the 'a lifetime using this trick. Ref. https://users.rust-lang.org/t/add-lifetime-to/97988/5
    let mut gen_clone = ast.generics.clone();
    let lt = Lifetime::new("'a", Span::mixed_site());
    let ltp = LifetimeParam::new(lt);

    //don't add the 'a lifetime if already there
    let is_a_present = ast.generics.lifetimes().find(|l| **l == ltp);

    if is_a_present.is_none() {
        gen_clone.params.push(GenericParam::from(ltp));
    }

    gen_clone
}
