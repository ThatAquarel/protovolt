use crate::hal::converter::ConverterFlags;
use crate::hal::event::{Channel, ChannelHardwareState, Limits, Readout};
use crate::hal::temperature::TemperatureReading;

pub use crate::config::{CH_OTP_C, MCU_OTP_C};

pub fn is_fault_state(state: ChannelHardwareState) -> bool {
    state.is_fault()
}

pub fn derive_hw_state(
    channel: Channel,
    enable: bool,
    flags: Option<ConverterFlags>,
    readout: Option<Readout>,
    limits: Limits,
    temps: &TemperatureReading,
    prot_latched: bool,
    latched_state: ChannelHardwareState,
) -> ChannelHardwareState {
    if prot_latched && is_fault_state(latched_state) {
        return latched_state;
    }

    if temps.mcu >= MCU_OTP_C {
        return ChannelHardwareState::OverTemperature;
    }

    let ch_temp = match channel {
        Channel::A => temps.ch_a,
        Channel::B => temps.ch_b,
    };
    if ch_temp >= CH_OTP_C {
        return ChannelHardwareState::OverTemperature;
    }

    if let Some(readout) = readout {
        if readout.current > limits.current {
            return ChannelHardwareState::OverCurrent;
        }
        if readout.voltage > limits.voltage {
            return ChannelHardwareState::OverVoltage;
        }
    }

    if let Some(flags) = flags {
        if flags.ovp {
            return ChannelHardwareState::OverVoltage;
        }
        if flags.scp {
            return ChannelHardwareState::ShortCircuit;
        }
    }

    if !enable {
        return ChannelHardwareState::Off;
    }

    if let Some(flags) = flags {
        if flags.ocp {
            return ChannelHardwareState::ConstantCurrent;
        }
        return ChannelHardwareState::ConstantVoltage;
    }

    ChannelHardwareState::ConstantVoltage
}

pub fn mcu_overtemp(temps: &TemperatureReading) -> bool {
    temps.mcu >= MCU_OTP_C
}
