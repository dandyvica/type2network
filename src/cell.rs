//! All functions/trait to convert DNS structures to network order back & forth
use std::cell::{Cell, OnceCell, RefCell};
use std::io::ErrorKind;
use std::io::{Error, Read, Write};

//use crate::error::Error;
use crate::{FromNetworkOrder, ToNetworkOrder};

// helper macro for boiler plate definitions
// #[macro_export]
// macro_rules! impl_cell {
//     ($t:ty) => {
//         impl<T> ToNetworkOrder for $t
//         where
//             T: ToNetworkOrder + Copy,
//         {
//             fn serialize_to(&self, buffer: &mut W) -> std::io::Result<usize> {
//                 self.get().serialize_to(buffer)
//             }
//         }

//         impl<T> FromNetworkOrder<'a> for $t
//         where
//             T: FromNetworkOrder<'a> + Default,
//         {
//             fn deserialize_from(&mut self, buffer: &mut R) -> std::io::Result<()> {
//                 let mut v: T = T::default();
//                 v.deserialize_from(buffer)?;

//                 self.set(v)?;
//                 Ok(())
//             }
//         }
//     };
// }

impl<T, W: Write> ToNetworkOrder<W> for Cell<T>
where
    T: ToNetworkOrder<W> + Copy,
{
    /// ```
    /// use std::cell::Cell;
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let v = Cell::new([[0xFFFF_u16;3],[0xFFFF;3],[0xFFFF;3]]);
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 18);
    /// assert_eq!(&buffer, &[0xFF; 18]);
    /// ```       
    fn serialize_to(&self, buffer: &mut W) -> std::io::Result<usize> {
        self.get().serialize_to(buffer)
    }
}

impl<'a, T, R: Read> FromNetworkOrder<'a, R> for Cell<T>
where
    T: FromNetworkOrder<'a, R> + Default,
{
    /// ```
    /// use std::io::Cursor;
    /// use std::cell::Cell;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = [0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v = Cell::new([0_u16;2]);
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v, Cell::new([0x1234_u16, 0x5678]));
    /// ```      
    fn deserialize_from(&mut self, buffer: &mut R) -> std::io::Result<()> {
        let mut v: T = T::default();
        v.deserialize_from(buffer)?;

        self.set(v);
        Ok(())
    }
}

impl<T, W: Write> ToNetworkOrder<W> for OnceCell<T>
where
    T: ToNetworkOrder<W> + Copy,
{
    /// ```
    /// use std::cell::OnceCell;
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let mut v = OnceCell::new();
    /// v.set([[0xFFFF_u16;3],[0xFFFF;3],[0xFFFF;3]]).unwrap();
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 18);
    /// assert_eq!(&buffer, &[0xFF; 18]);
    /// ```     
    fn serialize_to(&self, buffer: &mut W) -> std::io::Result<usize> {
        match self.get() {
            None => Ok(0),
            Some(v) => v.serialize_to(buffer),
        }
    }
}

impl<'a, T, R: Read> FromNetworkOrder<'a, R> for OnceCell<T>
where
    T: FromNetworkOrder<'a, R> + Default,
{
    /// ```
    /// use std::io::Cursor;
    /// use std::cell::OnceCell;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = [0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v: OnceCell<[u16;2]> = OnceCell::new();
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v.get().unwrap(), &[0x1234_u16, 0x5678]);
    /// ```      
    fn deserialize_from(&mut self, buffer: &mut R) -> std::io::Result<()> {
        let mut v: T = T::default();
        v.deserialize_from(buffer)?;

        match self.set(v) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new(ErrorKind::Other, "cell is full")),
        }
    }
}

impl<T, W: Write> ToNetworkOrder<W> for RefCell<T>
where
    T: ToNetworkOrder<W>,
{
    /// ```
    /// use std::cell::RefCell;
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer: Vec<u8> = Vec::new();
    /// let v = RefCell::new(vec![[0xFFFF_u16;3],[0xFFFF;3],[0xFFFF;3]]);
    /// assert_eq!(v.serialize_to(&mut buffer).unwrap(), 18);
    /// assert_eq!(&buffer, &[0xFF; 18]);
    /// ```      
    fn serialize_to(&self, buffer: &mut W) -> std::io::Result<usize> {
        self.borrow().serialize_to(buffer)
    }
}

impl<'a, T, R: Read> FromNetworkOrder<'a, R> for RefCell<T>
where
    T: FromNetworkOrder<'a, R> + Default + std::fmt::Debug,
{
    /// ```
    /// use std::io::Cursor;
    /// use std::cell::RefCell;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x12, 0x34, 0x56, 0x78];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut v = RefCell::new([0_u16;2]);
    /// assert!(v.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(v, RefCell::new([0x1234_u16, 0x5678]));
    /// ```     
    fn deserialize_from(&mut self, buffer: &mut R) -> std::io::Result<()> {
        let mut v = T::default();
        v.deserialize_from(buffer)?;

        self.replace(v);
        Ok(())
    }
}
