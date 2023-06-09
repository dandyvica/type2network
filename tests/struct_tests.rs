// some tests for structs
use type2network::{FromNetworkOrder, ToNetworkOrder};
use type2network_derive::ToNetwork;

pub fn to_network_helper<T: ToNetworkOrder>(val: T, size: usize, v: &[u8]) {
    let mut buffer: Vec<u8> = Vec::new();
    assert_eq!(val.to_network_order(&mut buffer).unwrap(), size);
    assert_eq!(buffer, v);
}

#[test]
fn struct_basic() {
    #[derive(ToNetwork)]
    struct Point {
        x: u16,
        y: u16,
    }

    let pt = Point {
        x: 0x1234,
        y: 0x5678,
    };
    to_network_helper(pt, 4, &[0x12, 0x34, 0x56, 0x78]);
}

#[test]
fn struct_tuple_basic() {
    #[derive(ToNetwork)]
    struct Point(u16, u16);

    let pt = Point(0x1234, 0x5678);
    to_network_helper(pt, 4, &[0x12, 0x34, 0x56, 0x78]);
}

#[test]
fn struct_one_typeparam() {
    #[derive(ToNetwork)]
    struct Point<T>
    where
        T: ToNetworkOrder,
    {
        x: T,
        y: T,
    }

    let pt = Point::<u16> {
        x: 0x1234,
        y: 0x5678,
    };
    to_network_helper(pt, 4, &[0x12, 0x34, 0x56, 0x78]);
}
