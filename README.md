[![Actions](https://github.com/dandyvica/siphash_c_d/actions/workflows/rust.yml/badge.svg)](https://github.com/dandyvica/siphash_c_d/actions/workflows/rust.yml)

# type2network
Traits and procedural macros to convert structures and enums into bigendian data streams.

It's used to send struct or enum data to the wire, or receive them from the wire.
The trait is defined as:

```rust
// function to convert to network order (big-endian)
pub trait ToNetworkOrder {
    // copy structure data to a network-order buffer
    fn serialize_to(&self, buffer: &mut W) -> std::io::Result<usize>;
}

// function to convert from network order (big-endian)
pub trait FromNetworkOrder<'a> {
    // copy from a network-order buffer to a structure
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>)
        -> std::io::Result<()>;
}
```

It's using the ```byteorder``` crate in order to convert integers or floats to a bigendian buffer of ```u8```. It is compatible with other attributes like those provided by ```serde```.

## How to use it ?

Just add :

* ```#[derive(FromNetwork)]``` to auto-implement the ```ToNetworkOrder``` trait
* ```#[derive(ToNetwork)]``` to auto-implement the ```FromNetworkOrder``` trait
* ```#[derive(ToNetwork, FromNetwork)]``` to auto-implement the ```ToNetworkOrder``` & ```FromNetworkOrder``` traits

The ```FromNetworkOrder``` is only supported for unit-like enums. For the ```ToNetworkOrder``` trait on unit-like enums, it needs to be ```Copy, Clone```.

## The #[deser] attribute
In addition it's possible to add a field attribute on a field for structs for the ```FromNetworkOrder``` trait:

* ```#[deser(ignore)]``` : the field is not deserialized.
* ```#[deser(debug)]``` : a ```dbg!(self.field_name)``` statement is inserted after the field is deserialized.
* ```#[deser(with_fn(func))]``` : the function ```func(&mut self) -> std::io::Result<()>``` is called for that field.
* ```#[deser(with_code(code))]``` : the ```code``` block is injected before the field is being deserialized.

Examples:

```rust
// z field is not deserialized
#[derive(Debug, Default, PartialEq, FromNetwork)]
struct PointAttrNo {
    x: u16,
    y: u16,

    // last field is not deserialized
    #[deser(ignore)]
    z: u16,
}

// this function will be called for z field
fn update(p: &mut PointFn) {
    p.z = 3;
}

#[derive(Debug, Default, PartialEq, FromNetwork)]
struct PointFn {
    x: u16,
    y: u16,

    // last field is not deserialized
    #[deser(with_fn(update))]
    z: u16,
}

// the block: self.z = 0xFFFF; is injected
#[derive(Debug, Default, PartialEq, FromNetwork)]
struct PointCode {
    x: u16,
    y: u16,

    // last field is not deserialized
    #[deser(with_code(self.z = 0xFFFF;))]
    z: u16,
}
```

## List of supported types

| Type    | ToNetwork | FromNetwork |
| -------- | ------- |------- |
| ```all integers & floats```  |yes    |yes|
| ```char``` | yes     |yes|
| ```&[u8]``` | yes     |no|
| ```&str``` | yes     |no|
| ```String``` | yes     |no|
| ```Option<T>``` | yes     |yes|
| ```Vec<T>``` | yes     |yes|
| ```Box<T>``` | yes     |yes|
| ```PhantomData<T>``` | yes     |yes|
| ```()``` | yes     |yes|
| ```Cell<T>``` | yes     |yes|
| ```OnceCell<T>``` | yes     |yes|
| ```RefCell<T>``` | yes     |yes|
| ```Box<dyn ToNetworkOrder>``` | yes     |no|
| ```Box<dyn FromNetworkOrder<'a>>``` | no     |yes|
| ```Ipv4Addr``` | yes     |yes|
| ```Ipv6Addr``` | yes     |yes|
| ```Either<L,R>``` | yes     |no|
| ```Bytes``` | yes     |no|
| ```BytesMut``` | no     |yes|

## Examples
Two examples can be found in the ```examples``` directory:

* [ntp](examples/ntp.rs)
* [dns](examples/dns.rs)

In general, you should add the ```#[derive(ToNetwork, FromNetwork)]``` derives to benefit from the procedural macros which automatically convert structure to network order back and forth. They are included using:

```rust
use type2network::{FromNetworkOrder, ToNetworkOrder};
use type2network_derive::{FromNetwork, ToNetwork};
```