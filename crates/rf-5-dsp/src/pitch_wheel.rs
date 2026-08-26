//! SD334 pitch-wheel potentiometer, diode pair and serviced performance span.
//!
//! The centred R1 wiper is trimmed to 0 V at P301-7. Anti-parallel 1N914
//! diodes D315/D316 couple that node to the tune summer with a soft silicon
//! knee. The 100 kohm R3100 shunt, R1's position-dependent source resistance
//! and downstream 1 Mohm / 100 kohm path all load that junction.

pub const RANGE_SEMITONES: f32 = 7.0;

const SUPPLY_VOLTS: f32 = 15.0;
const WHEEL_POT_OHMS: f32 = 100_000.0;
const POSITIVE_RAIL_SERIES_OHMS: f32 = 4_700.0;
const WIPER_SHUNT_OHMS: f32 = 100_000.0;
const MASTER_TUNE_INPUT_OHMS: f32 = 1_000_000.0;
const MASTER_TUNE_FEEDBACK_OHMS: f32 = 100_000.0;
const VOLTS_PER_OCTAVE: f32 = 1.0;
const SEMITONES_PER_OCTAVE: f32 = 12.0;
#[cfg(test)]
const SERVICE_CENTER_TOLERANCE_VOLTS: f32 = 0.05;

// Vishay's 25 C typical 1N914 graph is closely bounded from 1 uA through
// 1 mA by a 1.7 ideality factor and 1.5 nA saturation current. The modern
// graph bounds the historical part rather than identifying its population.
const DIODE_IDEALITY_FACTOR: f32 = 1.7;
const THERMAL_VOLTAGE_VOLTS: f32 = 0.025_85;
const DIODE_SATURATION_CURRENT_AMPS: f32 = 1.5e-9;

// With R3129 serviced to centre the 100k track, its effective negative-side
// resistance matches R3106's 4.7k positive-side feed. The wheel therefore
// sees symmetric track endpoints inside the +/-15 V rails.
const TRACK_HALF_SPAN_VOLTS: f32 =
    SUPPLY_VOLTS * WHEEL_POT_OHMS / (WHEEL_POT_OHMS + 2.0 * POSITIVE_RAIL_SERIES_OHMS);

// The owner's-manual approximately-one-fifth span fixes mechanical travel,
// not an invented electronic gain. Solving the complete nominal network for
// seven semitones places each wheel endpoint 26.978% of the track from centre.
const MECHANICAL_HALF_TRAVEL: f32 = 0.269_783_7;
const FULL_SCALE_SUMMER_CURRENT_AMPS: f32 =
    RANGE_SEMITONES / SEMITONES_PER_OCTAVE * VOLTS_PER_OCTAVE / MASTER_TUNE_FEEDBACK_OHMS;

pub fn normalized_output(value: u16) -> f32 {
    let input = midi_normalized(value);
    if input == 0.0 {
        0.0
    } else {
        let current = wheel_current_amps(input.abs());
        let normalized = (current / FULL_SCALE_SUMMER_CURRENT_AMPS).clamp(0.0, 1.0);
        input.signum() * normalized
    }
}

fn wheel_current_amps(normalized_magnitude: f32) -> f32 {
    let position = 0.5 + normalized_magnitude.clamp(0.0, 1.0) * MECHANICAL_HALF_TRAVEL;
    let source_volts = (2.0 * position - 1.0) * TRACK_HALF_SPAN_VOLTS;
    let source_resistance_ohms = position * (1.0 - position) * WHEEL_POT_OHMS;

    let mut low = 0.0;
    let mut high = source_volts / MASTER_TUNE_INPUT_OHMS;
    for _ in 0..32 {
        let current = (low + high) * 0.5;
        let diode_volts = diode_pair_voltage(current);
        let wiper_volts = current * MASTER_TUNE_INPUT_OHMS + diode_volts;
        let required_source_volts =
            wiper_volts + source_resistance_ohms * (wiper_volts / WIPER_SHUNT_OHMS + current);
        if required_source_volts < source_volts {
            low = current;
        } else {
            high = current;
        }
    }
    (low + high) * 0.5
}

