//! STUSB4500 startup negotiation: attach, source-cap capture, PDO scoring, reprogram, contract verify.
//!
//! ## Manual bench validation
//!
//! | Adapter | Expected contract | Log tags to verify |
//! |---|---|---|
//! | 65 W USB-C PD (20 V) | 20 V at ≤ 5 A | `[pd] select`, `[pd] contract` |
//! | 9 V / 3 A PD-only | 9 V / 3 A | `[pd] src cap`, `[pd] contract` |
//! | 5 V / 3 A Type-C (no PD) | 5 V Type-C current | `[pd] fallback: no source caps` or attach-only |
//! | No cable / open port | Standard 5 V limits | `[pd] fallback: no attach` |

use defmt::{info, warn};
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::{Duration, Timer};
use embedded_hal::i2c::I2c;

use crate::hal::event::{Limits, PowerType};
use crate::hal::power::regs::SRC_CAP_POLL_MS;

use super::device::PowerDeliveryDevice;
use super::diag::{
    limits_to_power_type, log_attach, log_fallback, log_indexed_source_capabilities,
    log_negotiation_result, log_pe_state, log_programming, log_selection, log_sink_profile,
};
use super::pdo::{FixedPdo, INPUT_CURRENT_MAX, build_sink_slots, select_best_indexed_source_pdo};
use super::regs::{ATTACH_TIMEOUT_MS, NEGOTIATE_TIMEOUT_MS, PE_SNK_READY, TLOAD_MS};

impl<M, BUS> PowerDeliveryDevice<'_, M, BUS>
where
    M: RawMutex,
    BUS: I2c,
{
    pub async fn init_and_negotiate(&mut self) -> PowerType {
        Timer::after(Duration::from_millis(TLOAD_MS)).await;

        if self.registers_need_baseline() {
            if self.is_attached().unwrap_or(false) {
                warn!("[pd] sink registers mismatch but source attached; skip NVM program");
            } else if self.program_manufacturing_baseline().is_err() {
                warn!("[pd] baseline NVM program failed");
            }
        }

        if !self.wait_attach().await {
            let limits = self.limits_from_typec();
            log_fallback("no attach", &limits);
            return limits_to_power_type(limits, false);
        }

        log_sink_profile(self);

        let mut indexed_caps = self.burst_poll_source_capabilities(SRC_CAP_POLL_MS).await;
        if indexed_caps.is_empty() {
            info!("[pd] sending PD soft reset");
            if self.pd_soft_reset().is_err() {
                warn!("[pd] PD soft reset failed");
            }
            indexed_caps = self.burst_poll_source_capabilities(SRC_CAP_POLL_MS).await;
        }

        if indexed_caps.is_empty() {
            if let Ok(Some(limits)) = self.read_active_contract() {
                info!("[pd] using NVM-negotiated contract (no src cap capture)");
                log_negotiation_result(&limits, 0, 0);
                return limits_to_power_type(limits, true);
            }

            let limits = self.limits_from_typec();
            log_fallback("no source caps", &limits);
            return limits_to_power_type(limits, false);
        }

        self.set_source_caps(&indexed_caps);
        log_indexed_source_capabilities(&indexed_caps);

        let Some(best_entry) = select_best_indexed_source_pdo(&indexed_caps) else {
            if let Ok(Some(limits)) = self.read_active_contract() {
                log_negotiation_result(&limits, 0, 0);
                return limits_to_power_type(limits, true);
            }
            let limits = self.limits_from_typec();
            log_fallback("no valid PDO", &limits);
            return limits_to_power_type(limits, false);
        };

        let best = best_entry.pdo;
        log_selection(best);

        let mut fixed_caps = heapless::Vec::<FixedPdo, 7>::new();
        for entry in indexed_caps.iter() {
            let _ = fixed_caps.push(entry.pdo);
        }
        let slots = build_sink_slots(&fixed_caps, best);
        log_programming(&slots);

        if self.program_sink_slots(&slots).is_err() {
            if let Ok(Some(limits)) = self.read_active_contract() {
                log_negotiation_result(&limits, 0, 0);
                return limits_to_power_type(limits, true);
            }
            let limits = self.limits_from_typec();
            log_fallback("program sink failed", &limits);
            return limits_to_power_type(limits, false);
        }

        info!("[pd] re-negotiating with programmed sink PDOs");
        if self.pd_soft_reset().is_err() {
            warn!("[pd] post-program soft reset failed");
        }

        if !self.wait_snk_ready().await {
            if let Ok(Some(limits)) = self.read_active_contract() {
                log_negotiation_result(&limits, 0, 0);
                return limits_to_power_type(limits, true);
            }
            let limits = self.limits_from_typec();
            log_fallback("contract timeout", &limits);
            return limits_to_power_type(limits, false);
        }

        // Timer::after(Duration::from_millis(100)).await;

        match self.read_contract() {
            Ok((raw, limits, pos)) => {
                if (limits.voltage - best.voltage_v).abs() > 1.0 {
                    warn!(
                        "[pd] contract V={}V below selected {}V (pos={})",
                        limits.voltage, best.voltage_v, pos
                    );
                }
                log_negotiation_result(&limits, raw, pos);
                limits_to_power_type(limits, true)
            }
            Err(()) => {
                let limits = Limits {
                    voltage: best.voltage_v,
                    current: best.current_a.min(INPUT_CURRENT_MAX),
                };
                log_fallback("RDO read failed, using selected PDO", &limits);
                limits_to_power_type(limits, true)
            }
        }
    }

    async fn wait_attach(&mut self) -> bool {
        let deadline = Duration::from_millis(ATTACH_TIMEOUT_MS);
        let start = embassy_time::Instant::now();
        loop {
            let attached = self.is_attached().unwrap_or(false);
            let cc = self.cc_status().unwrap_or(0);
            if attached {
                log_attach(true, cc);
                return true;
            }
            if start.elapsed() >= deadline {
                log_attach(false, cc);
                return false;
            }
            Timer::after(Duration::from_millis(50)).await;
        }
    }

    async fn wait_snk_ready(&mut self) -> bool {
        let deadline = Duration::from_millis(NEGOTIATE_TIMEOUT_MS);
        let start = embassy_time::Instant::now();
        loop {
            if let Ok(state) = self.pe_state() {
                log_pe_state(state);
                if state == PE_SNK_READY {
                    return true;
                }
            }
            if start.elapsed() >= deadline {
                return false;
            }
            Timer::after(Duration::from_millis(20)).await;
        }
    }
}
