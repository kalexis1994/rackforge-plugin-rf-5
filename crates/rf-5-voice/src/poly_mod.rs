//! SD431 per-voice Poly Mod destination network.
//!
//! Both amount VCAs feed one physical PMOD voltage. Three switches then route
//! that same voltage through the populated oscillator-frequency, pulse-width
//! and filter-frequency networks. Keeping the uncertain bus voltage in one
//! place preserves those hardware ratios and makes later calibration local.

const OCTAVE_SEMITONES: f32 = 12.0;

// SD431 routes PMOD through R4357 to the CEM3340 oscillator-A summing node.
// The board's calibrated pitch input uses 100 kohm for one volt per octave,
// so the 30.1 kohm PMOD input supplies 100/30.1 octaves per PMOD volt.
const PITCH_REFERENCE_INPUT_OHMS: f32 = 100_000.0;
const PITCH_POLY_MOD_INPUT_OHMS: f32 = 30_100.0;

// R4112 feeds the inverting U432 pulse-width summer and R4162 closes its
// feedback loop. The CEM3340's complete duty-cycle control range is 5 V.
const PULSE_WIDTH_POLY_MOD_INPUT_OHMS: f32 = 30_100.0;
const PULSE_WIDTH_FEEDBACK_OHMS: f32 = 52_300.0;
const CEM3340_PULSE_WIDTH_RANGE_VOLTS: f32 = 5.0;

// R4181 injects PMOD into the same U433 filter-frequency summer that receives
// the calibrated common filter CV through R4143. Their resistance ratio is
// independent of the per-voice FIL 1 SCALE trimmer later in the same stage.
const FILTER_REFERENCE_INPUT_OHMS: f32 = 100_000.0;
const FILTER_POLY_MOD_INPUT_OHMS: f32 = 54_900.0;

// The schematic establishes every destination ratio but not the absolute
// loaded voltage at U431's PMOD buffer. A 1.2 V unit-bus span is retained as
// the sole candidate anchor: it keeps the established near-four-octave pitch
// audition while allowing all other destinations to follow SD431 instead of
// unrelated hand-tuned depths.
const CANDIDATE_PMOD_BUS_VOLTS_PER_UNIT: f32 = 1.2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PolyModDestinations {
    pub oscillator_a_semitones: f32,
    pub oscillator_a_pulse_width: f32,
    pub filter_octaves: f32,
}

pub fn destinations(normalized_bus: f32) -> PolyModDestinations {
    if !normalized_bus.is_finite() {
        return PolyModDestinations::default();
    }

    let bus_volts = normalized_bus * CANDIDATE_PMOD_BUS_VOLTS_PER_UNIT;
    let oscillator_octaves = bus_volts * PITCH_REFERENCE_INPUT_OHMS / PITCH_POLY_MOD_INPUT_OHMS;
    let pulse_width = bus_volts * PULSE_WIDTH_FEEDBACK_OHMS
        / PULSE_WIDTH_POLY_MOD_INPUT_OHMS
        / CEM3340_PULSE_WIDTH_RANGE_VOLTS;
    let filter_octaves = bus_volts * FILTER_REFERENCE_INPUT_OHMS / FILTER_POLY_MOD_INPUT_OHMS;

    PolyModDestinations {
        oscillator_a_semitones: oscillator_octaves * OCTAVE_SEMITONES,
        oscillator_a_pulse_width: pulse_width,
        filter_octaves,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_voltage_anchor_drives_every_destination() {
        let routed = destinations(1.0);
        assert!((routed.oscillator_a_semitones - 47.840_53).abs() < 1.0e-5);
        assert!((routed.oscillator_a_pulse_width - 0.417_009_98).abs() < 1.0e-5);
        assert!((routed.filter_octaves - 2.185_792_4).abs() < 1.0e-5);
    }

    #[test]
    fn destination_ratios_follow_sd431_resistors() {
        let routed = destinations(0.37);
        let oscillator_octaves = routed.oscillator_a_semitones / OCTAVE_SEMITONES;
        let expected_pulse_ratio = (PULSE_WIDTH_FEEDBACK_OHMS
            / PULSE_WIDTH_POLY_MOD_INPUT_OHMS
            / CEM3340_PULSE_WIDTH_RANGE_VOLTS)
            / (PITCH_REFERENCE_INPUT_OHMS / PITCH_POLY_MOD_INPUT_OHMS);
        let expected_filter_ratio = (FILTER_REFERENCE_INPUT_OHMS / FILTER_POLY_MOD_INPUT_OHMS)
            / (PITCH_REFERENCE_INPUT_OHMS / PITCH_POLY_MOD_INPUT_OHMS);

        assert!(
            (routed.oscillator_a_pulse_width / oscillator_octaves - expected_pulse_ratio).abs()
                < 1.0e-6
        );
        assert!(
            (routed.filter_octaves / oscillator_octaves - expected_filter_ratio).abs() < 1.0e-6
        );
    }

    #[test]
    fn bus_is_bipolar_and_invalid_input_is_silent() {
        assert_eq!(destinations(f32::NAN), PolyModDestinations::default());
        assert_eq!(destinations(f32::INFINITY), PolyModDestinations::default());

        let positive = destinations(0.43);
        let negative = destinations(-0.43);
        assert_eq!(
            positive.oscillator_a_semitones,
            -negative.oscillator_a_semitones
        );
        assert_eq!(
            positive.oscillator_a_pulse_width,
            -negative.oscillator_a_pulse_width
        );
        assert_eq!(positive.filter_octaves, -negative.filter_octaves);
    }
}