fn diode_pair_voltage(current_amps: f32) -> f32 {
    if !current_amps.is_finite() || current_amps <= 0.0 {
        return 0.0;
    }
    let normalized_current = current_amps / (2.0 * DIODE_SATURATION_CURRENT_AMPS);
    DIODE_IDEALITY_FACTOR
        * THERMAL_VOLTAGE_VOLTS
        * libm::logf(
            normalized_current + libm::sqrtf(normalized_current * normalized_current + 1.0),
        )
}

fn midi_normalized(value: u16) -> f32 {
    let value = value.min(16_383);
    if value < 8_192 {
        (f32::from(value) - 8_192.0) / 8_192.0
    } else {
        (f32::from(value) - 8_192.0) / 8_191.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anti_parallel_diodes_create_a_symmetric_soft_knee() {
        assert_eq!(normalized_output(8_192), 0.0);
        for offset in [1, 64, 256, 1_024, 4_096, 8_191] {
            let positive = normalized_output(8_192 + offset);
            let negative = normalized_output(8_192 - offset);
            assert!(positive > 0.0);
            assert!(negative < 0.0);
            assert!((positive + negative).abs() < 2.0e-4);
        }
        assert!(normalized_output(8_192 + 256) < 0.01);
        assert!(normalized_output(8_192 + 4_096) > 0.4);
    }

    #[test]
    fn serviced_center_tolerance_is_musically_negligible() {
        let full_wiper_source_volts = 2.0 * MECHANICAL_HALF_TRAVEL * TRACK_HALF_SPAN_VOLTS;
        let normalized_tolerance = SERVICE_CENTER_TOLERANCE_VOLTS / full_wiper_source_volts;
        let midi_tolerance = (normalized_tolerance * 8_191.0).ceil() as u16;
        let positive_cents = normalized_output(8_192 + midi_tolerance) * RANGE_SEMITONES * 100.0;
        let negative_cents = normalized_output(8_192 - midi_tolerance) * RANGE_SEMITONES * 100.0;
        assert!(positive_cents < 0.5);
        assert!(negative_cents > -0.5);
    }

    #[test]
    fn full_midi_range_preserves_the_documented_fifth_endpoints() {
        assert_eq!(normalized_output(0), -1.0);
        assert_eq!(normalized_output(16_383), 1.0);
        assert_eq!(normalized_output(0) * RANGE_SEMITONES, -7.0);
        assert_eq!(normalized_output(16_383) * RANGE_SEMITONES, 7.0);
    }

    #[test]
    fn circuit_transfer_is_monotonic_and_bounded() {
        let mut previous = -1.0;
        for value in 0..=16_383 {
            let output = normalized_output(value);
            assert!((-1.0..=1.0).contains(&output));
            assert!(output >= previous);
            previous = output;
        }
    }

    #[test]
    fn nominal_supply_and_service_trim_center_the_track() {
        assert!((13.70..13.72).contains(&TRACK_HALF_SPAN_VOLTS));
        let track_current_amps = 2.0 * TRACK_HALF_SPAN_VOLTS / WHEEL_POT_OHMS;
        let required_negative_trim_ohms =
            (SUPPLY_VOLTS - TRACK_HALF_SPAN_VOLTS) / track_current_amps;
        assert!((4_699.0..4_701.0).contains(&required_negative_trim_ohms));
        assert!(required_negative_trim_ohms < 10_000.0);
    }

    #[test]
    fn diode_fit_tracks_the_datasheet_microamp_curve() {
        assert!((0.26..0.31).contains(&diode_pair_voltage(1.0e-6)));
        assert!((0.36..0.41).contains(&diode_pair_voltage(10.0e-6)));
        assert!((0.56..0.63).contains(&diode_pair_voltage(1.0e-3)));
    }
}
