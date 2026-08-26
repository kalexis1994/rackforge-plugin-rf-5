//! Physical CA3280 boundaries in the RF-5 audio path.
//!
//! Each voice card uses both halves of one CA3280 for oscillator A/B level,
//! with the linearizing-diode terminal cut off. A separate linearized half is
//! the final voice VCA. The common noise level and master volume each have
//! their own physical CA3280 stage. Balance and voice-volume trimmers are
//! treated as serviced, so they cancel zero-input feed-through and equalize
//! the five final-VCA small-signal gains.

const UNLINEARIZED_INPUT_DRIVE: f32 = 0.55;
const LINEARIZED_INPUT_DRIVE: f32 = 0.05;
// Intersil Figure 3A plots the diode-linearized transfer at IABC = 650 uA,
// ID = 200 uA and 10 kohm in each input. Its 1 V/div horizontal scale shows
// the rounded current limit at approximately +/-4 V source drive. SD431 uses
// 20 kohm at both final-VCA inputs, doubling that source-voltage span. The
// filter candidate's explicit 2 V/internal-unit conversion therefore places
// the final VCA asymptote at four internal units. A sixth-order norm follows
// the graph's long linear centre and rounded knee without claiming that the
// bitmap is a precision transfer measurement.
const DATASHEET_LINEARIZED_INPUT_RESISTANCE_OHMS: f32 = 10_000.0;
const FINAL_VCA_INPUT_RESISTANCE_OHMS: f32 = 20_000.0;
const DATASHEET_LINEARIZED_LIMIT_VOLTS: f32 = 4.0;
const FINAL_VCA_CIRCUIT_VOLTS_PER_UNIT: f32 = 2.0;
const FINAL_VCA_SOFT_KNEE_ORDER: f32 = 6.0;
const FINAL_VCA_NOMINAL_LIMIT_UNITS: f32 = DATASHEET_LINEARIZED_LIMIT_VOLTS
    * (FINAL_VCA_INPUT_RESISTANCE_OHMS / DATASHEET_LINEARIZED_INPUT_RESISTANCE_OHMS)
    / FINAL_VCA_CIRCUIT_VOLTS_PER_UNIT;
// SD431 converts the nominal 0-5 V CEM3310 amplifier-envelope output to the
// final CA3280 IABC current through R4495 + R4533 and grounded-base PNP Q410.
// Fairchild's 2N4250 curve is approximately 0.56 V at 100 uA and rises by one
// silicon thermal slope per e-fold. The implicit diode-plus-resistor equation
// is solved as x + ln(x) = z (the logarithmic Lambert-W form) without adding a
// sample-rate-dependent smoothing approximation.
const ENVELOPE_NOMINAL_PEAK_VOLTS: f32 = 5.0;
const ENVELOPE_MAXIMUM_PEAK_VOLTS: f32 = 5.3;
// The direct filter-envelope half of SD431 U422 is reconstructed as a current
// path into U433 rather than as a normalized VCA followed by an arbitrary
// octave range. The still-provisional common S/H boundary supplies 0-5 V to
// R449. R451 programs the linearizing diodes across the nominal +/-15 V
// supply, and the serviced balance network gives the inverting input its AC
// return through R450 and R453 plus the 100k trimmer's centre Thevenin value.
// Intersil gives RD = 52 / ID(mA) * 1.34 for the diode network. U433's 100k
// common-CV resistor defines 10 uA as one octave of filter control current.
const FILTER_ENVELOPE_AMOUNT_CV_SPAN_VOLTS: f32 = 5.0;
const FILTER_ENVELOPE_IABC_RESISTANCE_OHMS: f32 = 4_750.0;
const FILTER_ENVELOPE_DIODE_RESISTANCE_OHMS: f32 = 121_000.0;
const FILTER_ENVELOPE_SOURCE_RESISTANCE_OHMS: f32 = 475_000.0;
const FILTER_ENVELOPE_BALANCE_SERIES_RESISTANCE_OHMS: f32 = 475_000.0;
const FILTER_ENVELOPE_INPUT_RETURN_RESISTANCE_OHMS: f32 = 47_500.0;
const FILTER_ENVELOPE_BALANCE_TRIM_THEVENIN_OHMS: f32 = 25_000.0;
const FILTER_ENVELOPE_DIODE_DYNAMIC_OHM_AMPS: f32 = 52.0e-3 * 1.34;
const FILTER_ENVELOPE_SUPPLY_SPAN_VOLTS: f32 = 30.0;
const FILTER_SUM_COMMON_INPUT_RESISTANCE_OHMS: f32 = 100_000.0;
const VCA_CONTROL_SERIES_RESISTANCE_OHMS: f32 = 3_300.0 + 3_300.0;
#[cfg(test)]
const Q410_REFERENCE_CURRENT_AMPS: f32 = 100.0e-6;
#[cfg(test)]
const Q410_REFERENCE_VBE_VOLTS: f32 = 0.56;
const Q410_THERMAL_VOLTAGE_VOLTS: f32 = 0.026;
const Q410_SATURATION_CURRENT_AMPS: f32 = 4.425_527e-14;
const Q410_NOMINAL_CONTROL_CURRENT_AMPS: f32 = 665.262_1e-6;
const FINAL_VCA_MAXIMUM_CONTROL_RATIO: f32 = 1.067_936_5;
// SD430 repeats the grounded-base 2N4250 conversion for the master CA3280,
// but Q411 uses R4542 + R4541 = 9.4 kohm. PCB1's R113 is driven across the
// five-volt analog control domain, giving approximately 468 uA at full volume.
const MASTER_VOLUME_MAXIMUM_CV_VOLTS: f32 = 5.0;
const MASTER_VCA_CONTROL_SERIES_RESISTANCE_OHMS: f32 = 4_700.0 + 4_700.0;
const MASTER_VCA_NOMINAL_CONTROL_CURRENT_AMPS: f32 = 468.071_3e-6;
// U479 is close to the Figure 3A operating point: R4561 supplies about 212 uA
// to the diode terminal against the plotted 200 uA. Reading the graph's centre
// gives approximately 100 uA/V with 10 kohm inputs at 650 uA IABC. SD430 uses
// 15 kohm and the 468 uA Q411 endpoint, then develops output voltage across
// R4562 || R4541 = 16.667 kohm. The resulting full-volume small-signal voltage
// gain is approximately 0.8. These graph-derived values remain explicit
// candidate anchors rather than hidden host normalization.
const MASTER_VCA_DATASHEET_CONTROL_CURRENT_AMPS: f32 = 650.0e-6;
const MASTER_VCA_DATASHEET_INPUT_RESISTANCE_OHMS: f32 = 10_000.0;
const MASTER_VCA_DATASHEET_SLOPE_AMPS_PER_VOLT: f32 = 100.0e-6;
const MASTER_VCA_INPUT_RESISTANCE_OHMS: f32 = 15_000.0;
#[cfg(test)]
const MASTER_VCA_DIODE_RESISTANCE_OHMS: f32 = 68_000.0;
#[cfg(test)]
const MASTER_VCA_POSITIVE_RAIL_VOLTS: f32 = 15.0;
#[cfg(test)]
const MASTER_VCA_DIODE_DROP_VOLTS: f32 = 0.6;
const MASTER_VCA_OUTPUT_LOAD_OHMS: f32 = 1.0 / (1.0 / 20_000.0 + 1.0 / 100_000.0);
const MASTER_VCA_NOMINAL_VOLTAGE_GAIN: f32 = MASTER_VCA_DATASHEET_SLOPE_AMPS_PER_VOLT
    * (MASTER_VCA_DATASHEET_INPUT_RESISTANCE_OHMS / MASTER_VCA_INPUT_RESISTANCE_OHMS)
    * (MASTER_VCA_NOMINAL_CONTROL_CURRENT_AMPS / MASTER_VCA_DATASHEET_CONTROL_CURRENT_AMPS)
    * MASTER_VCA_OUTPUT_LOAD_OHMS;
