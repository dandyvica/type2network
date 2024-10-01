//! Traits and procedural macros to convert structures and enums to `bigendian` data streams.
//!
//! It's used to send `struct` or `enum` data to the wire, or receive data from the wire.
//! The trait is defined as:
//!
//! ```rust
//! // function to convert to network order (bigendian)
//! pub trait ToNetworkOrder {
//!     // copy structure data to a network-order buffer
//!     fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize>;
//! }
//!
//! // function to convert from network order (big-endian)
//! pub trait FromNetworkOrder<'a> {
//!     // copy from a network-order buffer to a structure
//!     fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>)
//!         -> std::io::Result<()>;
//! }
//! ```
//!
//! It's extensively using the [```byteorder```](https://docs.rs/byteorder/latest/byteorder) crate in order to convert integers or floats
//! to a bigendian buffer of ```u8```.
//! It is compatible with other attributes like those provided by [```serde```](https://crates.io/crates/serde) crate.
//!
//! ## How to use it ?
//!
//! Just add :
//!
//! * ```#[derive(FromNetwork)]``` to auto-implement the ```ToNetworkOrder``` trait
//! * ```#[derive(ToNetwork)]``` to auto-implement the ```FromNetworkOrder``` trait
//! * ```#[derive(ToNetwork, FromNetwork)]``` to auto-implement the ```ToNetworkOrder``` & ```FromNetworkOrder``` traits
//!
//! The ```ToNetworkOrder``` trait is supported for all structs or enums containing supported primary types (see below for a list of supported types).
//!
//! The ```FromNetworkOrder``` trait is only supported for C-like unit-only enums or those having a fallback variant.
//! For the ```ToNetworkOrder``` trait on C-like enums, it needs to be ```Copy, Clone```.
//!
//! ## The ```#[from_network]``` field attribute
//! In addition it's possible to add a field attribute on a struct's field for the ```FromNetworkOrder``` trait:
//!
//! * ```#[from_network(ignore)]``` : the field is not deserialized.
//! * ```#[from_network(debug)]``` : a ```dbg!(self.field_name)``` statement is inserted after the field is being deserialized.
//! * ```#[from_network(with_fn(func))]``` : the function ```func(&mut self) -> std::io::Result<()>``` is called for that field.
//! * ```#[from_network(with_code(block))]``` : the ```code``` block is injected before the field is being deserialized.
//!
//! 
//! ## The ```#[from_network]``` enum attribute
//! Two types of enums are supported for the ```FromNetworkOrder``` trait:
//! 
//! * C-like enums: in that case, the `TryFrom` trait must be defined, and ```#[from_network(TryFrom)]``` must be added as an outer attribute
//! * C-like enums having in addition a catch all fallback value (refer to the `num_enum` crate). In that case, the `From` trait must be defined
//! 
//! and ```#[from_network(From)]``` must be added as an outer attribute
//! 
//! Refer to [integration test](https://github.com/dandyvica/type2network/blob/main/tests/integration_tests.rs) for examples.
//! 
//! 
//! 
//! ## List of supported types
//!
//! | Type    | ```ToNetwork``` | ```FromNetwork``` |
//! | -------- | ------- |------- |
//! | ```all integers & floats```  |yes    |yes|
//! | ```char``` | yes     |yes|
//! | ```&[u8]``` | yes     |no|
//! | ```&str``` | yes     |no|
//! | ```String``` | yes     |no|
//! | ```Option<T>``` | yes     |yes|
//! | ```Vec<T>``` | yes     |yes|
//! | ```Box<T>``` | yes     |yes|
//! | ```PhantomData<T>``` | yes     |yes|
//! | ```()``` | yes     |yes|
//! | ```Cell<T>``` | yes     |yes|
//! | ```OnceCell<T>``` | yes     |yes|
//! | ```RefCell<T>``` | yes     |yes|
//! | ```Box<dyn ToNetworkOrder>``` | yes     |no|
//! | ```Box<dyn FromNetworkOrder<'a>>``` | no     |yes|
//! | ```Ipv4Addr``` | yes     |yes|
//! | ```Ipv6Addr``` | yes     |yes|
//! | ```Either<L,R>``` | yes     |no|
//! | ```Bytes``` | yes     |no|
//! | ```BytesMut``` | no     |yes|
//! 
//! ## Examples
//!
//! ```ignore
//! // z field is not deserialized
//! #[derive(Debug, Default, PartialEq, FromNetwork)]
//! struct PointAttrNo {
//!     x: u16,
//!     y: u16,
//!
//!     // last field is not deserialized
//!     #[from_network(ignore)]
//!     z: u16,
//! }
//!
//! // this function will be called for z field
//! fn update(p: &mut PointFn) -> std::io::Result<()> {
//!     p.z = 3;
//! }
//!
//! #[derive(Debug, Default, PartialEq, FromNetwork)]
//! struct PointFn {
//!     x: u16,
//!     y: u16,
//!
//!     // last field is not deserialized
//!     #[from_network(with_fn(update))]
//!     z: u16,
//! }
//!
//! // the block: self.z = 0xFFFF; is injected
//! #[derive(Debug, Default, PartialEq, FromNetwork)]
//! struct PointCode {
//!     x: u16,
//!     y: u16,
//!
//!     // last field is not deserialized
//!     #[from_network(with_code(self.z = 0xFFFF;))]
//!     z: u16,
//! }
//! ```
//!
//!
//! ## Full examples
//! Two examples can be found in the ```examples``` directory:
//!
//! * [ntp](https://github.com/dandyvica/type2network/blob/main/examples/ntp.rs)
//! * [dns](https://github.com/dandyvica/type2network/blob/main/examples/dns.rs)
//!
//! In general, you should add the ```#[derive(ToNetwork, FromNetwork)]``` derive macros to benefit from the procedural macros which automatically convert structure to network order back and forth. They are included using:
//!
//! ```rust
//! use type2network::{FromNetworkOrder, ToNetworkOrder};
//! use type2network_derive::{FromNetwork, ToNetwork};
//! ```
//! 
//! ## Why not `bincode` or `serde` ?
//! The `serde` data model is not managing C-like enums. You have to tap into another crate called `serde_repr`.
//! In addition, using serde, you define a function, not a trait for your struct. As for bincoe, I found it too much
//! complicated for my needs. Last but not least, it was meant to enhance my understanding of proc macros.

