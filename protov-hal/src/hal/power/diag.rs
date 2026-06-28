use defmt::{info, warn};

use embassy_sync::blocking_mutex::raw::RawMutex;
use embedded_hal::i2c::I2c;

use crate::hal::event::{Limits, PowerType};

pub fn limits_to_power_type(limits: Limits, pd: bool) -> PowerType {
    if pd {
        PowerType::PowerDelivery(limits)
    } else {
        PowerType::Standard(limits)
    }
}

use super::device::PowerDeliveryDevice;
use super::pdo::FixedPdo;

pub fn log_rx_message(msg_type: u8, num_obj: u8) {
    info!("[pd] rx msg type={} objs={}", msg_type, num_obj);
}

pub fn log_sink_profile<M, BUS>(dev: &mut PowerDeliveryDevice<M, BUS>)
where
    M: RawMutex,
    BUS: I2c,
{
    let pdo_count = dev.get_pdo_number().unwrap_or(0);
    info!("[pd] sink profile: {} PDO(s) configured", pdo_count);

    for pdo in 1..=3u8 {
        info!(
            "[pd] sink PDO{}: V={}V I={}A UV={}% OV={}%",
            pdo,
            dev.get_voltage(pdo).unwrap_or(0.0),
            dev.get_current(pdo).unwrap_or(0.0),
            dev.get_lower_voltage_limit(pdo).unwrap_or(0),
            dev.get_upper_voltage_limit(pdo).unwrap_or(0),
        );
    }

    if dev.has_nvm_cache() {
        info!(
            "[pd] sink config: flex={}A ext_pwr={} usb_comm={} ok_gpio={} gpio={} above5v_only={} req_src_cur={}",
            dev.get_flex_current().unwrap_or(0.0),
            dev.get_external_power().unwrap_or(0),
            dev.get_usb_comm_capable().unwrap_or(0),
            dev.get_config_ok_gpio().unwrap_or(0),
            dev.get_gpio_ctrl().unwrap_or(0),
            dev.get_power_above_5v_only().unwrap_or(0),
            dev.get_req_src_current().unwrap_or(0),
        );
    }
}

pub fn log_source_capabilities(caps: &[FixedPdo]) {
    info!("[pd] src cap: {} fixed PDO(s)", caps.len());
    for (i, pdo) in caps.iter().enumerate() {
        info!(
            "[pd] src cap #{}: V={}V I={}A P={}W",
            i + 1,
            pdo.voltage_v,
            pdo.current_a,
            pdo.power_w()
        );
    }
}

pub fn log_indexed_source_capabilities(caps: &[super::pdo::IndexedSourcePdo]) {
    info!("[pd] src cap: {} fixed PDO(s)", caps.len());
    for entry in caps {
        info!(
            "[pd] src cap pos{}: V={}V I={}A P={}W",
            entry.index,
            entry.pdo.voltage_v,
            entry.pdo.current_a,
            entry.pdo.power_w()
        );
    }
}

pub fn log_selection(best: FixedPdo) {
    info!(
        "[pd] select: V={}V I={}A P={}W",
        best.voltage_v,
        best.current_a,
        best.power_w()
    );
}

pub fn log_programming(slots: &super::pdo::SinkSlots) {
    info!(
        "[pd] program: PDO1={}V/{}A PDO2={}V/{}A PDO3={}V/{}A",
        slots.pdo1.voltage_v,
        slots.pdo1.current_a,
        slots.pdo2.voltage_v,
        slots.pdo2.current_a,
        slots.pdo3.voltage_v,
        slots.pdo3.current_a,
    );
}

pub fn log_negotiation_result(limits: &Limits, rdo_raw: u32, object_pos: u8) {
    info!(
        "[pd] contract: V={}V I={}A P={}W rdo=0x{:08x} pos={}",
        limits.voltage,
        limits.current,
        limits.voltage * limits.current,
        rdo_raw,
        object_pos,
    );
}

pub fn log_attach(attached: bool, cc_status: u8) {
    info!(
        "[pd] attach: connected={} cc_status=0x{:02x}",
        attached, cc_status
    );
}

pub fn log_fallback(reason: &str, limits: &Limits) {
    warn!(
        "[pd] fallback: {} -> Standard/PD V={}V I={}A",
        reason, limits.voltage, limits.current
    );
}

pub fn log_pe_state(state: u8) {
    info!("[pd] pe_state=0x{:02x}", state);
}
