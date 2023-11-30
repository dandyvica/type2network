use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

// some tests for structs
use type2network::{FromNetworkOrder, ToNetworkOrder};
use type2network_derive::{FromNetwork, ToNetwork};

use enum_from::EnumTryFrom;

// used for boiler plate unit tests for integers, floats etc
pub fn to_network_test<T: ToNetworkOrder>(val: &T, size: usize, v: &[u8]) {
    let mut buffer: Vec<u8> = Vec::new();
    assert_eq!(val.serialize_to(&mut buffer).unwrap(), size);
    assert_eq!(buffer, v);
}

pub fn from_network_test<'a, T>(def: Option<T>, val: &T, buf: &'a Vec<u8>)
where
    T: FromNetworkOrder<'a> + Default + std::fmt::Debug + std::cmp::PartialEq,
{
    let mut buffer = std::io::Cursor::new(buf.as_slice());
    let mut v: T = if def.is_none() {
        T::default()
    } else {
        def.unwrap()
    };
    assert!(v.deserialize_from(&mut buffer).is_ok());
    assert_eq!(&v, val);
}

#[test]
#[allow(dead_code)]
fn struct_unit() {
    #[derive(ToNetwork, FromNetwork)]
    struct Unit;
}

#[test]
fn struct_basic() {
    #[derive(Debug, Default, PartialEq, ToNetwork, FromNetwork)]
    struct PointStruct {
        x: u16,
        y: u16,
    }

    let pt = PointStruct {
        x: 0x1234,
        y: 0x5678,
    };
    to_network_test(&pt, 4, &[0x12, 0x34, 0x56, 0x78]);
    from_network_test(None, &pt, &vec![0x12, 0x34, 0x56, 0x78]);
}

#[test]
fn struct_tuple_basic() {
    #[derive(Debug, Default, PartialEq, ToNetwork, FromNetwork)]
    struct PointUnit(u16, u16);

    let pt = PointUnit(0x1234, 0x5678);
    to_network_test(&pt, 4, &[0x12, 0x34, 0x56, 0x78]);
    from_network_test(None, &pt, &vec![0x12, 0x34, 0x56, 0x78]);
}

#[test]
fn struct_one_typeparam() {
    #[derive(Debug, Default, PartialEq, ToNetwork, FromNetwork)]
    struct PointTypeParam<T>
    where
        T: ToNetworkOrder + for<'b> FromNetworkOrder<'b>,
    {
        x: T,
        y: T,
    }

    let pt = PointTypeParam::<u16> {
        x: 0x1234,
        y: 0x5678,
    };
    to_network_test(&pt, 4, &[0x12, 0x34, 0x56, 0x78]);
    from_network_test(None, &pt, &vec![0x12, 0x34, 0x56, 0x78]);
}

#[test]
fn struct_lifetime_to() {
    #[derive(Debug, PartialEq, ToNetwork)]
    struct DataLifeTime<'a, T, V>
    where
        T: ToNetworkOrder,
        V: ToNetworkOrder,
    {
        x: &'a str,
        y: T,
        z: Option<V>,
    }

    let pt = DataLifeTime::<Option<u16>, Vec<Option<u8>>> {
        x: &"\x01\x02\x03\x04\x05\x06\x07\x08",
        y: Some(0x090A),
        z: Some(vec![None, Some(0x0B), Some(0x0C)]),
    };
    to_network_test(
        &pt,
        12,
        &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
        ],
    );
}

#[test]
fn struct_lifetime_from() {
    #[derive(Debug, PartialEq, FromNetwork)]
    struct DataLifeTimeWithTypeParam<'a, T, V>
    where
        T: FromNetworkOrder<'a>,
        V: FromNetworkOrder<'a>,
    {
        x: &'a str,
        y: T,
        z: Option<V>,
    }
}