pub const MASTER_VCA_VOLTAGE_GAIN: f32 = MASTER_VCA_NOMINAL_VOLTAGE_GAIN;
const MASTER_VCA_NOMINAL_LIMIT_UNITS: f32 = DATASHEET_LINEARIZED_LIMIT_VOLTS
    * (MASTER_VCA_INPUT_RESISTANCE_OHMS / DATASHEET_LINEARIZED_INPUT_RESISTANCE_OHMS)
    / FINAL_VCA_CIRCUIT_VOLTS_PER_UNIT;
// TM1000D.2 section 2-5 gives approximately 100 kohm for a CA3280 input with
// its linearizing-diode terminal cut off. SD431 feeds saw/triangle through
// 150 kohm and pulse through 200 kohm. Conductances are normalized to the
// populated 150 kohm path so the existing single-saw calibration remains the
// sole circuit-to-host level anchor.
const UNLINEARIZED_INPUT_RESISTANCE_OHMS: f32 = 100_000.0;
const MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS: f32 = 150_000.0;
const MIXER_INPUT_CONDUCTANCE_RATIO: f32 =
    MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS / UNLINEARIZED_INPUT_RESISTANCE_OHMS;

// SD334's two halves of U378 sum their output currents into R3113. The source
// mix S/H spans 0-10 V. Q307 converts the noise side through R3116, while Q309
// receives the complementary voltage against the 10 V Thevenin source formed
// by R3128/R3130. Their 3.3 kohm collector resistors preserve compliance but
// do not set the grounded-base emitter current.
const WHEEL_MOD_CONTROL_RANGE_VOLTS: f32 = 10.0;
const WHEEL_MOD_NOISE_CONTROL_RESISTANCE_OHMS: f32 = 8_200.0;
const WHEEL_MOD_LFO_DIVIDER_HIGH_OHMS: f32 = 10_000.0;
const WHEEL_MOD_LFO_DIVIDER_LOW_OHMS: f32 = 20_000.0;
const WHEEL_MOD_LFO_CONTROL_RESISTANCE_OHMS: f32 =
    1.0 / (1.0 / WHEEL_MOD_LFO_DIVIDER_HIGH_OHMS + 1.0 / WHEEL_MOD_LFO_DIVIDER_LOW_OHMS);

// U380's selected LFO sum is represented relative to a nominal 10 Vpp saw.
// U374's pink-noise value already contains its 100k/47k closed-loop gain; the
// MM5837 data sheet guarantees each output level within 1.5 V of a 15 V supply
// rail, establishing at least 12 Vpp, or 6 V peak about the centred signal.
const WHEEL_MOD_LFO_SOURCE_VOLTS_PER_UNIT: f32 = 5.0;
const WHEEL_MOD_NOISE_SOURCE_VOLTS_PER_UNIT: f32 = 6.0;
const WHEEL_MOD_LFO_INPUT_RESISTANCE_OHMS: f32 = 160_000.0;
const WHEEL_MOD_NOISE_INPUT_RESISTANCE_OHMS: f32 = 20_000.0;
const WHEEL_MOD_INPUT_SHUNT_OHMS: f32 = 330.0;
const WHEEL_MOD_OUTPUT_LOAD_OHMS: f32 = 10_000.0;

// CA3280 data-sheet typicals: 16 mS at 1 mA IABC and 410 uA peak output at
// 500 uA IABC. Their ratio supplies both the small-signal slope and the
// unlinearized current limit without a host-level saturation constant.
const CA3280_TRANSCONDUCTANCE_SIEMENS_PER_AMP: f32 = 16.0;
const CA3280_PEAK_OUTPUT_CURRENT_RATIO: f32 = 410.0 / 500.0;
#[cfg(test)]
const DATASHEET_MINIMUM_PEAK_CURRENT_RATIO: f32 = 0.70;
#[cfg(test)]
const DATASHEET_MAXIMUM_PEAK_CURRENT_RATIO: f32 = 1.30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MixerChannel {
    OscillatorA,
    OscillatorB,
}

#[derive(Clone, Copy, Debug)]
struct OtaHalfProfile {
    transconductance_ratio: f32,
    input_drive_ratio: f32,
}

#[derive(Clone, Copy, Debug)]
struct MixerProfile {
    oscillator_a: OtaHalfProfile,
    oscillator_b: OtaHalfProfile,
}

#[derive(Clone, Copy, Debug)]
struct EnvelopeAmountProfile {
    direct_filter: OtaHalfProfile,
    poly_mod: OtaHalfProfile,
}

// One dual OTA per voice card. The conservative deterministic spread stays
// well inside the data-sheet 0.70-1.30 peak-output-current ratio. Both halves
// of a package remain close, while no two physical packages collapse to the
// same transfer.
const MIXER_PROFILES: [MixerProfile; 5] = [
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(0.965, 1.055),
        oscillator_b: OtaHalfProfile::new(0.982, 1.027),
    },
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(1.018, 0.973),
        oscillator_b: OtaHalfProfile::new(1.006, 0.991),
    },
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(0.992, 1.009),
        oscillator_b: OtaHalfProfile::new(1.011, 0.982),
    },
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(1.036, 0.945),
        oscillator_b: OtaHalfProfile::new(1.021, 0.967),
    },
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(0.978, 1.033),
        oscillator_b: OtaHalfProfile::new(0.995, 1.018),
    },
];

