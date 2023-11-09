# type2network
Traits and procedural macros to convert structures and enums into bigendian data streams.

It's used to send struct or enum data to the wire, or receive them from the wire.
The trait is defined as:

```rust
// function to convert to network order (big-endian)
pub trait ToNetworkOrder {
    // copy structure data to a network-order buffer
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize>;
}

// function to convert from network order (big-endian)
pub trait FromNetworkOrder<'a> {
    // copy from a network-order buffer to a structure
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>)
        -> std::io::Result<()>;
}
```

It's using the ```byteorder``` crate in order to convert integers or floats to a BigEndian buffer of ```u8```.

# List of supported types

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
| ```Cell<T>``` | yes     |yes|
| ```OnceCell<T>``` | yes     |yes|
| ```RefCell<T>``` | yes     |yes|
| ```Box<dyn ToNetworkOrder>``` | yes     |no|
| ```Box<dyn FromNetworkOrder<'a>>``` | no     |yes|


# Use case: NTP protocol
As an example, we can use this crate to send NTP packets to get time from an NTP server. The documention of the (simple) NTP protocol can be found in
https://datatracker.ietf.org/doc/html/rfc4330.

First define the NTP packet structures to describe the protocol details:

```rust
#[derive(Debug, Default, ToNetwork, FromNetwork)]
struct Header {
    values: u8,  // includes Leap Indicator, Version Number & Mode
    stratum: u8,
    poll: u8,
    precision: u8,
}

#[derive(Debug, Default, ToNetwork, FromNetwork)]
struct Timestamp {
    seconds: u32,
    fraction: u32,
}

#[derive(Debug, Default, ToNetwork, FromNetwork)]
struct SNTPPacket {
    header: Header,
    root_delay: u32,
    root_dispersion: u32,
    ref_id: u32,
    ref_tms: Timestamp,
    orig_tms: Timestamp,
    recv_tms: Timestamp,
    xmit_tms: Timestamp,
    id: Option<u32>,
    digest: Option<u128>,
}
```

You should add the ```#[derive(ToNetwork, FromNetwork)]``` derives to benefit from the procedural macros which automatically convert structure to network order back and forth. They are included using:

```rust
use type2network::{FromNetworkOrder, ToNetworkOrder};
use type2network_derive::{FromNetwork, ToNetwork};
```

Both crates should be included in the ```Cargo.toml``` file.

The first step is to define and fill the NTP packet structure:

```rust
// fill client packet
let mut ntp = SNTPPacket::default();

// set version number to 4 and mode to 3 (client)
ntp.header.values = 4 << 3 | 3;
```

Then, before sending data through the wire, we need to convert structure into a buffer:

```rust
// serialize to send the SNTP packet through the wire using UDP
let mut buffer: Vec<u8> = Vec::new();
ntp.serialize_to(&mut buffer)?;   
```

and ready to be sent using an UDP socket:

```rust
let socket = UdpSocket::bind("0.0.0.0:0")?;

// send packet to destination (one of the free NTP servers)
let dest = "fr.pool.ntp.org:123";
socket.send_to(&buffer, dest)?;
```

and receive the request:

```rust
// now receive the request using a sufficient buffer
let mut recbuf = [0; 512];
let received = socket.recv(&mut recbuf)?;
let mut cursor = Cursor::new(&recbuf[..received]);

// get response
ntp.deserialize_from(&mut cursor)?;
println!("{:?}", ntp);
```