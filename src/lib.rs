use std::io::{Read, Write};

//use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::error::Error;
//pub (crate) type Result

// function to convert to network order (big-endian)
pub trait ToNetworkOrder {
    // copy structure data to a network-order buffer
    fn to_network_order<W: Write>(&self, buffer: &mut W) -> Result<usize, Error>;
}

// function to convert from network order (big-endian)
pub trait FromNetworkOrder {
    // copy from a network-order buffer to a structure
    fn from_network_order<R: Read>(&mut self, v: &mut R) -> Result<(), Error>;
}

//all definitions of to_network_order()/from_network_order() for standard types
//pub mod composed;
pub mod cell;
pub mod error;
pub mod generics;
pub mod primitive;

// pub fn unit_only_to_network<W: Write, T>(&self, value: T, buffer: &mut W) -> Result<usize, Error> {
//     let size = std::mem::size_of_val(&value);
//     match size {
//         1 => buffer.write_u8(value as u8)?,
//         2 => buffer.write_u16::<BigEndian>(value as u16)?,
//         4 => buffer.write_u32::<BigEndian>(value as u32)?,
//         8 => buffer.write_u64::<BigEndian>(value as u64)?,
//         _ => todo!(),
//     }

//     Ok(size)
// }

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use std::io::Cursor;

    // used for boiler plate unit tests for integers
    pub fn to_network_test<T: ToNetworkOrder>(val: T, size: usize, v: &[u8]) {
        let mut buffer: Vec<u8> = Vec::new();
        assert_eq!(val.to_network_order(&mut buffer).unwrap(), size);
        assert_eq!(buffer, v);
    }

    // used for boiler plate unit tests for integers, floats etc
    pub fn from_network_test<'a, T>(def: Option<T>, val: T, buf: &'a Vec<u8>)
    where
        T: FromNetworkOrder + Default + std::fmt::Debug + std::cmp::PartialEq,
    {
        let mut buffer = Cursor::new(buf.as_slice());
        let mut v: T = if def.is_none() {
            T::default()
        } else {
            def.unwrap()
        };
        assert!(v.from_network_order(&mut buffer).is_ok());
        assert_eq!(v, val);
    }
}