/// Copy structured data to a network-order buffer. Could be used on a ```struct```, an ```enum```, but
/// not an ```union```.
pub trait ToNetworkOrder {
    /// Returns the number of bytes copied or an [`std::io::Error`] error if any.
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize>;
}

/// Copy data from a network-order buffer to structured data.
pub trait FromNetworkOrder<'a> {
    /// Copy data from a network-order buffer to structured data.
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()>;
}

// all definitions of serialize_to()/deserialize_from() for standard types
mod additional;
mod cell;
mod generics;
mod net;
mod primitive;

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use std::io::Cursor;

    // used for boiler plate unit tests for integers
    pub fn to_network_test<T: ToNetworkOrder>(val: T, size: usize, v: &[u8]) {
        let mut buffer: Vec<u8> = Vec::new();
        assert_eq!(val.serialize_to(&mut buffer).unwrap(), size);
        assert_eq!(buffer, v);
    }

    // used for boiler plate unit tests for integers, floats etc
    pub fn from_network_test<'a, T>(def: Option<T>, val: T, buf: &'a Vec<u8>)
    where
        T: FromNetworkOrder<'a> + Default + std::fmt::Debug + std::cmp::PartialEq,
    {
        let mut buffer = Cursor::new(buf.as_slice());
        let mut v: T = if def.is_none() {
            T::default()
        } else {
            def.unwrap()
        };
        assert!(v.deserialize_from(&mut buffer).is_ok());
        assert_eq!(v, val);
    }
}


