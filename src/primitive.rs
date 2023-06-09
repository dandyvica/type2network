//! All functions/trait to convert DNS structures to network order back & forth
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Error, ErrorKind, Result, Write};

use crate::{impl_primitive, FromNetworkOrder, ToNetworkOrder};

// define impl for signed integers
impl ToNetworkOrder for i8 {
    fn to_network_order<V: Write>(&self, buffer: &mut V) -> Result<usize> {
        buffer.write_i8(*self)?;
        Ok(1)
    }
}

impl FromNetworkOrder for i8 {
    fn from_network_order(&mut self, buffer: &mut Cursor<&[u8]>) -> Result<()> {
        *self = buffer.read_i8()?;
        Ok(())
    }
}

impl_primitive!(i16, write_i16, read_i16);
impl_primitive!(i32, write_i32, read_i32);
impl_primitive!(i64, write_i64, read_i64);
impl_primitive!(i128, write_i128, read_i128);

// define impl for unsigned integers
impl ToNetworkOrder for u8 {
    fn to_network_order<V: Write>(&self, buffer: &mut V) -> Result<usize> {
        buffer.write_u8(*self)?;
        Ok(1)
    }
}

impl FromNetworkOrder for u8 {
    fn from_network_order<'a>(&mut self, buffer: &mut Cursor<&[u8]>) -> Result<()> {
        *self = buffer.read_u8()?;
        Ok(())
    }
}

impl_primitive!(u16, write_u16, read_u16);
impl_primitive!(u32, write_u32, read_u32);
impl_primitive!(u64, write_u64, read_u64);
impl_primitive!(u128, write_u128, read_u128);

// floats
impl_primitive!(f32, write_f32, read_f32);
impl_primitive!(f64, write_f64, read_f64);

impl ToNetworkOrder for char {
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer = Vec::new();;
    /// assert_eq!('💯'.to_network_order(&mut buffer).unwrap(), 4);
    /// assert_eq!(buffer, [0, 1, 244, 175]);
    /// ```
    fn to_network_order<V: Write>(&self, buffer: &mut V) -> Result<usize> {
        let u = *self as u32;
        u.to_network_order(buffer)?;
        //println!("u={} buffer={:?}", u, buffer);
        Ok(std::mem::size_of::<char>())
    }
}

impl FromNetworkOrder for char {
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0, 1, 244, 175];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut c = char::default();
    /// assert!(c.from_network_order(&mut buffer).is_ok());
    /// assert_eq!(c, '💯');
    /// ```
    fn from_network_order<'a>(&mut self, buffer: &mut Cursor<&'a [u8]>) -> Result<()> {
        // convert first to u32
        let mut u = 0_u32;
        u.from_network_order(buffer)?;
        *self = char::from_u32(u).unwrap();

        Ok(())
    }
}

impl ToNetworkOrder for &[u8] {
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert!(&[0x12_u8, 0x34, 0x56, 0x78].to_network_order(&mut buffer).is_ok());
    /// assert_eq!(buffer, &[0x12, 0x34, 0x56, 0x78]);
    /// ```
    fn to_network_order<V: Write>(&self, buffer: &mut V) -> Result<usize> {
        buffer.write(&mut self.to_vec())?;
        Ok(self.len())
    }
}

impl<'a> ToNetworkOrder for &'a str {
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert!(&[0x12_u8, 0x34, 0x56, 0x78].to_network_order(&mut buffer).is_ok());
    /// assert_eq!(buffer, &[0x12, 0x34, 0x56, 0x78]);
    /// ```
    fn to_network_order<V: Write>(&self, buffer: &mut V) -> Result<usize> {
        buffer.write(&mut self.as_bytes().to_vec())?;
        Ok(self.len())
    }
}

impl ToNetworkOrder for String {
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert!(String::from("I ❤ 東京").to_network_order(&mut buffer).is_ok());
    /// assert_eq!(buffer, &[73, 32, 226, 157, 164, 32, 230, 157, 177, 228, 186, 172]);
    /// ```    
    fn to_network_order<V: Write>(&self, buffer: &mut V) -> Result<usize> {
        buffer.write(&mut self.as_bytes().to_vec())?;
        Ok(self.len())
    }
}

