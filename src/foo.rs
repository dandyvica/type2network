impl std::convert::TryFrom<u64> for Color {
    type Error = u64;
    fn try_from(value: u64) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Black),
            1 => Ok(Self::White),
            3 => Ok(Self::Yellow),
            55 => Ok(Self::Brown),
            _ => Err(value),
        }
    }
}
impl std::convert::TryFrom<u16> for OpCodeReserved {
    type Error = u16;
    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Query),
            1 => Ok(Self::IQuery),
            2 => Ok(Self::Status),
            3 => Ok(Self::Unassigned),
            4 => Ok(Self::Notify),
            5 => Ok(Self::Update),
            6 => Ok(Self::DOS),
            _ => Ok(OpCodeReserved::Reserved(value)),
        }
    }
}
