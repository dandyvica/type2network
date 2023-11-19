//! All functions/trait to convert DNS structures to network order back & forth
use std::io::Write;
use std::marker::PhantomData;

use crate::{FromNetworkOrder, ToNetworkOrder};

impl<T: ToNetworkOrder> ToNetworkOrder for Option<T> {
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert_eq!(Some(0xFF_u8).serialize_to(&mut buffer).unwrap(), 1);
    /// assert_eq!(buffer, &[0xFF]);
    /// 
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let r: Option<u8> = None;
    /// assert_eq!(r.serialize_to(&mut buffer).unwrap(), 0);
    /// assert!(buffer.is_empty());
    /// ```    
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        if self.is_none() {
            Ok(0)
        } else {
            self.as_ref().unwrap().serialize_to(buffer)
        }
    }
}

impl<'a, T: FromNetworkOrder<'a>> FromNetworkOrder<'a> for Option<T> {
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v: Option<u32> = None;
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert!(v.is_none());
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v: Option<u32> = Some(0u32);
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v.unwrap(), 0x12345678);
    /// ```
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        if self.is_none() {
            Ok(())
        } else {
            self.as_mut().unwrap().deserialize_from(buffer)
        }
    }
}

impl<T: ToNetworkOrder, const N: usize> ToNetworkOrder for [T; N] {
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// assert_eq!([0xFFFF_u16; 10].serialize_to(&mut buffer).unwrap(), 20);
    /// assert_eq!(buffer, &[0xFF; 20]);
    /// ```    
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut length = 0usize;
        let mut buf: Vec<u8> = Vec::new();

        for x in self {
            // first convert x to network bytes
            length += x.serialize_to(&mut buf)?;

            _ = buffer.write(&buf)?;
            buf.clear();
        }

        Ok(length)
    }
}

impl<'a, T: FromNetworkOrder<'a>, const N: usize> FromNetworkOrder<'a> for [T; N] {
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v = [0u8;4];
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v, [0x12_u8, 0x34, 0x56, 0x78]);
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v = [0u16;2];
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v, [0x1234_u16, 0x5678]);
    /// ```
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        for x in self {
            x.deserialize_from(buffer)?;
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
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 18);
    /// assert_eq!(&buffer, &[0xFF; 18]);
    /// ```
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut length = 0usize;

        // copy data for each element
        for item in self {
            length += item.serialize_to(buffer)?;
        }

        Ok(length)
    }
}

impl<'a, T> FromNetworkOrder<'a> for Vec<T>
where
    T: Default + FromNetworkOrder<'a>,
{
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v: Vec<u16> = Vec::<u16>::with_capacity(2);
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v, &[0x1234_u16, 0x5678]);
    /// ```
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        // println!("inside Vec, buffer={:?}", buffer);
        // for item in self {
        //     item.deserialize_from(buffer)?;
        // }
        // Ok(())
        // the length field holds the length of data field in bytes
        let length = self.capacity();

        for _ in 0..length {
            let mut u: T = T::default();
            u.deserialize_from(buffer)?;
            self.push(u);
        }
        Ok(())        
    }
}

impl<T> ToNetworkOrder for Box<T>
where
    T: ToNetworkOrder,
{
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let v = Box::new(vec![[0xFFFF_u16;3],[0xFFFF;3],[0xFFFF;3]]);
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 18);
    /// assert_eq!(&buffer, &[0xFF; 18]);
    /// ```    
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        use std::ops::Deref;
        self.deref().serialize_to(buffer)
    }
}

impl<'a, T> FromNetworkOrder<'a> for Box<T>
where
    T: FromNetworkOrder<'a>,
{
    /// ```
    /// use std::io::Cursor;
    /// use std::ops::Deref;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v = Box::new([0;2]);
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v.deref(), &[0x1234_u16, 0x5678]);
    /// ```    
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        use std::ops::DerefMut;
        self.deref_mut().deserialize_from(buffer)
    }
}

impl ToNetworkOrder for Box<dyn ToNetworkOrder>
{
    /// ```
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let v = Box::new([0xFFFF_u16;3]);
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 6);
    /// assert_eq!(&buffer, &[0xFF; 6]);
    /// ```    
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        use std::ops::Deref;
        self.deref().serialize_to(buffer)
    }
}

impl<'a> FromNetworkOrder<'a> for Box<dyn FromNetworkOrder<'a>>
{
    /// ```
    /// use std::io::Cursor;
    /// use std::ops::Deref;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v = Box::new([0;2]);
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v.deref(), &[0x1234_u16, 0x5678]);
    /// ```    
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        use std::ops::DerefMut;
        self.deref_mut().deserialize_from(buffer)
    }
}

impl<T> ToNetworkOrder for PhantomData<T> {
    fn serialize_to(&self, _: &mut Vec<u8>) -> std::io::Result<usize> {
        Ok(0)
    }
}

impl<'a, T> FromNetworkOrder<'a> for PhantomData<T> {
    fn deserialize_from(&mut self, _: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{from_network_test, to_network_test};

    #[test]
    fn array() {
        // Array of 5 Option<u16>
        to_network_test(
            [None, Some(0x1234_u16), None, Some(0x5678_u16), None],
            4,
            &[0x12_u8, 0x34, 0x56, 0x78],
        );

        from_network_test(
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
        to_network_test(
            val,
            12,
            &[
                0x12_u8, 0x34, 0x56, 0x78, 0x23_u8, 0x45, 0x67, 0x89, 0x34_u8, 0x56, 0x78, 0x9A,
            ],
        );

        use crate::FromNetworkOrder;
        use type2network_derive::{FromNetwork, ToNetwork};
        
        #[derive(Debug, Default, PartialEq, FromNetwork)]
        struct Point {
            x: u16,
            y: u16
        }
        let w = Vec::<Point>::with_capacity(3);

        from_network_test(
            //Some(vec![[Some(0_u16); 2]; 3]),
            Some(w),
            vec![
                Point { x:0x1234_u16, y:0x5678_u16 },
                Point { x:0x2345_u16, y:0x6789_u16 },
                Point { x:0x3456_u16, y:0x789A_u16 },
            ],
            &vec![
                0x12_u8, 0x34, 0x56, 0x78, 0x23, 0x45, 0x67, 0x89, 0x34, 0x56, 0x78, 0x9A,
            ],
        );
    }
}
