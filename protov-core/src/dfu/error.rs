#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DfuError {
    InvalidSize,
    WrongState,
    Overflow,
    BlockTooLarge,
    EmptyBlock,
}

impl DfuError {
    pub const fn scpi_message(self) -> &'static str {
        match self {
            Self::InvalidSize => "Invalid firmware size",
            Self::WrongState => "Wrong update state",
            Self::Overflow => "Firmware image overflow",
            Self::BlockTooLarge => "Block exceeds page size",
            Self::EmptyBlock => "Empty firmware block",
        }
    }
}
