use crate::types::Rgb;

#[cfg(not(feature = "limits"))]
mod fwup_limits {
    pub const FWUP_MAX_IMAGE_SIZE: u32 = 800 * 1024;
    pub const FWUP_PAGE_SIZE: u32 = 4096;
    pub const FWUP_MAX_BLOCK_LEN: usize = 4096;
    pub const FWUP_SIGNATURE_LEN: usize = 64;
}

#[cfg(feature = "limits")]
pub use protov_nvm::fwup::{
    FWUP_MAX_BLOCK_LEN, FWUP_MAX_IMAGE_SIZE, FWUP_PAGE_SIZE, FWUP_SIGNATURE_LEN,
};

#[cfg(not(feature = "limits"))]
pub use fwup_limits::{
    FWUP_MAX_BLOCK_LEN, FWUP_MAX_IMAGE_SIZE, FWUP_PAGE_SIZE, FWUP_SIGNATURE_LEN,
};

pub const FWUP_DATA_PREFIX: &str = "SYST:FWUP:DATA";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScpiCommand {
    IdnQuery,
    Rst,
    Sav {
        slot: u8,
    },
    Rcl {
        slot: u8,
    },
    Del {
        slot: u8,
    },
    MeasQuery {
        kind: MeasKind,
        channel: crate::types::ScpiChannel,
    },
    ChannelQuery {
        channel: crate::types::ScpiChannel,
        param: ChannelParam,
    },
    ChannelSet {
        channel: crate::types::ScpiChannel,
        param: ChannelParam,
        value: f32,
    },
    ColorSet {
        channel: crate::types::ScpiChannel,
        rgb: Rgb,
    },
    LcdBrightnessSet {
        value: u8,
    },
    LcdBrightnessQuery,
    LedBrightnessSet {
        value: u8,
    },
    LedBrightnessQuery,
    OutputSet {
        channel: crate::types::ScpiChannel,
        on: bool,
    },
    OutputQuery {
        channel: crate::types::ScpiChannel,
    },
    ResetProt {
        channel: Option<crate::types::ScpiChannel>,
    },
    SystErrQuery,
    SystVersQuery,
    SystLoc,
    SystRem,
    TelemQuery,
    TempQuery {
        slot: TempSlot,
    },
    InpQuery,
    DiagQuery,
    Ina226RegQuery {
        channel: crate::types::RegisterChannel,
    },
    Tps55289RegQuery {
        channel: crate::types::RegisterChannel,
    },
    FwupStatQuery,
    FwupStar {
        size: u32,
    },
    FwupData,
    FwupAppl {
        signature: [u8; 64],
    },
    FwupAbor,
    Unknown {
        command: [u8; 64],
        command_len: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasKind {
    Volt,
    Curr,
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelParam {
    Volt,
    Curr,
    Ovp,
    Ocp,
    Colr,
    Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempSlot {
    Cha,
    Chb,
    Mcu,
}
