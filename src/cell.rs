//! All functions/trait to convert DNS structures to network order back & forth
use std::cell::{Cell, OnceCell, RefCell};
use std::io::{Read, Write};

//use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::error::Error;
use crate::{FromNetworkOrder, ToNetworkOrder};

// helper macro for boiler plate definitions
// #[macro_export]
// macro_rules! impl_cell {
//     ($t:ty) => {
//         impl<T> ToNetworkOrder for $t
//         where
//             T: ToNetworkOrder + Copy,
//         {
//             fn to_network_order<W: Write>(&self, buffer: &mut W) -> Result<usize, Error> {
//                 self.get().to_network_order(buffer)
//             }
//         }

//         impl<T> FromNetworkOrder for $t
//         where
//             T: FromNetworkOrder + Default,
//         {
//             fn from_network_order<R: Read>(&mut self, buffer: &mut R) -> Result<(), Error> {
//                 let mut v: T = T::default();
//                 v.from_network_order(buffer)?;

//                 self.set(v)?;
//                 Ok(())
//             }
//         }
//     };
// }

impl<T> ToNetworkOrder for Cell<T>
where
    T: ToNetworkOrder + Copy,
{
    fn to_network_order<W: Write>(&self, buffer: &mut W) -> Result<usize, Error> {
        self.get().to_network_order(buffer)
    }
}

impl<T> FromNetworkOrder for Cell<T>
where
    T: FromNetworkOrder + Default,
{
    fn from_network_order<R: Read>(&mut self, buffer: &mut R) -> Result<(), Error> {
        let mut v: T = T::default();
        v.from_network_order(buffer)?;

        self.set(v);
        Ok(())
    }
}

impl<T> ToNetworkOrder for OnceCell<T>
where
    T: ToNetworkOrder + Copy,
{
    fn to_network_order<W: Write>(&self, buffer: &mut W) -> Result<usize, Error> {
        match self.get() {
            None => Ok(0),
            Some(v) => v.to_network_order(buffer),
        }
    }
}

impl<T> FromNetworkOrder for OnceCell<T>
where
    T: FromNetworkOrder + Default,
{
    fn from_network_order<R: Read>(&mut self, buffer: &mut R) -> Result<(), Error> {
        let mut v: T = T::default();
        v.from_network_order(buffer)?;

        match self.set(v) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Custom(String::from("cell is full"))),
        }
    }
}

impl<T> ToNetworkOrder for RefCell<T>
where
    T: ToNetworkOrder,
{
    fn to_network_order<W: Write>(&self, buffer: &mut W) -> Result<usize, Error> {
        self.borrow().to_network_order(buffer)
    }
}

impl<T> FromNetworkOrder for RefCell<T>
where
    T: FromNetworkOrder + Default,
{
    fn from_network_order<R: Read>(&mut self, buffer: &mut R) -> Result<(), Error> {
        let mut v: T = T::default();
        v.from_network_order(buffer)?;

        self.replace(v);
        Ok(())
    }
}