// The service procedure separately trims each final voice level. Therefore
// these profiles intentionally retain unity small-signal gain and vary only
// the strong-signal knee left after diode linearization.
const FINAL_VCA_PROFILES: [OtaHalfProfile; 5] = [
    OtaHalfProfile::new(1.0, 1.040),
    OtaHalfProfile::new(1.0, 0.956),
    OtaHalfProfile::new(1.0, 1.018),
    OtaHalfProfile::new(1.0, 0.978),
    OtaHalfProfile::new(1.0, 1.009),
];

// Both halves of U422 on each voice card: direct filter-envelope amount and
// the inverted filter-envelope contribution to Poly Mod. The service trims
// cancel offsets, not the small remaining transconductance spread.
const ENVELOPE_AMOUNT_PROFILES: [EnvelopeAmountProfile; 5] = [
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(0.984, 1.018),
        poly_mod: OtaHalfProfile::new(0.997, 0.991),
    },
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(1.012, 0.973),
        poly_mod: OtaHalfProfile::new(0.989, 1.027),
    },
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(0.995, 1.009),
        poly_mod: OtaHalfProfile::new(1.015, 0.982),
    },
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(1.021, 0.956),
        poly_mod: OtaHalfProfile::new(1.008, 0.967),
    },
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(0.991, 1.033),
        poly_mod: OtaHalfProfile::new(1.018, 1.009),
    },
];

// One unlinearized oscillator-B Poly Mod amount OTA per voice card.
const POLY_MOD_OSCILLATOR_B_PROFILES: [OtaHalfProfile; 5] = [
    OtaHalfProfile::new(0.976, 1.040),
    OtaHalfProfile::new(1.019, 0.970),
    OtaHalfProfile::new(0.991, 1.010),
    OtaHalfProfile::new(1.028, 0.950),
    OtaHalfProfile::new(1.004, 1.020),
];

// U378 is the common dual OTA whose two halves move in opposite directions
// under the Wheel Mod source-mix CV. Its balance trimmers remove zero-input
// offset while retaining the two real transfer paths.
const WHEEL_MOD_LFO_PROFILE: OtaHalfProfile = OtaHalfProfile::new(0.993, 1.018);
const WHEEL_MOD_NOISE_PROFILE: OtaHalfProfile = OtaHalfProfile::new(1.007, 0.982);

const COMMON_NOISE_PROFILE: OtaHalfProfile = OtaHalfProfile::new(0.987, 1.036);
const MASTER_VCA_PROFILE: OtaHalfProfile = OtaHalfProfile::new(1.0, 0.991);

impl OtaHalfProfile {
    const fn new(transconductance_ratio: f32, input_drive_ratio: f32) -> Self {
        Self {
            transconductance_ratio,
            input_drive_ratio,
        }
    }
}

/// One half of the dual, unlinearized oscillator-level OTA on a voice card.
pub fn oscillator_mixer(
    input: f32,
    control: f32,
    voice_index: usize,
    channel: MixerChannel,
) -> f32 {
    let profile = MIXER_PROFILES[voice_index % MIXER_PROFILES.len()];
    let half = match channel {
        MixerChannel::OscillatorA => profile.oscillator_a,
        MixerChannel::OscillatorB => profile.oscillator_b,
    };
    ota_transfer(input, control, UNLINEARIZED_INPUT_DRIVE, half)
}

/// One oscillator mixer half including the finite CA3280 input loading shared
/// by every simultaneously selected waveform resistor.
///
/// `source_conductance` is relative to one 150 kohm path: saw and triangle are
/// 1.0 each, while the populated 200 kohm pulse path is 0.75. The normalization
/// deliberately leaves one selected saw unchanged, avoiding a second unknown
/// circuit-volts-to-host calibration.
pub fn oscillator_mixer_loaded(
    input: f32,
    source_conductance: f32,
    control: f32,
    voice_index: usize,
    channel: MixerChannel,
) -> f32 {
    if !input.is_finite() || !source_conductance.is_finite() || source_conductance <= 0.0 {
        return 0.0;
    }
    let reference_loaded_conductance = MIXER_INPUT_CONDUCTANCE_RATIO + 1.0;
    let selected_loaded_conductance = MIXER_INPUT_CONDUCTANCE_RATIO + source_conductance;
    oscillator_mixer(
        input * reference_loaded_conductance / selected_loaded_conductance,
        control,
        voice_index,
        channel,
    )
}

/// The single common noise-level OTA before noise reaches all five filters.
pub fn common_noise(input: f32, control: f32) -> f32 {
    ota_transfer(
        input,
        control,
        UNLINEARIZED_INPUT_DRIVE,
        COMMON_NOISE_PROFILE,
    )
}

/// The common dual-OTA current mixer feeding the physical modulation wheel.
///
/// The returned value is the reconstructed voltage across SD334 R3113, not a
/// normalized host bus. Q307/Q309 create the complementary IABC currents and
/// the two populated input dividers set each unlinearized CA3280 drive.
pub fn wheel_mod_source(lfo: f32, noise: f32, source_mix: f32) -> f32 {
    if !lfo.is_finite() || !noise.is_finite() || !source_mix.is_finite() {
        return 0.0;
    }
    let source_mix_cv = source_mix.clamp(0.0, 1.0) * WHEEL_MOD_CONTROL_RANGE_VOLTS;
    let lfo_iabc = grounded_base_2n4250_collector_current_amps(
        WHEEL_MOD_CONTROL_RANGE_VOLTS - source_mix_cv,
        WHEEL_MOD_LFO_CONTROL_RESISTANCE_OHMS,
    );
    let noise_iabc = grounded_base_2n4250_collector_current_amps(
        source_mix_cv,
        WHEEL_MOD_NOISE_CONTROL_RESISTANCE_OHMS,
    );

    unlinearized_ota_loaded_voltage(
        lfo * WHEEL_MOD_LFO_SOURCE_VOLTS_PER_UNIT,
        WHEEL_MOD_LFO_INPUT_RESISTANCE_OHMS,
        lfo_iabc,
        WHEEL_MOD_LFO_PROFILE,
    ) + unlinearized_ota_loaded_voltage(
        noise * WHEEL_MOD_NOISE_SOURCE_VOLTS_PER_UNIT,
        WHEEL_MOD_NOISE_INPUT_RESISTANCE_OHMS,
        noise_iabc,
        WHEEL_MOD_NOISE_PROFILE,
    )
}

