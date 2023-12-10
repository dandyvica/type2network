//! All functions/trait to convert DNS structures to network order back & forth
use bytes::{Bytes, BytesMut};
use either::*;

use crate::{FromNetworkOrder, ToNetworkOrder};

impl<L, R> ToNetworkOrder for Either<L, R>
where
    L: ToNetworkOrder,
    R: ToNetworkOrder,
{
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
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        let length = match &self {
            Either::Left(l) => l.serialize_to(buffer)?,
            Either::Right(r) => r.serialize_to(buffer)?,
        };

        Ok(length)
    }
}

impl ToNetworkOrder for Bytes {
    /// ```
    /// use bytes::Bytes;
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let v = Bytes::from(vec![0u8,1,2,3]);
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 4);
    /// assert_eq!(&buffer, &[0,1,2,3]);
    /// ```    
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        buffer.extend(self.iter());

        Ok(self.len())
    }
}

impl<'a> FromNetworkOrder<'a> for BytesMut {
    /// ```
    /// use std::io::Cursor;
    /// use bytes::BytesMut;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v = BytesMut::with_capacity(4);
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v, vec![0x12, 0x34, 0x56, 0x78]);
    /// ```
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        self.extend_from_slice(buffer.get_ref());

        Ok(())
    }
}
