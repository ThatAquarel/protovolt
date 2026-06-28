//! STUSB4500 startup negotiation: attach, existing contract, program, fallback.

use defmt::{info, warn};
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::{Duration, Timer};
use embedded_hal::i2c::I2c;

use crate::hal::event::PowerType;

use super::device::PowerDeliveryDevice;
use super::diag::{
    limits_to_power_type, log_attach, log_fallback, log_indexed_source_capabilities,
    log_negotiation_result, log_pe_state, log_programming, log_selection, log_sink_profile,
};
use super::pdo::{FixedPdo, build_sink_slots, select_best_indexed_source_pdo};
use super::regs::{
    ATTACH_POLL_MS, ATTACH_TIMEOUT_MS, ATTEMPT_BUDGET_MS, PD_ATTEMPTS, PE_POLL_MS, PE_SNK_READY,
    RDO_SETTLE_MS, SRC_CAP_POLL_MS, TLOAD_MS,
};

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

        for _ in 0..PD_ATTEMPTS {
            if self.wait_pe_snk_ready(ATTEMPT_BUDGET_MS).await {
                Timer::after(Duration::from_millis(RDO_SETTLE_MS)).await;
                if let Ok((raw, limits, pos)) = self.read_contract() {
                    if limits.current > 0.0 && limits.voltage > 5.0 {
                        info!(
                            "[pd] using existing contract V={}V I={}A",
                            limits.voltage, limits.current
                        );
                        log_negotiation_result(&limits, raw, pos);
                        return limits_to_power_type(limits, true);
                    }
                }
            }
        }

        for attempt in 0..PD_ATTEMPTS {
            info!("[pd] negotiate attempt {}", attempt + 1);

            if self.pd_soft_reset().is_err() {
                warn!("[pd] soft reset failed");
                continue;
            }

            let indexed_caps = self.burst_poll_source_capabilities(SRC_CAP_POLL_MS).await;
            if indexed_caps.is_empty() {
                continue;
            }

            self.set_source_caps(&indexed_caps);
            log_indexed_source_capabilities(&indexed_caps);

            let Some(best_entry) = select_best_indexed_source_pdo(&indexed_caps) else {
                continue;
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
                warn!("[pd] program sink failed");
                continue;
            }

            if self.pd_soft_reset().is_err() {
                warn!("[pd] post-program soft reset failed");
                continue;
            }

            if !self.wait_pe_snk_ready(ATTEMPT_BUDGET_MS).await {
                continue;
            }

            Timer::after(Duration::from_millis(RDO_SETTLE_MS)).await;

            if let Ok((raw, limits, pos)) = self.read_contract() {
                if (limits.voltage - best.voltage_v).abs() <= 1.0 && limits.current > 0.0 {
                    log_negotiation_result(&limits, raw, pos);
                    return limits_to_power_type(limits, true);
                }
                warn!(
                    "[pd] contract V={}V below selected {}V (pos={})",
                    limits.voltage, best.voltage_v, pos
                );
            }
        }

        let limits = self.limits_from_typec();
        log_fallback("negotiation failed", &limits);
        limits_to_power_type(limits, false)
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
            Timer::after(Duration::from_millis(ATTACH_POLL_MS)).await;
        }
    }

    async fn wait_pe_snk_ready(&mut self, budget_ms: u64) -> bool {
        let deadline = Duration::from_millis(budget_ms);
        let start = embassy_time::Instant::now();
        let mut last_state = 0u8;
        loop {
            if let Ok(state) = self.pe_state() {
                if state != last_state {
                    log_pe_state(state);
                    last_state = state;
                }
                if state == PE_SNK_READY {
                    return true;
                }
            }
            if start.elapsed() >= deadline {
                if last_state != 0 {
                    log_pe_state(last_state);
                }
                return false;
            }
            Timer::after(Duration::from_millis(PE_POLL_MS)).await;
        }
    }
}