// impl<'a> FromNetworkOrder for String {
//     fn from_network_order<'a>(&mut self, buffer: &mut Cursor<&[u8]>) -> Result<()> {
//         // get a reference on [u8]
//         let position = buffer.position() as usize;
//         let inner_data = buffer.get_ref();

//         // first char is the string length
//         let length = inner_data[position] as u8;

//         // move the cursor forward
//         buffer.seek(SeekFrom::Current(length as i64))?;

//         // save data
//         let s = &buffer.get_ref()[position + 1..position + length as usize + 1];
//         let ss = std::str::from_utf8(s)?;
//         self.push_str(ss);

//         Ok(())
//     }
// }

impl<T: ToNetworkOrder> ToNetworkOrder for Option<T> {
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert_eq!(Some(0xFF_u8).to_network_order(&mut buffer).unwrap(), 1);
    /// assert_eq!(buffer, &[0xFF]);
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let r: Option<u8> = None;
    /// assert_eq!(r.to_network_order(&mut buffer).unwrap(), 0);
    /// assert!(buffer.is_empty());
    /// ```    
    fn to_network_order<V: Write>(&self, buffer: &mut V) -> Result<usize> {
        if self.is_none() {
            Ok(0)
        } else {
            self.as_ref().unwrap().to_network_order(buffer)
        }
    }
}

impl<T: FromNetworkOrder> FromNetworkOrder for Option<T> {
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v: Option<u32> = None;
    /// assert!(v.from_network_order(&mut buffer).is_ok());
    /// assert!(v.is_none());
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v: Option<u32> = Some(0u32);
    /// assert!(v.from_network_order(&mut buffer).is_ok());
    /// assert_eq!(v.unwrap(), 0x12345678);
    /// ```
    fn from_network_order<'a>(&mut self, buffer: &mut Cursor<&'a [u8]>) -> Result<()> {
        if self.is_none() {
            Ok(())
        } else {
            self.as_mut().unwrap().from_network_order(buffer)
        }
    }
}

impl<T: ToNetworkOrder, const N: usize> ToNetworkOrder for [T; N] {
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert_eq!([0xFFFF_u16; 10].to_network_order(&mut buffer).unwrap(), 20);
    /// assert_eq!(buffer, &[0xFF; 20]);
    /// ```    
    fn to_network_order<V: Write>(&self, buffer: &mut V) -> Result<usize> {
        let mut length = 0usize;
        let mut buf: Vec<u8> = Vec::new();

        for x in self {
            // first convert x to network bytes
            length += x.to_network_order(&mut buf)?;
            println!("array ToNetworkOrder length={}", length);

            buffer.write(&mut buf)?;
            buf.clear();
        }

        Ok(length)
    }
}

impl<T: FromNetworkOrder, const N: usize> FromNetworkOrder for [T; N] {
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v = [0u8;4];
    /// assert!(v.from_network_order(&mut buffer).is_ok());
    /// assert_eq!(v, [0x12_u8, 0x34, 0x56, 0x78]);
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v = [0u16;2];
    /// assert!(v.from_network_order(&mut buffer).is_ok());
    /// assert_eq!(v, [0x1234_u16, 0x5678]);
    /// ```
    fn from_network_order<'a>(&mut self, buffer: &mut Cursor<&'a [u8]>) -> Result<()> {
        for x in self {
            x.from_network_order(buffer)?;
        }
        Ok(())
    }
}

impl<T> ToNetworkOrder for Vec<T>
where
    T: ToNetworkOrder,
{
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let v = vec![[0xFFFF_u16;3],[0xFFFF;3],[0xFFFF;3]];
    /// assert_eq!(v.to_network_order(&mut buffer).unwrap(), 18);
    /// assert_eq!(&buffer, &[0xFF; 18]);
    /// ```
    fn to_network_order<V: Write>(&self, buffer: &mut V) -> Result<usize> {
        let mut length = 0usize;

        // copy data for each element
        for item in self {
            length += item.to_network_order(buffer)?;
        }

        Ok(length)
    }
}

