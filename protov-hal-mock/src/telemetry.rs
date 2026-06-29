//! Static telemetry context defaults; live ripple deferred to a later iteration.

use protov_core::scpi::ScpiContext;

pub fn default_context() -> ScpiContext {
    ScpiContext::default()
}

pub fn health_from_protection(prot_latched_ch1: bool, prot_latched_ch2: bool) -> (bool, bool) {
    let sense_ok = !prot_latched_ch1 && !prot_latched_ch2;
    (sense_ok, sense_ok)
}

pub fn refresh_context(ctx: &mut ScpiContext, prot_latched_ch1: bool, prot_latched_ch2: bool) {
    let (sense_ok, converter_ok) = health_from_protection(prot_latched_ch1, prot_latched_ch2);
    ctx.sense_ok = sense_ok;
    ctx.converter_ok = converter_ok;
    ctx.prot_latched_a = prot_latched_ch1;
    ctx.prot_latched_b = prot_latched_ch2;
}