#[test]
fn struct_rr() {
    // A generic RR structure which could be use with OPT too
    #[derive(Debug, Default, ToNetwork)]
    pub struct DNSRR<T, U, V>
    where
        T: ToNetworkOrder,
        U: ToNetworkOrder,
        V: ToNetworkOrder,
    {
        pub name: T, // an owner name, i.e., the name of the node to which this resource record pertains.
        pub r#type: u16, // two octets containing one of the RR TYPE codes.
        pub class: U, // two octets containing one of the RR CLASS codes or payload size in case of OPT
        pub ttl: u32, //   a bit = 32 signed (actually unsigned) integer that specifies the time interval
        // that the resource record may be cached before the source
        // of the information should again be consulted.  Zero
        // values are interpreted to mean that the RR can only be
        // used for the transaction in progress, and should not be
        // cached.  For example, SOA records are always distributed
        // with a zero TTL to prohibit caching.  Zero values can
        // also be used for extremely volatile data.
        pub rd_length: u16, // an unsigned 16 bit integer that specifies the length in octets of the RDATA field.
        pub r_data: Option<V>,
        //  a variable length string of octets that describes the
        //  resource.  The format of this information varies
        //  according to the TYPE and CLASS of the resource record.
    }
}

#[test]
#[allow(dead_code)]
fn enum_c_like() {
    #[derive(Copy, Clone, ToNetwork)]
    #[repr(u8)]
    enum Boolean {
        True,
        False,
    }

    let b = Boolean::True;
    to_network_test(&b, 1, &[0x00]);
}

#[test]
#[allow(dead_code)]
fn enum_simple() {
    #[derive(Copy, Clone, ToNetwork, FromNetwork, EnumTryFrom)]
    #[repr(u64)]
    enum Color {
        Black = 0,
        White = 1,
        Yellow = 3,
        Brown = 55,
    }

    // let c = Color::Brown;
    // to_network_test(&c, 8, &[0, 0, 0, 0, 0, 0, 0, 55]);
}

#[test]
#[allow(dead_code)]
fn enum_message() {
    #[derive(ToNetwork)]
    enum Message {
        Move { x: u16, y: u16 },
        Write(String),
        ChangeColor(u16, u16, u16),
    }

    let m = Message::Move {
        x: 0x1234,
        y: 0x5678,
    };
    to_network_test(&m, 4, &[0x12, 0x34, 0x56, 0x78]);

    let m = Message::ChangeColor(0x1234, 0x5678, 0x9ABC);
    to_network_test(&m, 6, &[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
}

#[test]
#[allow(dead_code)]
fn enum_lifetime() {
    #[derive(ToNetwork)]
    enum Question<'a> {
        Label(&'a str),
        Answer(String),
    }

    let m = Question::Label("this is my question");
    to_network_test(&m, 19, "this is my question".as_bytes());
}

#[test]
fn struct_attr_no() {
    #[derive(Debug, Default, PartialEq, FromNetwork)]
    struct PointAttrNo {
        x: u16,
        y: u16,

        // last field is not deserialized
        #[deser(no)]
        z: u16,
    }

    let pt = PointAttrNo {
        x: 0x1234,
        y: 0x5678,
        z: 0,
    };
    from_network_test(None, &pt, &vec![0x12, 0x34, 0x56, 0x78, 0x00, 0x00]);
}

#[test]
fn struct_attr_fn() {
    // this function will be called whenever the attrbiute is found
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

    let pt = PointFn {
        x: 0x1234,
        y: 0x5678,
        z: 3,
    };
    from_network_test(None, &pt, &vec![0x12, 0x34, 0x56, 0x78, 0x03, 0x00]);
}

#[test]
fn struct_attr_code() {
    #[derive(Debug, Default, PartialEq, FromNetwork)]
    struct PointCode {
        x: u16,
        y: u16,

        // last field is not deserialized
        #[deser(with_code(self.z = 0xFFFF;))]
        z: u16,
    }

    let pt = PointCode {
        x: 0x1234,
        y: 0x5678,
        z: 0xFFFF,
    };
    from_network_test(None, &pt, &vec![0x12, 0x34, 0x56, 0x78, 0xFF, 0xFF]);
}

#[test]
fn struct_debug() {
    #[derive(Debug, Default, PartialEq, ToNetwork, FromNetwork)]
    struct PointDebug {
        #[deser(debug)]
        x: u16,

        #[deser(debug)]
        y: u16,
    }
}
