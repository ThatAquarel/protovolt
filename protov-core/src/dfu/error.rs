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

#[cfg(test)]
mod tests {
    use super::DfuError;

    #[test]
    fn scpi_messages_are_stable() {
        assert_eq!(DfuError::InvalidSize.scpi_message(), "Invalid firmware size");
        assert_eq!(DfuError::WrongState.scpi_message(), "Wrong update state");
        assert_eq!(DfuError::Overflow.scpi_message(), "Firmware image overflow");
        assert_eq!(
            DfuError::BlockTooLarge.scpi_message(),
            "Block exceeds page size"
        );
        assert_eq!(DfuError::EmptyBlock.scpi_message(), "Empty firmware block");
    }
}