/// Filter-cutoff displacement produced by the direct envelope half of U422.
///
/// The result is in octaves because it is expressed relative to the 10 uA
/// that a one-volt common filter CV sends through U433's populated 100 kohm
/// input. Only the 0-5 V amount-cell span remains a replaceable candidate.
pub fn filter_envelope_cutoff_octaves(envelope: f32, amount: f32, voice_index: usize) -> f32 {
    if !envelope.is_finite() || envelope <= 0.0 || !amount.is_finite() || amount <= 0.0 {
        return 0.0;
    }

    let profile =
        ENVELOPE_AMOUNT_PROFILES[voice_index % ENVELOPE_AMOUNT_PROFILES.len()].direct_filter;
    let iabc_amps = amount.clamp(0.0, 1.0) * FILTER_ENVELOPE_AMOUNT_CV_SPAN_VOLTS
        / FILTER_ENVELOPE_IABC_RESISTANCE_OHMS;
    let diode_current_amps =
        FILTER_ENVELOPE_SUPPLY_SPAN_VOLTS / FILTER_ENVELOPE_DIODE_RESISTANCE_OHMS;
    let diode_dynamic_resistance_ohms = FILTER_ENVELOPE_DIODE_DYNAMIC_OHM_AMPS / diode_current_amps;
    let balance_return_resistance_ohms = 1.0
        / (1.0 / FILTER_ENVELOPE_INPUT_RETURN_RESISTANCE_OHMS
            + 1.0
                / (FILTER_ENVELOPE_BALANCE_SERIES_RESISTANCE_OHMS
                    + FILTER_ENVELOPE_BALANCE_TRIM_THEVENIN_OHMS));
    let input_loop_resistance_ohms =
        FILTER_ENVELOPE_SOURCE_RESISTANCE_OHMS + balance_return_resistance_ohms;
    let envelope_volts = (envelope * ENVELOPE_NOMINAL_PEAK_VOLTS).min(ENVELOPE_MAXIMUM_PEAK_VOLTS);
    let differential_input_volts = envelope_volts * diode_dynamic_resistance_ohms
        / (input_loop_resistance_ohms + diode_dynamic_resistance_ohms)
        * profile.input_drive_ratio;
    let small_signal_output_current_amps =
        iabc_amps * CA3280_TRANSCONDUCTANCE_SIEMENS_PER_AMP * differential_input_volts;
    let peak_output_current_amps = iabc_amps * CA3280_PEAK_OUTPUT_CURRENT_RATIO;
    let output_current_amps = peak_output_current_amps
        * libm::tanhf(small_signal_output_current_amps / peak_output_current_amps)
        * profile.transconductance_ratio;
    output_current_amps * FILTER_SUM_COMMON_INPUT_RESISTANCE_OHMS
}

/// The second, inverted-at-the-summing-node U422 envelope amount path.
pub fn poly_mod_filter_envelope(input: f32, control: f32, voice_index: usize) -> f32 {
    ota_transfer(
        input,
        control,
        LINEARIZED_INPUT_DRIVE,
        ENVELOPE_AMOUNT_PROFILES[voice_index % ENVELOPE_AMOUNT_PROFILES.len()].poly_mod,
    )
}

/// The unlinearized oscillator-B waveform amount OTA in one Poly Mod path.
pub fn poly_mod_oscillator_b(input: f32, control: f32, voice_index: usize) -> f32 {
    ota_transfer(
        input,
        control,
        UNLINEARIZED_INPUT_DRIVE,
        POLY_MOD_OSCILLATOR_B_PROFILES[voice_index % POLY_MOD_OSCILLATOR_B_PROFILES.len()],
    )
}

/// Convert the CEM3310 amplifier-envelope voltage into the IABC ratio applied
/// to the final CA3280 by Q410 and the two populated 3.3 kohm resistors.
///
/// The returned value is normalized to the current produced by the nominal
/// 5 V envelope peak, so the existing serviced voice-level anchor is retained.
pub fn amplifier_envelope_control(envelope: f32) -> f32 {
    if !envelope.is_finite() || envelope <= 0.0 {
        return 0.0;
    }
    let envelope_volts = (envelope * ENVELOPE_NOMINAL_PEAK_VOLTS).min(ENVELOPE_MAXIMUM_PEAK_VOLTS);
    grounded_base_2n4250_collector_current_amps(envelope_volts, VCA_CONTROL_SERIES_RESISTANCE_OHMS)
        / Q410_NOMINAL_CONTROL_CURRENT_AMPS
}

/// The diode-linearized and service-calibrated final VCA on one voice card.
pub fn final_voice(input: f32, control: f32, voice_index: usize) -> f32 {
    final_voice_transfer(
        input,
        control,
        FINAL_VCA_PROFILES[voice_index % FINAL_VCA_PROFILES.len()],
    )
}

/// Convert the smoothed R113 wiper voltage through Q411 and the two populated
/// 4.7 kohm resistors to the master CA3280's normalized IABC current.
pub fn master_volume_control_from_cv(volume_cv_volts: f32) -> f32 {
    if !volume_cv_volts.is_finite() || volume_cv_volts <= 0.0 {
        return 0.0;
    }
    grounded_base_2n4250_collector_current_amps(
        volume_cv_volts.min(MASTER_VOLUME_MAXIMUM_CV_VOLTS),
        MASTER_VCA_CONTROL_SERIES_RESISTANCE_OHMS,
    ) / MASTER_VCA_NOMINAL_CONTROL_CURRENT_AMPS
}

/// The diode-linearized common VCA driven by the reconstructed Q411 current.
pub fn master_output(input: f32, control_current_ratio: f32) -> f32 {
    if control_current_ratio <= 0.0 || !control_current_ratio.is_finite() || !input.is_finite() {
        return 0.0;
    }
    let limit = MASTER_VCA_NOMINAL_LIMIT_UNITS / MASTER_VCA_PROFILE.input_drive_ratio;
    sixth_order_limited(input, limit)
        * control_current_ratio.clamp(0.0, 1.0)
        * MASTER_VCA_NOMINAL_VOLTAGE_GAIN
        * MASTER_VCA_PROFILE.transconductance_ratio
}

