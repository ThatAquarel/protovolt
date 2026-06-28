//! USB PD fixed-supply PDO parsing, scoring, and sink slot assignment.

pub const INPUT_VOLTAGE_MIN: f32 = 5.0;
pub const INPUT_VOLTAGE_MAX: f32 = 20.0;
pub const INPUT_CURRENT_MAX: f32 = 5.0;
pub const FALLBACK_5V_CURRENT: f32 = 3.0;

/// Fixed-voltage source or sink PDO in engineering units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedPdo {
    pub voltage_v: f32,
    pub current_a: f32,
}

impl FixedPdo {
    pub fn power_w(&self) -> f32 {
        self.voltage_v * self.current_a
    }
}

/// Three sink PDO slots: (PDO1 low priority, PDO3 high priority).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SinkSlots {
    pub pdo1: FixedPdo,
    pub pdo2: FixedPdo,
    pub pdo3: FixedPdo,
}

const PDO_TYPE_FIXED: u32 = 0;
const PDO_TYPE_MASK: u32 = 0xC000_0000;

/// Decode a 32-bit USB PD fixed-supply source PDO. Returns `None` for non-fixed types.
pub fn decode_fixed_src_pdo(raw: u32) -> Option<FixedPdo> {
    if (raw & PDO_TYPE_MASK) >> 30 != PDO_TYPE_FIXED {
        return None;
    }
    let current_a = (raw & 0x3FF) as f32 * 0.01;
    let voltage_v = ((raw >> 10) & 0x3FF) as f32 * 0.05;
    Some(FixedPdo {
        voltage_v,
        current_a,
    })
}

/// Encode a fixed sink PDO for DPM_SNK_PDO registers.
pub fn encode_fixed_sink_pdo(voltage_v: f32, current_a: f32) -> u32 {
    let v = quantize_voltage(voltage_v);
    let i = quantize_current(current_a);
    let voltage_bits = (v * 20.0) as u32 & 0x3FF;
    let current_bits = (i / 0.01) as u32 & 0x3FF;
    (voltage_bits << 10) | current_bits
}

fn quantize_voltage(v: f32) -> f32 {
    let clamped = v.clamp(INPUT_VOLTAGE_MIN, INPUT_VOLTAGE_MAX);
    let units = (clamped * 20.0 + 0.5) as u32;
    units as f32 / 20.0
}

fn quantize_current(i: f32) -> f32 {
    i.clamp(0.0, INPUT_CURRENT_MAX)
}

/// Source capability entry with its 1-based index in the Source_Capabilities message.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IndexedSourcePdo {
    pub index: u8,
    pub pdo: FixedPdo,
}

impl IndexedSourcePdo {
    pub fn voltage_for_rdo_pos(caps: &[IndexedSourcePdo], pos: u8) -> Option<f32> {
        caps.iter()
            .find(|entry| entry.index == pos)
            .map(|entry| entry.pdo.voltage_v)
    }
}

