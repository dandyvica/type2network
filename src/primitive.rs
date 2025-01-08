//! All functions/trait to convert DNS structures to network order back & forth for
//! primitive types.
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Write;

use crate::{FromNetworkOrder, ToNetworkOrder};

// helper macro for boiler plate definitions
//#[macro_export]
macro_rules! impl_primitive {
    ($t:ty, $fw:path, $fr:path) => {
        impl ToNetworkOrder for $t {
            fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
                $fw(buffer, *self as $t)?;
                Ok(std::mem::size_of::<$t>())
            }
        }

        impl<'a> FromNetworkOrder<'a> for $t {
            fn deserialize_from(
                &mut self,
                buffer: &mut std::io::Cursor<&'a [u8]>,
            ) -> std::io::Result<()> {
                *self = $fr(buffer)?;
                Ok(())
            }
        }
    };
}

// signed
impl_primitive!(i8, WriteBytesExt::write_i8, ReadBytesExt::read_i8);
impl_primitive!(
    i16,
    WriteBytesExt::write_i16::<BigEndian>,
    ReadBytesExt::read_i16::<BigEndian>
);
impl_primitive!(
    i32,
    WriteBytesExt::write_i32::<BigEndian>,
    ReadBytesExt::read_i32::<BigEndian>
);
impl_primitive!(
    i64,
    WriteBytesExt::write_i64::<BigEndian>,
    ReadBytesExt::read_i64::<BigEndian>
);
impl_primitive!(
    i128,
    WriteBytesExt::write_i128::<BigEndian>,
    ReadBytesExt::read_i128::<BigEndian>
);

// unsigned
impl_primitive!(u8, WriteBytesExt::write_u8, ReadBytesExt::read_u8);
impl_primitive!(
    u16,
    WriteBytesExt::write_u16::<BigEndian>,
    ReadBytesExt::read_u16::<BigEndian>
);
impl_primitive!(
    u32,
    WriteBytesExt::write_u32::<BigEndian>,
    ReadBytesExt::read_u32::<BigEndian>
);
impl_primitive!(
    u64,
    WriteBytesExt::write_u64::<BigEndian>,
    ReadBytesExt::read_u64::<BigEndian>
);
impl_primitive!(
    u128,
    WriteBytesExt::write_u128::<BigEndian>,
    ReadBytesExt::read_u128::<BigEndian>
);

// // floats
impl_primitive!(
    f32,
    WriteBytesExt::write_f32::<BigEndian>,
    ReadBytesExt::read_f32::<BigEndian>
);
impl_primitive!(
    f64,
    WriteBytesExt::write_f64::<BigEndian>,
    ReadBytesExt::read_f64::<BigEndian>
);

impl ToNetworkOrder for char {
    /// ```char``` is serialized as 4 bytes.
    /// Example:
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer = Vec::new();;
    /// assert_eq!('💯'.serialize_to(&mut buffer).unwrap(), 4);
    /// assert_eq!(buffer, [0, 1, 244, 175]);
    ///
    /// buffer.clear();
    /// assert_eq!('a'.serialize_to(&mut buffer).unwrap(), 4);
    /// assert_eq!(buffer, [0, 0, 0, 97]);
    /// ```
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        let u = *self as u32;
        u.serialize_to(buffer)?;
        //println!("u={} buffer={:?}", u, buffer);
        Ok(std::mem::size_of::<char>())
    }
}

impl<'a> FromNetworkOrder<'a> for char {
    /// Example:    
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0, 1, 244, 175];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut c = char::default();
    /// assert!(c.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(c, '💯');
    /// ```
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        // convert first to u32
        let mut u = 0_u32;
        u.deserialize_from(buffer)?;
        *self = char::from_u32(u).unwrap();

        Ok(())
    }
}

impl ToNetworkOrder for [u8] {
    /// Example:    
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert!(&[0x12_u8, 0x34, 0x56, 0x78].serialize_to(&mut buffer).is_ok());
    /// assert_eq!(buffer, &[0x12, 0x34, 0x56, 0x78]);
    /// ```
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        _ = buffer.write(self.as_ref())?;
        Ok(self.len())
    }
}

// can't implement this
impl<'a> FromNetworkOrder<'a> for &'a [u8] {
    fn deserialize_from(&mut self, _buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        unimplemented!("the deserialize_from() method can't be implemented on &[u8]")
    }
}

impl<'a> ToNetworkOrder for &'a str {
    /// Example:    
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert!(&[0x12_u8, 0x34, 0x56, 0x78].serialize_to(&mut buffer).is_ok());
    /// assert_eq!(buffer, &[0x12, 0x34, 0x56, 0x78]);
    /// ```
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        _ = buffer.write(self.as_bytes())?;
        Ok(self.len())
    }
}

// a void implementation to ease integration test
impl<'a> FromNetworkOrder<'a> for &'a str {
    fn deserialize_from(&mut self, _buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        Ok(())
    }
}

impl ToNetworkOrder for String {
    /// Example:
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert!(String::from("I ❤ 東京").serialize_to(&mut buffer).is_ok());
    /// assert_eq!(buffer, &[73, 32, 226, 157, 164, 32, 230, 157, 177, 228, 186, 172]);
    /// ```    
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        _ = buffer.write(self.as_bytes())?;
        Ok(self.len())
    }
}

impl ToNetworkOrder for () {
    fn serialize_to(&self, _buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        Ok(0)
    }
}

// impl<'a> FromNetworkOrder<'a> for String {
//     fn deserialize_from<'a>(&mut self, buffer: &mut Cursor<&[u8]>) -> Result<(), Error> {
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

#[cfg(test)]
mod tests {
    use crate::test_helpers::{from_network_test, to_network_test};

    #[test]
    fn to_net() {
        // unsigned ints
        to_network_test(255_u8, 1, &[0xFF]);
        to_network_test(0x1234_u16, 2, &[0x12, 0x34]);
        to_network_test(0x12345678_u32, 4, &[0x12, 0x34, 0x56, 0x78]);
        to_network_test(
            0x1234567812345678_u64,
            8,
            &[0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78],
        );

        // floats
        to_network_test(std::f32::consts::PI, 4, &[0x40, 0x49, 0x0f, 0xdb]);
        to_network_test(
            std::f64::consts::PI,
            8,
            &[0x40, 0x09, 0x21, 0xfb, 0x54, 0x44, 0x2d, 0x18],
        );

        // char
        to_network_test('💯', 4, &[0, 1, 244, 175]);
    }

    #[test]
    fn from_net() {
        // unsigned ints
        from_network_test(None, 255_u8, &vec![0xFF]);
        from_network_test(None, 0x1234_u16, &vec![0x12, 0x34]);
        from_network_test(None, 0x12345678_u32, &vec![0x12, 0x34, 0x56, 0x78]);
        from_network_test(
            None,
            0x1234567812345678_u64,
            &vec![0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78],
        );

        // floats
        from_network_test(None, std::f32::consts::PI, &vec![0x40, 0x49, 0x0f, 0xdb]);
        from_network_test(
            None,
            std::f64::consts::PI,
            &vec![0x40, 0x09, 0x21, 0xfb, 0x54, 0x44, 0x2d, 0x18],
        );

        // char
        from_network_test(None, '💯', &vec![0, 1, 244, 175]);
    }
}
