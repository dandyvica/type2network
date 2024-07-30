//! Serialization and Deserialization for IPV4 and IPV6 addresses.
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::{FromNetworkOrder, ToNetworkOrder};

impl ToNetworkOrder for Ipv4Addr {
    /// Example:    
    /// ```
    /// use std::str::FromStr;
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer = Vec::new();
    /// let ip = std::net::Ipv4Addr::from_str("142.250.179.100").unwrap();
    /// assert_eq!(ip.serialize_to(&mut buffer).unwrap(), 4);
    /// assert_eq!(buffer, [0x8e, 0xfa, 0xb3, 0x64]);
    /// ```
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        self.octets().serialize_to(buffer)
    }
}

impl<'a> FromNetworkOrder<'a> for Ipv4Addr {
    /// Example:    
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x8e, 0xfa, 0xb3, 0x64];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut ip = std::net::Ipv4Addr::UNSPECIFIED;
    /// assert!(ip.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(ip.to_string(), "142.250.179.100");
    /// ```
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        // get the array of 4 bytes
        let mut array = [0u8; 4];
        array.deserialize_from(buffer)?;
        *self = Ipv4Addr::from(array);

        Ok(())
    }
}

impl ToNetworkOrder for Ipv6Addr {
    /// Example:    
    /// ```
    /// use std::str::FromStr;
    /// use type2network::ToNetworkOrder;
    ///
    /// let mut buffer = Vec::new();
    /// let ip = std::net::Ipv6Addr::from_str("2607:f8b0:4004:c07::93").unwrap();
    /// assert_eq!(ip.serialize_to(&mut buffer).unwrap(), 16);
    /// assert_eq!(buffer, [0x26, 0x07, 0xf8, 0xb0, 0x40, 0x04, 0x0c, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x93]);
    /// ```
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
        self.octets().serialize_to(buffer)
    }
}

impl<'a> FromNetworkOrder<'a> for Ipv6Addr {
    /// Example:    
    /// ```
    /// use std::io::Cursor;
    /// use type2network::FromNetworkOrder;
    ///
    /// let b = vec![0x26, 0x07, 0xf8, 0xb0, 0x40, 0x04, 0x0c, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x93];
    /// let mut buffer = Cursor::new(b.as_slice());
    /// let mut ip = std::net::Ipv6Addr::UNSPECIFIED;
    /// assert!(ip.deserialize_from(&mut buffer).is_ok());
    /// assert_eq!(ip.to_string(), "2607:f8b0:4004:c07::93");
    /// ```
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()> {
        // get the array of 4 bytes
        let mut array = [0u8; 16];
        array.deserialize_from(buffer)?;
        *self = Ipv6Addr::from(array);

        Ok(())
    }
}
