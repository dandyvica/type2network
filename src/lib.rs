// function to convert to network order (big-endian)
pub trait ToNetworkOrder {
    // copy structure data to a network-order buffer
    fn serialize_to(&self, buffer: &mut Vec<u8>) -> std::io::Result<usize>;
}

// function to convert from network order (big-endian)
pub trait FromNetworkOrder<'a> {
    // copy from a network-order buffer to a structure
    fn deserialize_from(&mut self, buffer: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<()>;
}

//all definitions of serialize_to()/deserialize_from() for standard types
pub mod cell;
pub mod generics;
pub mod primitive;

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use std::io::Cursor;

    // used for boiler plate unit tests for integers
    pub fn to_network_test<T: ToNetworkOrder>(val: T, size: usize, v: &[u8]) {
        let mut buffer: Vec<u8> = Vec::new();
        assert_eq!(val.serialize_to(&mut buffer).unwrap(), size);
        assert_eq!(buffer, v);
    }

    // used for boiler plate unit tests for integers, floats etc
    pub fn from_network_test<'a, T>(def: Option<T>, val: T, buf: &'a Vec<u8>)
    where
        T: FromNetworkOrder<'a> + Default + std::fmt::Debug + std::cmp::PartialEq,
    {
        let mut buffer = Cursor::new(buf.as_slice());
        let mut v: T = if def.is_none() {
            T::default()
        } else {
            def.unwrap()
        };
        assert!(v.deserialize_from(&mut buffer).is_ok());
        assert_eq!(v, val);
    }
}