fn ota_transfer(input: f32, control: f32, nominal_drive: f32, profile: OtaHalfProfile) -> f32 {
    ota_transfer_limited(input, control, nominal_drive, profile, 1.0)
}

fn ota_transfer_limited(
    input: f32,
    control: f32,
    nominal_drive: f32,
    profile: OtaHalfProfile,
    maximum_control: f32,
) -> f32 {
    if control <= 0.0 || !control.is_finite() || !input.is_finite() {
        return 0.0;
    }
    let drive = nominal_drive * profile.input_drive_ratio;
    let current = libm::tanhf(input * drive) / drive;
    current * control.clamp(0.0, maximum_control) * profile.transconductance_ratio
}

fn unlinearized_ota_loaded_voltage(
    source_volts: f32,
    source_resistance_ohms: f32,
    iabc_amps: f32,
    profile: OtaHalfProfile,
) -> f32 {
    if !source_volts.is_finite()
        || !source_resistance_ohms.is_finite()
        || source_resistance_ohms <= 0.0
        || !iabc_amps.is_finite()
        || iabc_amps <= 0.0
    {
        return 0.0;
    }

    let differential_input_volts = source_volts * WHEEL_MOD_INPUT_SHUNT_OHMS
        / (source_resistance_ohms + WHEEL_MOD_INPUT_SHUNT_OHMS)
        * profile.input_drive_ratio;
    let peak_output_current_amps = iabc_amps * CA3280_PEAK_OUTPUT_CURRENT_RATIO;
    let small_signal_output_current_amps =
        iabc_amps * CA3280_TRANSCONDUCTANCE_SIEMENS_PER_AMP * differential_input_volts;
    let output_current_amps = peak_output_current_amps
        * libm::tanhf(small_signal_output_current_amps / peak_output_current_amps)
        * profile.transconductance_ratio;
    output_current_amps * WHEEL_MOD_OUTPUT_LOAD_OHMS
}

fn final_voice_transfer(input: f32, control: f32, profile: OtaHalfProfile) -> f32 {
    if control <= 0.0 || !control.is_finite() || !input.is_finite() {
        return 0.0;
    }

    // A larger drive ratio means an earlier knee, matching the population
    // convention used by the other OTA profiles. Evaluate the sixth-order
    // norm on either side of unity to avoid overflow for arbitrary finite host
    // input while preserving odd symmetry and a finite current asymptote.
    let limit = FINAL_VCA_NOMINAL_LIMIT_UNITS / profile.input_drive_ratio;
    sixth_order_limited(input, limit)
        * control.clamp(0.0, FINAL_VCA_MAXIMUM_CONTROL_RATIO)
        * profile.transconductance_ratio
}

fn sixth_order_limited(input: f32, limit: f32) -> f32 {
    let magnitude = input.abs();
    let ratio = magnitude / limit;
    let limited_magnitude = if ratio <= 1.0 {
        let ratio_squared = ratio * ratio;
        let ratio_sixth = ratio_squared * ratio_squared * ratio_squared;
        magnitude / libm::powf(1.0 + ratio_sixth, 1.0 / FINAL_VCA_SOFT_KNEE_ORDER)
    } else {
        let inverse = 1.0 / ratio;
        let inverse_squared = inverse * inverse;
        let inverse_sixth = inverse_squared * inverse_squared * inverse_squared;
        limit / libm::powf(1.0 + inverse_sixth, 1.0 / FINAL_VCA_SOFT_KNEE_ORDER)
    };
    input.signum() * limited_magnitude
}