/// Pick maximum V×I from indexed source capabilities (ignores index).
pub fn select_best_indexed_source_pdo(caps: &[IndexedSourcePdo]) -> Option<IndexedSourcePdo> {
    caps.iter()
        .copied()
        .filter(|entry| is_valid_input_pdo(&entry.pdo))
        .max_by(|a, b| {
            a.pdo
                .power_w()
                .partial_cmp(&b.pdo.power_w())
                .unwrap_or(core::cmp::Ordering::Equal)
                .then_with(|| {
                    a.pdo
                        .voltage_v
                        .partial_cmp(&b.pdo.voltage_v)
                        .unwrap_or(core::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    a.pdo
                        .current_a
                        .partial_cmp(&b.pdo.current_a)
                        .unwrap_or(core::cmp::Ordering::Equal)
                })
        })
}

/// Filter source PDOs to input limits and pick maximum V×I.
pub fn select_best_source_pdo(caps: &[FixedPdo]) -> Option<FixedPdo> {
    caps.iter()
        .copied()
        .filter(is_valid_input_pdo)
        .max_by(|a, b| {
            a.power_w()
                .partial_cmp(&b.power_w())
                .unwrap_or(core::cmp::Ordering::Equal)
                .then_with(|| {
                    a.voltage_v
                        .partial_cmp(&b.voltage_v)
                        .unwrap_or(core::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    a.current_a
                        .partial_cmp(&b.current_a)
                        .unwrap_or(core::cmp::Ordering::Equal)
                })
        })
}

fn is_valid_input_pdo(pdo: &FixedPdo) -> bool {
    pdo.voltage_v >= INPUT_VOLTAGE_MIN
        && pdo.voltage_v <= INPUT_VOLTAGE_MAX
        && pdo.current_a > 0.0
        && pdo.current_a <= INPUT_CURRENT_MAX
}

/// Build sink slot layout: best → PDO3, second distinct → PDO2, 5 V fallback → PDO1.
pub fn build_sink_slots(caps: &[FixedPdo], best: FixedPdo) -> SinkSlots {
    let pdo1 = FixedPdo {
        voltage_v: 5.0,
        current_a: FALLBACK_5V_CURRENT.min(INPUT_CURRENT_MAX),
    };

    let mut ranked: heapless::Vec<FixedPdo, 8> = heapless::Vec::new();
    for pdo in caps.iter().copied().filter(is_valid_input_pdo) {
        let _ = ranked.push(pdo);
    }
    ranked.sort_unstable_by(|a, b| {
        b.power_w()
            .partial_cmp(&a.power_w())
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    let second = ranked
        .iter()
        .copied()
        .find(|p| (p.voltage_v - best.voltage_v).abs() > f32::EPSILON)
        .unwrap_or(best);

    SinkSlots {
        pdo1,
        pdo2: second,
        pdo3: best,
    }
}

/// Decode negotiated RDO currents (10 mA LSB). Voltage comes from the matched PDO slot.
pub fn decode_rdo_currents(raw: u32) -> (f32, f32, u8) {
    let op_ma = (raw & 0x3FF) as f32 * 10.0;
    let max_ma = ((raw >> 10) & 0x3FF) as f32 * 10.0;
    let object_pos = ((raw >> 28) & 0x07) as u8;
    (op_ma / 1000.0, max_ma / 1000.0, object_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_fixed_src_pdo_9v_3a() {
        let raw = (180 << 10) | 300;
        let pdo = decode_fixed_src_pdo(raw).unwrap();
        assert!((pdo.voltage_v - 9.0).abs() < 0.01);
        assert!((pdo.current_a - 3.0).abs() < 0.01);
    }

    #[test]
    fn scorer_prefers_higher_power() {
        let caps = [
            FixedPdo {
                voltage_v: 5.0,
                current_a: 3.0,
            },
            FixedPdo {
                voltage_v: 9.0,
                current_a: 3.0,
            },
            FixedPdo {
                voltage_v: 20.0,
                current_a: 3.25,
            },
        ];
        let best = select_best_source_pdo(&caps).unwrap();
        assert!((best.voltage_v - 20.0).abs() < f32::EPSILON);
        assert!((best.current_a - 3.25).abs() < f32::EPSILON);
    }

    #[test]
    fn scorer_prefers_9v_over_5v() {
        let caps = [
            FixedPdo {
                voltage_v: 5.0,
                current_a: 3.0,
            },
            FixedPdo {
                voltage_v: 9.0,
                current_a: 3.0,
            },
        ];
        let best = select_best_source_pdo(&caps).unwrap();
        assert!((best.voltage_v - 9.0).abs() < f32::EPSILON);
    }

    #[test]
    fn sink_slots_assign_priority_order() {
        let caps = [
            FixedPdo {
                voltage_v: 5.0,
                current_a: 3.0,
            },
            FixedPdo {
                voltage_v: 15.0,
                current_a: 3.0,
            },
            FixedPdo {
                voltage_v: 20.0,
                current_a: 3.25,
            },
        ];
        let best = select_best_source_pdo(&caps).unwrap();
        let slots = build_sink_slots(&caps, best);
        assert!((slots.pdo3.voltage_v - 20.0).abs() < f32::EPSILON);
        assert!((slots.pdo1.voltage_v - 5.0).abs() < f32::EPSILON);
        assert!((slots.pdo2.voltage_v - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn encode_round_trip() {
        let raw = encode_fixed_sink_pdo(12.0, 2.5);
        let pdo = decode_fixed_src_pdo(raw).unwrap();
        assert!((pdo.voltage_v - 12.0).abs() < 0.05);
        assert!((pdo.current_a - 2.5).abs() < 0.02);
    }

    #[test]
    fn fallback_selects_typec_when_no_valid_pdo() {
        let caps = [FixedPdo {
            voltage_v: 28.0,
            current_a: 5.0,
        }];
        assert!(select_best_source_pdo(&caps).is_none());
    }
}