impl<T> FromNetworkOrder for Vec<T>
where
    T: FromNetworkOrder,
{
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v: Vec<u16> = vec![0_u16;2];
    /// assert!(v.from_network_order(&mut buffer).is_ok());
    /// assert_eq!(v, &[0x1234_u16, 0x5678]);
    /// ```
    fn from_network_order<'a>(&mut self, buffer: &mut Cursor<&'a [u8]>) -> Result<()> {
        for item in self {
            item.from_network_order(buffer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{from_network_helper, to_network_helper};

    #[test]
    fn to_net() {
        // unsigned ints
        to_network_helper(255_u8, 1, &[0xFF]);
        to_network_helper(0x1234_u16, 2, &[0x12, 0x34]);
        to_network_helper(0x12345678_u32, 4, &[0x12, 0x34, 0x56, 0x78]);
        to_network_helper(
            0x1234567812345678_u64,
            8,
            &[0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78],
        );

        // floats
        to_network_helper(std::f32::consts::PI, 4, &[0x40, 0x49, 0x0f, 0xdb]);
        to_network_helper(
            std::f64::consts::PI,
            8,
            &[0x40, 0x09, 0x21, 0xfb, 0x54, 0x44, 0x2d, 0x18],
        );

        // char
        to_network_helper('💯', 4, &[0, 1, 244, 175]);
    }

    #[test]
    fn from_net() {
        // unsigned ints
        from_network_helper(None, 255_u8, &vec![0xFF]);
        from_network_helper(None, 0x1234_u16, &vec![0x12, 0x34]);
        from_network_helper(None, 0x12345678_u32, &vec![0x12, 0x34, 0x56, 0x78]);
        from_network_helper(
            None,
            0x1234567812345678_u64,
            &vec![0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78],
        );

        // floats
        from_network_helper(None, std::f32::consts::PI, &vec![0x40, 0x49, 0x0f, 0xdb]);
        from_network_helper(
            None,
            std::f64::consts::PI,
            &vec![0x40, 0x09, 0x21, 0xfb, 0x54, 0x44, 0x2d, 0x18],
        );

        // char
        from_network_helper(None, '💯', &vec![0, 1, 244, 175]);
    }

    #[test]
    fn array() {
        // Array of 5 Option<u16>
        to_network_helper(
            [None, Some(0x1234_u16), None, Some(0x5678_u16), None],
            4,
            &[0x12_u8, 0x34, 0x56, 0x78],
        );

        from_network_helper(
            Some([Some(0_u16); 2]),
            [Some(0x1234_u16), Some(0x5678_u16)],
            &vec![0x12_u8, 0x34, 0x56, 0x78],
        );
    }

    #[test]
    fn vector() {
        // Vec of 5 Option<u16>
        let val = vec![
            [Some(0x1234_u16), Some(0x5678_u16)],
            [Some(0x2345_u16), Some(0x6789_u16)],
            [Some(0x3456_u16), Some(0x789A_u16)],
        ];
        to_network_helper(
            val,
            12,
            &[
                0x12_u8, 0x34, 0x56, 0x78, 0x23_u8, 0x45, 0x67, 0x89, 0x34_u8, 0x56, 0x78, 0x9A,
            ],
        );

        from_network_helper(
            Some(vec![[Some(0_u16); 2]; 3]),
            vec![
                [Some(0x1234_u16), Some(0x5678_u16)],
                [Some(0x2345_u16), Some(0x6789_u16)],
                [Some(0x3456_u16), Some(0x789A_u16)],
            ],
            &vec![
                0x12_u8, 0x34, 0x56, 0x78, 0x23, 0x45, 0x67, 0x89, 0x34, 0x56, 0x78, 0x9A,
            ],
        );
    }
}