fn grounded_base_2n4250_collector_current_amps(
    drive_volts: f32,
    series_resistance_ohms: f32,
) -> f32 {
    if !drive_volts.is_finite()
        || drive_volts <= 0.0
        || !series_resistance_ohms.is_finite()
        || series_resistance_ohms <= 0.0
    {
        return 0.0;
    }

    // I*R + nVt*ln(I/Is) = V. With x = I*R/nVt this becomes
    // x + ln(x) = V/nVt + ln(R*Is/nVt). Three Newton steps are sufficient
    // over the admitted 0-5.3 V CEM3310 range.
    let z = drive_volts / Q410_THERMAL_VOLTAGE_VOLTS
        + libm::logf(
            series_resistance_ohms * Q410_SATURATION_CURRENT_AMPS / Q410_THERMAL_VOLTAGE_VOLTS,
        );
    let mut normalized_current = if z >= 1.0 {
        (z - libm::logf(z)).max(f32::MIN_POSITIVE)
    } else {
        libm::expf(z).max(f32::MIN_POSITIVE)
    };
    for _ in 0..3 {
        let residual = normalized_current + libm::logf(normalized_current) - z;
        let slope = 1.0 + 1.0 / normalized_current;
        normalized_current = (normalized_current - residual / slope).max(f32::MIN_POSITIVE);
    }
    normalized_current * Q410_THERMAL_VOLTAGE_VOLTS / series_resistance_ohms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_bias_current_closes_every_physical_vca() {
        for input in [-8.0, -1.0, 0.0, 1.0, 8.0] {
            for voice in 0..5 {
                assert_eq!(
                    oscillator_mixer(input, 0.0, voice, MixerChannel::OscillatorA),
                    0.0
                );
                assert_eq!(
                    oscillator_mixer(input, 0.0, voice, MixerChannel::OscillatorB),
                    0.0
                );
                assert_eq!(final_voice(input, 0.0, voice), 0.0);
                assert_eq!(filter_envelope_cutoff_octaves(input, 0.0, voice), 0.0);
                assert_eq!(poly_mod_filter_envelope(input, 0.0, voice), 0.0);
                assert_eq!(poly_mod_oscillator_b(input, 0.0, voice), 0.0);
            }
            assert_eq!(common_noise(input, 0.0), 0.0);
            assert_eq!(master_output(input, 0.0), 0.0);
        }
    }

    #[test]
    fn control_current_changes_gain_monotonically() {
        let low = oscillator_mixer(0.5, 0.25, 2, MixerChannel::OscillatorA).abs();
        let middle = oscillator_mixer(0.5, 0.5, 2, MixerChannel::OscillatorA).abs();
        let high = oscillator_mixer(0.5, 1.0, 2, MixerChannel::OscillatorA).abs();
        assert!(low < middle && middle < high);
    }

    #[test]
    fn direct_filter_envelope_reaches_u433_through_the_populated_current_path() {
        let diode_current =
            FILTER_ENVELOPE_SUPPLY_SPAN_VOLTS / FILTER_ENVELOPE_DIODE_RESISTANCE_OHMS;
        let diode_resistance = FILTER_ENVELOPE_DIODE_DYNAMIC_OHM_AMPS / diode_current;
        assert!((240.0e-6..=255.0e-6).contains(&diode_current));
        assert!((270.0..=295.0).contains(&diode_resistance));

        for voice in 0..5 {
            let quarter = filter_envelope_cutoff_octaves(1.0, 0.25, voice);
            let half = filter_envelope_cutoff_octaves(1.0, 0.5, voice);
            let full = filter_envelope_cutoff_octaves(1.0, 1.0, voice);
            assert!(quarter > 0.0 && quarter < half && half < full);
            assert!((4.2..=4.9).contains(&full));
            assert_eq!(filter_envelope_cutoff_octaves(0.0, 1.0, voice), 0.0);
        }
    }

    #[test]
    fn q410_reconstructs_the_ca3280_datasheet_operating_current() {
        let reconstructed_saturation = Q410_REFERENCE_CURRENT_AMPS
            * libm::expf(-Q410_REFERENCE_VBE_VOLTS / Q410_THERMAL_VOLTAGE_VOLTS);
        assert!((reconstructed_saturation / Q410_SATURATION_CURRENT_AMPS - 1.0).abs() < 1.0e-6);
        let nominal = grounded_base_2n4250_collector_current_amps(
            ENVELOPE_NOMINAL_PEAK_VOLTS,
            VCA_CONTROL_SERIES_RESISTANCE_OHMS,
        );
        assert!((650.0e-6..=680.0e-6).contains(&nominal));
        assert!((nominal / Q410_NOMINAL_CONTROL_CURRENT_AMPS - 1.0).abs() < 1.0e-6);
        let maximum = grounded_base_2n4250_collector_current_amps(
            ENVELOPE_MAXIMUM_PEAK_VOLTS,
            VCA_CONTROL_SERIES_RESISTANCE_OHMS,
        ) / nominal;
        assert!((maximum / FINAL_VCA_MAXIMUM_CONTROL_RATIO - 1.0).abs() < 1.0e-6);

        for envelope_volts in [0.01, 0.1, 0.5, 1.0, 2.5, 5.0, 5.3] {
            let current = grounded_base_2n4250_collector_current_amps(
                envelope_volts,
                VCA_CONTROL_SERIES_RESISTANCE_OHMS,
            );
            let junction_voltage =
                Q410_THERMAL_VOLTAGE_VOLTS * libm::logf(current / Q410_SATURATION_CURRENT_AMPS);
            let reconstructed_voltage =
                current * VCA_CONTROL_SERIES_RESISTANCE_OHMS + junction_voltage;
            assert!((reconstructed_voltage - envelope_volts).abs() < 1.0e-5);
        }
    }

    #[test]
    fn amplifier_envelope_to_iabc_is_monotonic_and_has_a_silicon_knee() {
        let mut previous = amplifier_envelope_control(0.0);
        assert_eq!(previous, 0.0);
        for code in 1..=1_060 {
            let current = amplifier_envelope_control(code as f32 / 1_000.0);
            assert!(current > previous);
            previous = current;
        }
        assert_eq!(amplifier_envelope_control(1.0), 1.0);
        assert!(amplifier_envelope_control(0.5) < 0.45);
        assert!(amplifier_envelope_control(0.1) < 0.01);
        let maximum = amplifier_envelope_control(1.06);
        assert!(maximum > 1.06);
        assert!(maximum < 1.08);
        assert_eq!(amplifier_envelope_control(2.0), maximum);
    }

    #[test]
    fn q411_reconstructs_the_master_ca3280_control_current() {
        let maximum = grounded_base_2n4250_collector_current_amps(
            MASTER_VOLUME_MAXIMUM_CV_VOLTS,
            MASTER_VCA_CONTROL_SERIES_RESISTANCE_OHMS,
        );
        assert!((460.0e-6..=475.0e-6).contains(&maximum));
        assert!((maximum / MASTER_VCA_NOMINAL_CONTROL_CURRENT_AMPS - 1.0).abs() < 1.0e-6);
        assert_eq!(master_volume_control_from_cv(0.0), 0.0);
        assert!((master_volume_control_from_cv(5.0) - 1.0).abs() < 1.0e-6);
        assert!((master_volume_control_from_cv(10.0) - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn master_volume_current_has_the_grounded_base_silicon_knee() {
        let mut previous = master_volume_control_from_cv(0.0);
        for step in 1..=500 {
            let current = master_volume_control_from_cv(step as f32 * 0.01);
            assert!(current >= previous);
            previous = current;
        }
        assert!(master_volume_control_from_cv(0.5) < 0.01);
        assert!((0.40..0.45).contains(&master_volume_control_from_cv(2.44)));
    }

    #[test]
    fn master_vca_operates_near_the_figure_3a_diode_current() {
        let diode_current = (MASTER_VCA_POSITIVE_RAIL_VOLTS - MASTER_VCA_DIODE_DROP_VOLTS)
            / MASTER_VCA_DIODE_RESISTANCE_OHMS;
        assert!((205.0e-6..=220.0e-6).contains(&diode_current));
        assert!(
            (diode_current / 200.0e-6 - 1.0).abs() < 0.07,
            "diode_current={diode_current}"
        );
    }

    #[test]
    fn populated_master_vca_has_the_graph_derived_voltage_gain() {
        assert!((0.79..=0.81).contains(&MASTER_VCA_NOMINAL_VOLTAGE_GAIN));
        let measured = master_output(1.0e-4, 1.0) / 1.0e-4;
        assert!((measured / MASTER_VCA_NOMINAL_VOLTAGE_GAIN - 1.0).abs() < 1.0e-6);
        assert_eq!(MASTER_VCA_NOMINAL_LIMIT_UNITS, 3.0);
    }

    #[test]
    fn master_vca_transfer_is_odd_monotonic_and_bounded() {
        let limit = MASTER_VCA_NOMINAL_LIMIT_UNITS / MASTER_VCA_PROFILE.input_drive_ratio;
        let expected_asymptote = limit * MASTER_VCA_NOMINAL_VOLTAGE_GAIN;
        let positive = master_output(f32::MAX, 1.0);
        let negative = master_output(-f32::MAX, 1.0);
        assert!(positive.is_finite());
        assert!((positive - expected_asymptote).abs() < 1.0e-5);
        assert!((positive + negative).abs() < 1.0e-6);

        let mut previous = 0.0;
        for step in 1..=8_000 {
            let value = master_output(step as f32 * 0.002, 1.0);
            assert!(value >= previous);
            previous = value;
        }
    }

    #[test]
    fn final_vca_accepts_the_full_bounded_cem3310_population() {
        let maximum = FINAL_VCA_MAXIMUM_CONTROL_RATIO;
        let at_maximum = final_voice(0.25, maximum, 2);
        let at_nominal = final_voice(0.25, 1.0, 2);
        assert!(at_maximum > at_nominal);
        assert!(at_maximum < at_nominal * 1.08);
        assert_eq!(final_voice(0.25, maximum * 2.0, 2), at_maximum);
    }

    #[test]
    fn active_linearizing_diodes_extend_the_input_range() {
        let input = 3.0;
        let unlinearized = oscillator_mixer(input, 1.0, 2, MixerChannel::OscillatorA).abs();
        let linearized = final_voice(input, 1.0, 2).abs();
        let mixer_small_signal =
            oscillator_mixer(0.001, 1.0, 2, MixerChannel::OscillatorA).abs() / 0.001;
        let final_small_signal = final_voice(0.001, 1.0, 2).abs() / 0.001;
        let mixer_retained = unlinearized / (input * mixer_small_signal);
        let final_retained = linearized / (input * final_small_signal);
        assert!(final_retained > 0.97);
        assert!(mixer_retained < 0.65);
    }

    #[test]
    fn populated_input_resistors_scale_the_datasheet_voltage_span() {
        let populated_limit_volts = DATASHEET_LINEARIZED_LIMIT_VOLTS
            * FINAL_VCA_INPUT_RESISTANCE_OHMS
            / DATASHEET_LINEARIZED_INPUT_RESISTANCE_OHMS;
        assert_eq!(populated_limit_volts, 8.0);
        assert_eq!(FINAL_VCA_NOMINAL_LIMIT_UNITS, 4.0);
    }

    #[test]
    fn final_vca_knee_overlaps_the_cem3320_output_range() {
        fn retained_at_vpp(circuit_vpp: f32) -> f32 {
            let peak_internal = circuit_vpp * 0.5 / FINAL_VCA_CIRCUIT_VOLTS_PER_UNIT;
            final_voice(peak_internal, 1.0, 2) / peak_internal
        }

        // The CEM3320 population clips between 10 and 14 Vpp. Figure 3A's
        // scaled knee remains nearly linear at the low end but has begun
        // compressing before the high end, rather than sitting outside it.
        let at_10_vpp = retained_at_vpp(10.0);
        let at_14_vpp = retained_at_vpp(14.0);
        assert!(at_10_vpp > 0.98);
        assert!((0.90..0.97).contains(&at_14_vpp));
        assert!(at_14_vpp < at_10_vpp);
    }

    #[test]
    fn final_vca_preserves_small_signal_gain_and_has_a_finite_asymptote() {
        for (voice, profile) in FINAL_VCA_PROFILES.iter().copied().enumerate() {
            let small_signal_gain = final_voice(1.0e-4, 1.0, voice) / 1.0e-4;
            assert!((small_signal_gain - 1.0).abs() < 1.0e-6);

            let expected_limit = FINAL_VCA_NOMINAL_LIMIT_UNITS / profile.input_drive_ratio;
            let positive = final_voice(f32::MAX, 1.0, voice);
            let negative = final_voice(-f32::MAX, 1.0, voice);
            assert!(positive.is_finite());
            assert!((positive - expected_limit).abs() < 1.0e-5);
            assert!((positive + negative).abs() < 1.0e-6);
        }
    }

    #[test]
    fn final_vca_transfer_is_monotonic_odd_and_profiled() {
        let mut profiled = [0.0; 5];
        for (voice, output) in profiled.iter_mut().enumerate() {
            let mut previous = final_voice(0.0, 1.0, voice);
            for step in 1..=8_000 {
                let input = step as f32 * 0.002;
                let positive = final_voice(input, 1.0, voice);
                let negative = final_voice(-input, 1.0, voice);
                assert!(positive >= previous);
                assert!((positive + negative).abs() < 1.0e-6);
                previous = positive;
            }
            *output = final_voice(6.0, 1.0, voice);
        }
        assert!(profiled.windows(2).any(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn device_population_stays_inside_published_output_bounds() {
        fn assert_inside_bounds(profile: OtaHalfProfile) {
            assert!(
                (DATASHEET_MINIMUM_PEAK_CURRENT_RATIO..=DATASHEET_MAXIMUM_PEAK_CURRENT_RATIO)
                    .contains(&profile.transconductance_ratio)
            );
            assert!((0.90..=1.10).contains(&profile.input_drive_ratio));
        }

        for profile in MIXER_PROFILES {
            for half in [profile.oscillator_a, profile.oscillator_b] {
                assert_inside_bounds(half);
            }
        }
        for profile in FINAL_VCA_PROFILES {
            assert_eq!(profile.transconductance_ratio, 1.0);
            assert_inside_bounds(profile);
        }
        for profile in ENVELOPE_AMOUNT_PROFILES {
            assert_inside_bounds(profile.direct_filter);
            assert_inside_bounds(profile.poly_mod);
        }
        for profile in POLY_MOD_OSCILLATOR_B_PROFILES {
            assert_inside_bounds(profile);
        }
        assert_inside_bounds(WHEEL_MOD_LFO_PROFILE);
        assert_inside_bounds(WHEEL_MOD_NOISE_PROFILE);
        assert_inside_bounds(COMMON_NOISE_PROFILE);
        assert_inside_bounds(MASTER_VCA_PROFILE);
    }

    #[test]
    fn paired_mixer_halves_remain_close_but_not_identical() {
        for profile in MIXER_PROFILES {
            assert!(
                (profile.oscillator_a.transconductance_ratio
                    - profile.oscillator_b.transconductance_ratio)
                    .abs()
                    < 0.03
            );
            assert_ne!(
                profile.oscillator_a.transconductance_ratio,
                profile.oscillator_b.transconductance_ratio
            );
        }
    }

    #[test]
    fn mixer_loading_preserves_the_single_150k_path_anchor() {
        for voice in 0..5 {
            for channel in [MixerChannel::OscillatorA, MixerChannel::OscillatorB] {
                assert_eq!(
                    oscillator_mixer_loaded(0.75, 1.0, 0.6, voice, channel),
                    oscillator_mixer(0.75, 0.6, voice, channel)
                );
            }
        }
    }

    #[test]
    fn parallel_waveform_paths_load_the_finite_mixer_input() {
        let one_path = oscillator_mixer_loaded(0.5, 1.0, 1.0, 2, MixerChannel::OscillatorA);
        let two_equal_paths = oscillator_mixer_loaded(1.0, 2.0, 1.0, 2, MixerChannel::OscillatorA);
        let unloaded_linear_sum = one_path * 2.0;
        assert!(two_equal_paths > one_path);
        assert!(two_equal_paths < unloaded_linear_sum);

        let expected_input_ratio =
            2.0 * (MIXER_INPUT_CONDUCTANCE_RATIO + 1.0) / (MIXER_INPUT_CONDUCTANCE_RATIO + 2.0);
        let small_one = oscillator_mixer_loaded(0.001, 1.0, 1.0, 2, MixerChannel::OscillatorA);
        let small_two = oscillator_mixer_loaded(0.002, 2.0, 1.0, 2, MixerChannel::OscillatorA);
        assert!((small_two / small_one - expected_input_ratio).abs() < 1.0e-5);
    }

    #[test]
    fn pulse_path_uses_its_populated_200k_conductance() {
        let loaded = oscillator_mixer_loaded(0.75, 0.75, 1.0, 2, MixerChannel::OscillatorA);
        let reference = oscillator_mixer(0.75, 1.0, 2, MixerChannel::OscillatorA);
        assert!(loaded > reference);
        assert!(loaded < reference * 1.12);
    }

    #[test]
    fn absent_or_invalid_mixer_sources_are_silent() {
        for source in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                oscillator_mixer_loaded(1.0, source, 1.0, 0, MixerChannel::OscillatorA),
                0.0
            );
        }
    }

    #[test]
    fn paired_envelope_amount_halves_remain_close_but_not_identical() {
        for profile in ENVELOPE_AMOUNT_PROFILES {
            assert!(
                (profile.direct_filter.transconductance_ratio
                    - profile.poly_mod.transconductance_ratio)
                    .abs()
                    < 0.03
            );
            assert_ne!(
                profile.direct_filter.transconductance_ratio,
                profile.poly_mod.transconductance_ratio
            );
        }
    }

    #[test]
    fn wheel_mod_dual_ota_uses_complementary_physical_control_currents() {
        let lfo_only = wheel_mod_source(0.75, -8.0, 0.0);
        assert_eq!(lfo_only, wheel_mod_source(0.75, 8.0, 0.0));
        let noise_only = wheel_mod_source(-8.0, 0.75, 1.0);
        assert_eq!(noise_only, wheel_mod_source(8.0, 0.75, 1.0));
        assert!(lfo_only > 0.0);
        assert!(noise_only > 0.0);

        let low_mix_lfo = wheel_mod_source(0.75, 0.0, 0.25);
        let high_mix_lfo = wheel_mod_source(0.75, 0.0, 0.75);
        assert!(low_mix_lfo > high_mix_lfo);
        let low_mix_noise = wheel_mod_source(0.0, 0.75, 0.25);
        let high_mix_noise = wheel_mod_source(0.0, 0.75, 0.75);
        assert!(low_mix_noise < high_mix_noise);

        for mix in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert_eq!(wheel_mod_source(0.0, 0.0, mix), 0.0);
        }
    }

    #[test]
    fn wheel_mod_voltage_is_set_by_u378_and_r3113() {
        let lfo_volts = wheel_mod_source(1.0, 0.0, 0.0);
        assert!((2.0..2.5).contains(&lfo_volts));

        let maximum_lfo_iabc = grounded_base_2n4250_collector_current_amps(
            WHEEL_MOD_CONTROL_RANGE_VOLTS,
            WHEEL_MOD_LFO_CONTROL_RESISTANCE_OHMS,
        );
        let maximum_current_bound = maximum_lfo_iabc
            * CA3280_PEAK_OUTPUT_CURRENT_RATIO
            * WHEEL_MOD_LFO_PROFILE.transconductance_ratio;
        assert!(lfo_volts.abs() < maximum_current_bound * WHEEL_MOD_OUTPUT_LOAD_OHMS);

        let positive = wheel_mod_source(0.43, 0.0, 0.0);
        let negative = wheel_mod_source(-0.43, 0.0, 0.0);
        assert!((positive + negative).abs() < 1.0e-6);
    }

    #[test]
    fn poly_mod_amount_vcas_are_monotonic_and_mode_correct() {
        for voice in 0..5 {
            let low = poly_mod_oscillator_b(0.7, 0.25, voice).abs();
            let high = poly_mod_oscillator_b(0.7, 0.75, voice).abs();
            assert!(low < high);

            let linearized = poly_mod_filter_envelope(3.0, 1.0, voice);
            let unlinearized = poly_mod_oscillator_b(3.0, 1.0, voice);
            let linearized_small = poly_mod_filter_envelope(0.001, 1.0, voice) / 0.001;
            let unlinearized_small = poly_mod_oscillator_b(0.001, 1.0, voice) / 0.001;
            assert!(linearized / (3.0 * linearized_small) > 0.99);
            assert!(unlinearized / (3.0 * unlinearized_small) < 0.65);
        }
    }

    #[test]
    fn service_calibration_equalizes_final_vca_small_signal_gain() {
        let reference = final_voice(0.001, 1.0, 0);
        for voice in 1..5 {
            assert!((final_voice(0.001, 1.0, voice) - reference).abs() < 1.0e-8);
        }
    }

    #[test]
    fn transfers_are_odd_symmetric_finite_and_profiled() {
        let mut profile_outputs = [0.0; 5];
        for (voice, profile_output) in profile_outputs.iter_mut().enumerate() {
            for index in 0..10_000 {
                let input = index as f32 * 0.002;
                let positive = oscillator_mixer(input, 1.0, voice, MixerChannel::OscillatorA);
                let negative = oscillator_mixer(-input, 1.0, voice, MixerChannel::OscillatorA);
                assert!(positive.is_finite());
                assert!((positive + negative).abs() < 1.0e-6);
            }
            *profile_output = oscillator_mixer(2.0, 1.0, voice, MixerChannel::OscillatorA);
        }
        assert!(profile_outputs.windows(2).any(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn non_finite_controls_and_inputs_are_silenced() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(master_output(invalid, 1.0), 0.0);
            assert_eq!(master_output(1.0, invalid), 0.0);
        }
    }
}
