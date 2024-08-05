//! All functions/trait to convert DNS structures to network order back & forth
use std::io::{Read, Write};

use bytes::{Bytes, BytesMut};
use either::*;

use crate::{FromNetworkOrder, ToNetworkOrder};

impl<L, R, W: Write> ToNetworkOrder<W> for Either<L, R>
where
    L: ToNetworkOrder<W>,
    R: ToNetworkOrder<W>,
{
    /// Example:    
    /// ```
    /// use either::*;
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let v: Either<Vec<u8>,()> = Left(vec![0u8,1,2,3]);
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 4);
    /// assert_eq!(&buffer, &[0,1,2,3]);
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let v: Either<(), Vec<u8>> = Right(vec![0u8,1,2,3]);
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 4);
    /// assert_eq!(&buffer, &[0,1,2,3]);
    /// ```      
    fn serialize_to(&self, buffer: &mut W) -> std::io::Result<usize> {
        let length = match &self {
            Either::Left(l) => l.serialize_to(buffer)?,
            Either::Right(r) => r.serialize_to(buffer)?,
        };

        Ok(length)
    }
}

impl<W: Write> ToNetworkOrder<W> for Bytes {
    /// Example:
    /// ```
    /// use bytes::Bytes;
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let v = Bytes::from(vec![0u8,1,2,3]);
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 4);
    /// assert_eq!(&buffer, &[0,1,2,3]);
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 4);
    /// assert_eq!(&buffer, &[0,1,2,3,0,1,2,3]);
    /// ```
    fn serialize_to(&self, buffer: &mut W) -> std::io::Result<usize> {
        _ = buffer.write(self.as_ref());

        Ok(self.len())
    }
}

// impl<'a, R:Read> FromNetworkOrder<'a, R> for BytesMut {
//     /// Example:
//     /// ```
//     /// use std::io::Cursor;
//     /// use bytes::BytesMut;
//     /// use type2network::FromNetworkOrder;
//     ///
//     /// let b = vec![0x12, 0x34, 0x56, 0x78];
//     /// let mut buffer = Cursor::new(b.as_slice());
//     /// let mut v = BytesMut::with_capacity(4);
//     /// assert!(v.deserialize_from(&mut buffer).is_ok());
//     /// assert_eq!(v, vec![0x12, 0x34, 0x56, 0x78]);
//     /// ```
//     fn deserialize_from(&mut self, buffer: &mut R) -> std::io::Result<()> {
//         self.extend_from_slice(buffer.bytes().by_ref());

//         Ok(())
//     }
// }
