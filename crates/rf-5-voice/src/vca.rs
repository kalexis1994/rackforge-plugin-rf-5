//! Physical CA3280 boundaries in the RF-5 audio path.
//!
//! Each voice card uses both halves of one CA3280 for oscillator A/B level,
//! with the linearizing-diode terminal cut off. A separate linearized half is
//! the final voice VCA. The common noise level and master volume each have
//! their own physical CA3280 stage. Balance and voice-volume trimmers are
//! treated as serviced, so they cancel zero-input feed-through and equalize
//! the five final-VCA small-signal gains.

// Intersil Figure 3A plots the diode-linearized transfer at IABC = 650 uA,
// ID = 200 uA and 10 kohm in each input: linear until the input current
// reaches +/-ID (+/-4 V across the 20 kohm loop), where the output settles at
// the published peak. The linear range therefore scales with ID times the
// populated input loop. The ID terminal feeds an internal current mirror
// referred to the -15 V pin, so each 68 kohm programming resistor from +15 V
// sets about 424 uA. A sixth-order norm follows the graph's long linear
// centre and rounded knee. Audio values at this boundary are circuit volts.
const LINEARIZED_ID_PROGRAMMING_VOLTS: f32 = 30.0 - 2.0 * 0.6;
const FINAL_VCA_DIODE_RESISTANCE_OHMS: f32 = 68_000.0;
const FINAL_VCA_DIODE_CURRENT_AMPS: f32 =
    LINEARIZED_ID_PROGRAMMING_VOLTS / FINAL_VCA_DIODE_RESISTANCE_OHMS;
// R4548 and R4547 (voice 1) are 20 kohm on the two inputs.
const FINAL_VCA_INPUT_LOOP_OHMS: f32 = 20_000.0 + 20_000.0;
#[cfg(not(feature = "fast-math"))]
const FINAL_VCA_SOFT_KNEE_ORDER: f32 = 6.0;
const FINAL_VCA_NOMINAL_LIMIT_VOLTS: f32 = FINAL_VCA_DIODE_CURRENT_AMPS * FINAL_VCA_INPUT_LOOP_OHMS;
// The OTA drives its output current into R4529, the 25 kohm VOL rheostat that
// trim 4-22 leaves at or near maximum. Toward the summer the card is therefore
// a Thevenin source of I x 25 kohm behind 25 kohm plus its 39 kohm summing
// resistor. At the nominal Q410 current the Figure 3A law gives:
const VOICE_VOLUME_TRIM_OHMS: f32 = 25_000.0;
pub const VOICE_SUMMER_SOURCE_OHMS: f32 = VOICE_VOLUME_TRIM_OHMS + 39_000.0;
pub const FINAL_VCA_THEVENIN_GAIN: f32 = CA3280_LINEARIZED_CURRENT_TRANSFER
    * (Q410_NOMINAL_CONTROL_CURRENT_AMPS / FINAL_VCA_DIODE_CURRENT_AMPS)
    * VOICE_VOLUME_TRIM_OHMS
    / FINAL_VCA_INPUT_LOOP_OHMS;
// SD431 converts the nominal 0-5 V CEM3310 amplifier-envelope output to the
// final CA3280 IABC current through R4495 + R4533 and grounded-base PNP Q410.
// Fairchild's 2N4250 curve is approximately 0.56 V at 100 uA and rises by one
// silicon thermal slope per e-fold. The implicit diode-plus-resistor equation
// is solved as x + ln(x) = z (the logarithmic Lambert-W form) without adding a
// sample-rate-dependent smoothing approximation.
const ENVELOPE_NOMINAL_PEAK_VOLTS: f32 = 5.0;
const ENVELOPE_MAXIMUM_PEAK_VOLTS: f32 = 5.3;
// SD332's DAC steps 1/12 V per stored code, reaching 10.000 V at the panel's
// code-120 ceiling. On SD333, Q301/Q303/Q304 convert the three amount S/H
// voltages into collector current before the voice cards. Their populated
// emitter resistors, not the later collector-compliance resistors, set the
// CA3280 IABC curves and introduce the common 2N4250 silicon knee.
// PCB3 has one grounded-base current source for each stored amount, while its
// collector bus fans out to the corresponding IABC pin on all five voice
// cards. The transistor therefore establishes the total bias current; it is
// not replicated independently on every card. Serviced CA3280 input voltages
// keep the five parallel branches close enough that the nominal circuit is an
// equal division, with the bounded per-card profiles below retaining device
// spread in the signal transfer itself.
const SHARED_VOICE_CONTROL_CURRENT_BRANCHES: f32 = 5.0;
const FILTER_ENVELOPE_AMOUNT_CONTROL_RESISTANCE_OHMS: f32 = 5_100.0;
const POLY_MOD_OSCILLATOR_B_CONTROL_RESISTANCE_OHMS: f32 = 5_600.0;
const POLY_MOD_FILTER_ENVELOPE_CONTROL_RESISTANCE_OHMS: f32 = 3_000.0;
// SD333 applies the same 0-10 V DAC domain to the three audio-level cells.
// Q306 and Q302 drive the two per-voice oscillator-mixer halves through
// populated 33 kohm emitter resistors. Q305 drives the common noise VCA
// through 75 kohm. Normalizing each current at panel maximum preserves the
// serviced full-level boundary while recovering the transistor knee and the
// distinct noise taper.
const OSCILLATOR_MIX_CONTROL_RESISTANCE_OHMS: f32 = 33_000.0;
const NOISE_MIX_CONTROL_RESISTANCE_OHMS: f32 = 75_000.0;
// The direct filter-envelope half of SD431 U422 is reconstructed as a current
// path into U433 rather than as a normalized VCA followed by an arbitrary
// octave range. R451 programs the linearizing diodes across the nominal
// +/-15 V supply, and the serviced balance network gives the inverting input
// its AC return through R450 and R453 plus the 100k trimmer's centre Thevenin
// value. Intersil gives RD = 52 / ID(mA) * 1.34 for the diode network. U433's
// 100k common-CV resistor defines 10 uA as one octave of filter control current.
const FILTER_ENVELOPE_DIODE_RESISTANCE_OHMS: f32 = 121_000.0;
// R452 (voice 1; R462/R492 on voices 2/5) is 47.5 kohm, printed compactly as
// "47.5K" like its R450 return and unlike the spaced "475 K" of R453. The
// direct half therefore mirrors the Poly Mod half's equal 22k/22k pair.
const FILTER_ENVELOPE_SOURCE_RESISTANCE_OHMS: f32 = 47_500.0;
const FILTER_ENVELOPE_BALANCE_SERIES_RESISTANCE_OHMS: f32 = 475_000.0;
const FILTER_ENVELOPE_INPUT_RETURN_RESISTANCE_OHMS: f32 = 47_500.0;
const FILTER_ENVELOPE_BALANCE_TRIM_THEVENIN_OHMS: f32 = 25_000.0;
const FILTER_ENVELOPE_DIODE_DYNAMIC_OHM_AMPS: f32 = 52.0e-3 * 1.34;
// The ID pin sits two junctions above the -15 V rail, so R451/R446 program
// the diode network across about 28.8 V rather than the full 30 V.
const FILTER_ENVELOPE_SUPPLY_SPAN_VOLTS: f32 = LINEARIZED_ID_PROGRAMMING_VOLTS;
const FILTER_SUM_COMMON_INPUT_RESISTANCE_OHMS: f32 = 100_000.0;

// The lower half of U422 applies the same filter envelope to the PMOD current
// sum through matched 22 kohm signal/return paths. R4146 biases its
// linearizing diodes, while the serviced 470k/100k balance network contributes
// only a small parallel AC return. U422 and the unlinearized oscillator-B
// U428 output share R4108's 30 kohm current-to-voltage load before U431 buffers
// the resulting physical PMOD voltage. The CA3280 data sheet guarantees at
// least +/-12 V output swing on +/-15 V rails.
const POLY_MOD_ENVELOPE_DIODE_RESISTANCE_OHMS: f32 = 120_000.0;
const POLY_MOD_ENVELOPE_SOURCE_RESISTANCE_OHMS: f32 = 22_000.0;
const POLY_MOD_ENVELOPE_INPUT_RETURN_RESISTANCE_OHMS: f32 = 22_000.0;
const POLY_MOD_ENVELOPE_BALANCE_SERIES_RESISTANCE_OHMS: f32 = 470_000.0;
const POLY_MOD_ENVELOPE_BALANCE_TRIM_THEVENIN_OHMS: f32 = 25_000.0;
const POLY_MOD_BUS_LOAD_RESISTANCE_OHMS: f32 = 30_000.0;
const POLY_MOD_BUS_MINIMUM_OUTPUT_SWING_VOLTS: f32 = 12.0;
// Q410 is grounded-base (TM1000D.2 Fig. 2-4 Detail B): only the emitter
// resistor R4496 sets its current. R4533 in the collector merely isolates the
// IABC pin from a current source.
const VCA_CONTROL_SERIES_RESISTANCE_OHMS: f32 = 3_300.0;
#[cfg(test)]
const Q410_REFERENCE_CURRENT_AMPS: f32 = 100.0e-6;
#[cfg(test)]
const Q410_REFERENCE_VBE_VOLTS: f32 = 0.56;
const Q410_THERMAL_VOLTAGE_VOLTS: f32 = 0.026;
const Q410_SATURATION_CURRENT_AMPS: f32 = 4.425_527e-14;
const Q410_NOMINAL_CONTROL_CURRENT_AMPS: f32 = 1_325.095_2e-6;
const FINAL_VCA_MAXIMUM_CONTROL_RATIO: f32 = 1.068_213_5;
// SD430 repeats the grounded-base 2N4250 conversion for the master CA3280.
// As with Q410, only the emitter resistor R4542 (4.7 kohm) sets the current;
// R4541 sits in the collector. PCB1's R113 spans the five-volt analog control
// domain, giving approximately 0.93 mA at full volume.
const MASTER_VOLUME_MAXIMUM_CV_VOLTS: f32 = 5.0;
const MASTER_VCA_CONTROL_SERIES_RESISTANCE_OHMS: f32 = 4_700.0;
const MASTER_VCA_NOMINAL_CONTROL_CURRENT_AMPS: f32 = 932.330_8e-6;
// U479 is linearized by R4561 (68 kohm from +15 V, about 424 uA) and fed
// through R4564 15 kohm with R4563 13 kohm on the other input. Its output
// current develops across R4562 || R4543 = 16.667 kohm. The same Figure 3A
// law as every other linearized CA3280 gives a full-volume voltage gain of
// about 1.0 and a linear range of +/-ID x 28 kohm, about +/-11.9 V.
const MASTER_VCA_DIODE_RESISTANCE_OHMS: f32 = 68_000.0;
const MASTER_VCA_DIODE_CURRENT_AMPS: f32 =
    LINEARIZED_ID_PROGRAMMING_VOLTS / MASTER_VCA_DIODE_RESISTANCE_OHMS;
const MASTER_VCA_INPUT_LOOP_OHMS: f32 = 15_000.0 + 13_000.0;
const MASTER_VCA_OUTPUT_LOAD_OHMS: f32 = 1.0 / (1.0 / 20_000.0 + 1.0 / 100_000.0);
const MASTER_VCA_NOMINAL_VOLTAGE_GAIN: f32 = CA3280_LINEARIZED_CURRENT_TRANSFER
    * (MASTER_VCA_NOMINAL_CONTROL_CURRENT_AMPS / MASTER_VCA_DIODE_CURRENT_AMPS)
    * MASTER_VCA_OUTPUT_LOAD_OHMS
    / MASTER_VCA_INPUT_LOOP_OHMS;
pub const MASTER_VCA_VOLTAGE_GAIN: f32 = MASTER_VCA_NOMINAL_VOLTAGE_GAIN;
const MASTER_VCA_NOMINAL_LIMIT_VOLTS: f32 =
    MASTER_VCA_DIODE_CURRENT_AMPS * MASTER_VCA_INPUT_LOOP_OHMS;
// TM1000D.2 section 2-5 gives approximately 100 kohm for a CA3280 input with
// its linearizing-diode terminal cut off. SD431 feeds saw/triangle through
// 150 kohm and pulse through 200 kohm. The selected source conductances share
// the populated 330 ohm input shunt before the resulting differential voltage
// is converted to physical OTA output current and first-cell filter voltage.
const UNLINEARIZED_INPUT_RESISTANCE_OHMS: f32 = 100_000.0;
const MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS: f32 = 150_000.0;
const MIXER_INPUT_SHUNT_RESISTANCE_OHMS: f32 = 330.0;

// U464's two current outputs feed CEM3320 IN A directly. The first cell's
// populated 100 kohm feedback sees the nominal 1 megohm CEM3320 output
// impedance in parallel, producing 90.909 kohm of current-to-voltage gain.
// The result is passed directly to the filter in circuit volts, so this
// physical boundary contains no host-amplitude calibration.
const FILTER_FIRST_CELL_FEEDBACK_OHMS: f32 = 100_000.0;
const FILTER_CELL_OUTPUT_IMPEDANCE_OHMS: f32 = 1_000_000.0;
const FILTER_FIRST_CELL_TRANSIMPEDANCE_OHMS: f32 =
    1.0 / (1.0 / FILTER_FIRST_CELL_FEEDBACK_OHMS + 1.0 / FILTER_CELL_OUTPUT_IMPEDANCE_OHMS);

// SD430 couples U427 through 200k into the 10k-shunted U430 input. U430's
// current output develops voltage across R4129, the U474 follower distributes
// it through 100k to every CEM3320 IN A, and the same first-cell
// transimpedance converts that injected current to filter circuit volts.
const WHITE_NOISE_SOURCE_VOLTS_PER_UNIT: f32 = 6.0;
const WHITE_NOISE_SOURCE_RESISTANCE_OHMS: f32 = 200_000.0;
const WHITE_NOISE_INPUT_SHUNT_RESISTANCE_OHMS: f32 = 10_000.0;
const WHITE_NOISE_OUTPUT_LOAD_RESISTANCE_OHMS: f32 = 10_000.0;
const FILTER_NOISE_INPUT_RESISTANCE_OHMS: f32 = 100_000.0;

// SD334's two halves of U378 sum their output currents into R3113. The source
// mix S/H holds the ordinary 1/12 V-per-code DAC voltage. Q307 converts the
// noise side through R3116, while Q308 receives the complementary voltage
// against the 10 V Thevenin source formed by R3128/R3130. Their 3.3 kohm collector resistors preserve compliance but
// do not set the grounded-base emitter current.
const WHEEL_MOD_CONTROL_RANGE_VOLTS: f32 = 10.0;
const WHEEL_MOD_NOISE_CONTROL_RESISTANCE_OHMS: f32 = 8_200.0;
const WHEEL_MOD_LFO_DIVIDER_HIGH_OHMS: f32 = 10_000.0;
const WHEEL_MOD_LFO_DIVIDER_LOW_OHMS: f32 = 20_000.0;
const WHEEL_MOD_LFO_CONTROL_RESISTANCE_OHMS: f32 =
    1.0 / (1.0 / WHEEL_MOD_LFO_DIVIDER_HIGH_OHMS + 1.0 / WHEEL_MOD_LFO_DIVIDER_LOW_OHMS);

// One LFO unit is five circuit volts in R3131's 160 kohm current coordinate.
// The LFO module has already preserved U380's bipolar triangle and converted
// the unipolar saw/pulse 160k/200k paths into that shared coordinate.
// U374's pink-noise value already contains its 100k/47k closed-loop gain; the
// MM5837 data sheet guarantees each output level within 1.5 V of a 15 V supply
// rail, establishing at least 12 Vpp, or 6 V peak about the centred signal.
const WHEEL_MOD_LFO_SOURCE_VOLTS_PER_UNIT: f32 = 5.0;
const WHEEL_MOD_NOISE_SOURCE_VOLTS_PER_UNIT: f32 = 6.0;
const WHEEL_MOD_LFO_INPUT_RESISTANCE_OHMS: f32 = 160_000.0;
const WHEEL_MOD_NOISE_INPUT_RESISTANCE_OHMS: f32 = 20_000.0;
const WHEEL_MOD_INPUT_SHUNT_OHMS: f32 = 330.0;
// U378's output current develops across R3113 (10k) in parallel with the
// 100k MOD wheel pot R2, whose wiper feeds the U374 follower.
const WHEEL_MOD_OUTPUT_LOAD_OHMS: f32 = 1.0 / (1.0 / 10_000.0 + 1.0 / 100_000.0);

// CA3280 data-sheet typicals: 16 mS at 1 mA IABC and 410 uA peak output at
// 500 uA IABC. Their ratio supplies both the small-signal slope and the
// unlinearized current limit without a host-level saturation constant.
const CA3280_TRANSCONDUCTANCE_SIEMENS_PER_AMP: f32 = 16.0;
const CA3280_PEAK_OUTPUT_CURRENT_RATIO: f32 = 410.0 / 500.0;
// Figure 3A: 1066 uA of output swing across 8.36 V of input into 20.35 kohm
// (two 10k inputs plus the ~350 ohm diode network at 200 uA), scaled by
// ID/IABC = 200/650.
const CA3280_LINEARIZED_CURRENT_TRANSFER: f32 = 0.776;
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
// the positive filter-envelope contribution to Poly Mod. The service trims
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
    oscillator_mixer_to_filter_volts(input, 1.0, 0.0, 0.0, control, half)
}

/// One oscillator mixer half including the independent finite loading at both
/// 330-ohm-shunted CA3280 inputs.
///
/// Conductances are relative to one 150 kohm path. SD431 routes saw to the
/// positive input, while pulse and oscillator-B triangle reach the negative
/// input. Each source-voltage argument is already conductance-weighted.
pub fn oscillator_mixer_loaded(
    positive_source_volts: f32,
    positive_source_conductance: f32,
    negative_source_volts: f32,
    negative_source_conductance: f32,
    control: f32,
    voice_index: usize,
    channel: MixerChannel,
) -> f32 {
    oscillator_mixer_loaded_with_control_current(
        positive_source_volts,
        positive_source_conductance,
        negative_source_volts,
        negative_source_conductance,
        oscillator_mixer_control_current_amps(control),
        voice_index,
        channel,
    )
}

/// Convert one static oscillator-level CV into the CA3280 bias current that
/// all four internal sub-samples share.
pub fn oscillator_mixer_control_current_amps(control: f32) -> f32 {
    shared_voice_control_current_amps(control, OSCILLATOR_MIX_CONTROL_RESISTANCE_OHMS)
}

/// Loaded oscillator mixer with an already reconstructed CA3280 bias current.
pub fn oscillator_mixer_loaded_with_control_current(
    positive_source_volts: f32,
    positive_source_conductance: f32,
    negative_source_volts: f32,
    negative_source_conductance: f32,
    control_current_amps: f32,
    voice_index: usize,
    channel: MixerChannel,
) -> f32 {
    if !positive_source_volts.is_finite()
        || !positive_source_conductance.is_finite()
        || !negative_source_volts.is_finite()
        || !negative_source_conductance.is_finite()
        || (positive_source_conductance <= 0.0 && negative_source_conductance <= 0.0)
        || !control_current_amps.is_finite()
        || control_current_amps <= 0.0
    {
        return 0.0;
    }
    let profile = MIXER_PROFILES[voice_index % MIXER_PROFILES.len()];
    let half = match channel {
        MixerChannel::OscillatorA => profile.oscillator_a,
        MixerChannel::OscillatorB => profile.oscillator_b,
    };
    oscillator_mixer_to_filter_volts_with_control_current(
        positive_source_volts,
        positive_source_conductance,
        negative_source_volts,
        negative_source_conductance,
        control_current_amps,
        half,
    )
}

/// The single common noise-level OTA before noise reaches all five filters.
pub fn common_noise(input: f32, control: f32) -> f32 {
    common_noise_with_control_current(input, common_noise_control_current_amps(control))
}

pub fn common_noise_control_current_amps(control: f32) -> f32 {
    ten_volt_control_current_amps(control, NOISE_MIX_CONTROL_RESISTANCE_OHMS)
}

pub fn common_noise_with_control_current(input: f32, control_current_amps: f32) -> f32 {
    if !input.is_finite() || !control_current_amps.is_finite() || control_current_amps <= 0.0 {
        return 0.0;
    }
    let source_current_amps =
        input / WHITE_NOISE_SOURCE_RESISTANCE_OHMS * WHITE_NOISE_SOURCE_VOLTS_PER_UNIT;
    let input_conductance_siemens = 1.0 / WHITE_NOISE_SOURCE_RESISTANCE_OHMS
        + 1.0 / WHITE_NOISE_INPUT_SHUNT_RESISTANCE_OHMS
        + 1.0 / UNLINEARIZED_INPUT_RESISTANCE_OHMS;
    let differential_input_volts =
        source_current_amps / input_conductance_siemens * COMMON_NOISE_PROFILE.input_drive_ratio;
    let output_current_amps = ota_output_current_amps(
        differential_input_volts,
        control_current_amps,
        COMMON_NOISE_PROFILE,
    );
    let buffered_noise_volts = output_current_amps * WHITE_NOISE_OUTPUT_LOAD_RESISTANCE_OHMS;
    buffered_noise_volts / FILTER_NOISE_INPUT_RESISTANCE_OHMS
        * FILTER_FIRST_CELL_TRANSIMPEDANCE_OHMS
}

/// The common dual-OTA current mixer feeding the physical modulation wheel.
///
/// The returned value is the reconstructed voltage across SD334 R3113, not a
/// normalized host bus. Q307/Q308 create the complementary IABC currents and
/// the two populated input dividers set each unlinearized CA3280 drive.
pub fn wheel_mod_source(lfo: f32, noise: f32, source_mix: f32) -> f32 {
    let [lfo_current_amps, noise_current_amps] = wheel_mod_control_currents_amps(source_mix);
    wheel_mod_source_with_control_currents(lfo, noise, lfo_current_amps, noise_current_amps)
}

pub fn wheel_mod_control_currents_amps(source_mix: f32) -> [f32; 2] {
    if !source_mix.is_finite() {
        return [0.0; 2];
    }
    let source_mix_cv = rf_5_contract::hardware::normalized_control_volts(source_mix)
        .min(WHEEL_MOD_CONTROL_RANGE_VOLTS);
    [
        grounded_base_2n4250_collector_current_amps(
            WHEEL_MOD_CONTROL_RANGE_VOLTS - source_mix_cv,
            WHEEL_MOD_LFO_CONTROL_RESISTANCE_OHMS,
        ),
        grounded_base_2n4250_collector_current_amps(
            source_mix_cv,
            WHEEL_MOD_NOISE_CONTROL_RESISTANCE_OHMS,
        ),
    ]
}

pub fn wheel_mod_source_with_control_currents(
    lfo: f32,
    noise: f32,
    lfo_current_amps: f32,
    noise_current_amps: f32,
) -> f32 {
    if !lfo.is_finite()
        || !noise.is_finite()
        || !lfo_current_amps.is_finite()
        || !noise_current_amps.is_finite()
    {
        return 0.0;
    }

    unlinearized_ota_loaded_voltage(
        lfo * WHEEL_MOD_LFO_SOURCE_VOLTS_PER_UNIT,
        WHEEL_MOD_LFO_INPUT_RESISTANCE_OHMS,
        lfo_current_amps,
        WHEEL_MOD_LFO_PROFILE,
    ) + unlinearized_ota_loaded_voltage(
        noise * WHEEL_MOD_NOISE_SOURCE_VOLTS_PER_UNIT,
        WHEEL_MOD_NOISE_INPUT_RESISTANCE_OHMS,
        noise_current_amps,
        WHEEL_MOD_NOISE_PROFILE,
    )
}

/// Filter-cutoff displacement produced by the direct envelope half of U422.
///
/// The result is in octaves because it is expressed relative to the 10 uA
/// that a one-volt common filter CV sends through U433's populated 100 kohm
/// input. The amount current follows SD333 Q301 and its populated 5.1 kohm
/// emitter resistor.
pub fn filter_envelope_cutoff_octaves(envelope: f32, amount: f32, voice_index: usize) -> f32 {
    filter_envelope_cutoff_octaves_with_control_current(
        envelope,
        filter_envelope_control_current_amps(amount),
        voice_index,
    )
}

pub fn filter_envelope_control_current_amps(amount: f32) -> f32 {
    shared_voice_control_current_amps(amount, FILTER_ENVELOPE_AMOUNT_CONTROL_RESISTANCE_OHMS)
}

pub fn filter_envelope_cutoff_octaves_with_control_current(
    envelope: f32,
    control_current_amps: f32,
    voice_index: usize,
) -> f32 {
    if !envelope.is_finite()
        || envelope <= 0.0
        || !control_current_amps.is_finite()
        || control_current_amps <= 0.0
    {
        return 0.0;
    }

    let profile =
        ENVELOPE_AMOUNT_PROFILES[voice_index % ENVELOPE_AMOUNT_PROFILES.len()].direct_filter;
    let balance_return_resistance_ohms = 1.0
        / (1.0 / FILTER_ENVELOPE_INPUT_RETURN_RESISTANCE_OHMS
            + 1.0
                / (FILTER_ENVELOPE_BALANCE_SERIES_RESISTANCE_OHMS
                    + FILTER_ENVELOPE_BALANCE_TRIM_THEVENIN_OHMS));
    let input_loop_resistance_ohms =
        FILTER_ENVELOPE_SOURCE_RESISTANCE_OHMS + balance_return_resistance_ohms;
    let envelope_volts = (envelope * ENVELOPE_NOMINAL_PEAK_VOLTS).min(ENVELOPE_MAXIMUM_PEAK_VOLTS);
    let output_current_amps = linearized_ota_output_current_amps(
        envelope_volts,
        input_loop_resistance_ohms,
        FILTER_ENVELOPE_DIODE_RESISTANCE_OHMS,
        control_current_amps,
        profile,
    );
    output_current_amps * FILTER_SUM_COMMON_INPUT_RESISTANCE_OHMS
}

/// Output current from the second U422 half carrying filter envelope to PMOD.
pub fn poly_mod_filter_envelope_current_amps(
    envelope: f32,
    control: f32,
    voice_index: usize,
) -> f32 {
    poly_mod_filter_envelope_with_control_current_amps(
        envelope,
        poly_mod_filter_envelope_control_current_amps(control),
        voice_index,
    )
}

pub fn poly_mod_filter_envelope_control_current_amps(control: f32) -> f32 {
    shared_voice_control_current_amps(control, POLY_MOD_FILTER_ENVELOPE_CONTROL_RESISTANCE_OHMS)
}

pub fn poly_mod_filter_envelope_with_control_current_amps(
    envelope: f32,
    control_current_amps: f32,
    voice_index: usize,
) -> f32 {
    if !envelope.is_finite() || !control_current_amps.is_finite() || control_current_amps <= 0.0 {
        return 0.0;
    }
    let profile = ENVELOPE_AMOUNT_PROFILES[voice_index % ENVELOPE_AMOUNT_PROFILES.len()].poly_mod;
    let balance_return_resistance_ohms = 1.0
        / (1.0 / POLY_MOD_ENVELOPE_INPUT_RETURN_RESISTANCE_OHMS
            + 1.0
                / (POLY_MOD_ENVELOPE_BALANCE_SERIES_RESISTANCE_OHMS
                    + POLY_MOD_ENVELOPE_BALANCE_TRIM_THEVENIN_OHMS));
    let input_loop_resistance_ohms =
        POLY_MOD_ENVELOPE_SOURCE_RESISTANCE_OHMS + balance_return_resistance_ohms;
    let envelope_volts = (envelope * ENVELOPE_NOMINAL_PEAK_VOLTS)
        .clamp(-ENVELOPE_MAXIMUM_PEAK_VOLTS, ENVELOPE_MAXIMUM_PEAK_VOLTS);
    linearized_ota_output_current_amps(
        envelope_volts,
        input_loop_resistance_ohms,
        POLY_MOD_ENVELOPE_DIODE_RESISTANCE_OHMS,
        control_current_amps,
        profile,
    )
}

/// Output current from the unlinearized oscillator-B PMOD amount OTA U428.
pub fn poly_mod_oscillator_b_current_amps(
    input: f32,
    source_conductance: f32,
    control: f32,
    voice_index: usize,
) -> f32 {
    poly_mod_oscillator_b_with_control_current_amps(
        input,
        source_conductance,
        poly_mod_oscillator_b_control_current_amps(control),
        voice_index,
    )
}

pub fn poly_mod_oscillator_b_control_current_amps(control: f32) -> f32 {
    shared_voice_control_current_amps(control, POLY_MOD_OSCILLATOR_B_CONTROL_RESISTANCE_OHMS)
}

pub fn poly_mod_oscillator_b_with_control_current_amps(
    input: f32,
    source_conductance: f32,
    control_current_amps: f32,
    voice_index: usize,
) -> f32 {
    unlinearized_conductance_mixer_output_current_with_control_amps(
        input,
        source_conductance,
        0.0,
        0.0,
        control_current_amps,
        POLY_MOD_OSCILLATOR_B_PROFILES[voice_index % POLY_MOD_OSCILLATOR_B_PROFILES.len()],
    )
}

/// Shared U422/U428 current sum developed across R4108 and buffered by U431.
pub fn poly_mod_bus_voltage(
    filter_envelope_current_amps: f32,
    oscillator_b_current_amps: f32,
) -> f32 {
    if !filter_envelope_current_amps.is_finite() || !oscillator_b_current_amps.is_finite() {
        return 0.0;
    }
    let unloaded_bus_volts = (filter_envelope_current_amps + oscillator_b_current_amps)
        * POLY_MOD_BUS_LOAD_RESISTANCE_OHMS;
    sixth_order_limited(unloaded_bus_volts, POLY_MOD_BUS_MINIMUM_OUTPUT_SWING_VOLTS)
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
    if input == 0.0 {
        return input;
    }
    let limit = MASTER_VCA_NOMINAL_LIMIT_VOLTS / MASTER_VCA_PROFILE.input_drive_ratio;
    sixth_order_limited(input, limit)
        * control_current_ratio.clamp(0.0, 1.0)
        * MASTER_VCA_NOMINAL_VOLTAGE_GAIN
        * MASTER_VCA_PROFILE.transconductance_ratio
}

fn oscillator_mixer_to_filter_volts(
    positive_source_volts: f32,
    positive_source_conductance: f32,
    negative_source_volts: f32,
    negative_source_conductance: f32,
    control: f32,
    profile: OtaHalfProfile,
) -> f32 {
    oscillator_mixer_to_filter_volts_with_control_current(
        positive_source_volts,
        positive_source_conductance,
        negative_source_volts,
        negative_source_conductance,
        oscillator_mixer_control_current_amps(control),
        profile,
    )
}

fn oscillator_mixer_to_filter_volts_with_control_current(
    positive_source_volts: f32,
    positive_source_conductance: f32,
    negative_source_volts: f32,
    negative_source_conductance: f32,
    control_current_amps: f32,
    profile: OtaHalfProfile,
) -> f32 {
    unlinearized_conductance_mixer_output_current_with_control_amps(
        positive_source_volts,
        positive_source_conductance,
        negative_source_volts,
        negative_source_conductance,
        control_current_amps,
        profile,
    ) * FILTER_FIRST_CELL_TRANSIMPEDANCE_OHMS
}

fn unlinearized_conductance_mixer_output_current_with_control_amps(
    positive_source_volts: f32,
    positive_source_conductance: f32,
    negative_source_volts: f32,
    negative_source_conductance: f32,
    control_current_amps: f32,
    profile: OtaHalfProfile,
) -> f32 {
    if !positive_source_volts.is_finite()
        || !positive_source_conductance.is_finite()
        || !negative_source_volts.is_finite()
        || !negative_source_conductance.is_finite()
        || (positive_source_conductance <= 0.0 && negative_source_conductance <= 0.0)
        || !control_current_amps.is_finite()
        || control_current_amps <= 0.0
    {
        return 0.0;
    }

    let positive_input_volts =
        unlinearized_input_voltage(positive_source_volts, positive_source_conductance);
    let negative_input_volts =
        unlinearized_input_voltage(negative_source_volts, negative_source_conductance);
    let differential_input_volts =
        (positive_input_volts - negative_input_volts) * profile.input_drive_ratio;
    ota_output_current_amps(differential_input_volts, control_current_amps, profile)
}

fn unlinearized_input_voltage(source_volts: f32, source_conductance: f32) -> f32 {
    if source_conductance <= 0.0 {
        return 0.0;
    }
    let source_current_amps = source_volts / MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS;
    let input_conductance_siemens = source_conductance / MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS
        + 1.0 / MIXER_INPUT_SHUNT_RESISTANCE_OHMS
        + 1.0 / UNLINEARIZED_INPUT_RESISTANCE_OHMS;
    source_current_amps / input_conductance_siemens
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
    ota_output_current_amps(differential_input_volts, iabc_amps, profile)
        * WHEEL_MOD_OUTPUT_LOAD_OHMS
}

fn linearized_ota_output_current_amps(
    source_volts: f32,
    source_loop_resistance_ohms: f32,
    diode_resistance_ohms: f32,
    iabc_amps: f32,
    profile: OtaHalfProfile,
) -> f32 {
    if !source_volts.is_finite()
        || !source_loop_resistance_ohms.is_finite()
        || source_loop_resistance_ohms <= 0.0
        || !diode_resistance_ohms.is_finite()
        || diode_resistance_ohms <= 0.0
    {
        return 0.0;
    }
    // With ID active the signal enters as a current through the source loop
    // and the linearizing diodes' small dynamic impedance. Intersil's Figure
    // 3A (ID = 200 uA, IABC = 650 uA, 10k + 10k inputs) is linear at about
    // 125 uA/V until the input current reaches +/-ID, where the output
    // settles at the published 0.82 IABC peak: Iout = 0.776 Is IABC / ID.
    let diode_current_amps = FILTER_ENVELOPE_SUPPLY_SPAN_VOLTS / diode_resistance_ohms;
    let diode_dynamic_resistance_ohms = FILTER_ENVELOPE_DIODE_DYNAMIC_OHM_AMPS / diode_current_amps;
    let input_current_amps =
        source_volts / (source_loop_resistance_ohms + diode_dynamic_resistance_ohms);
    let linear_output_current_amps =
        CA3280_LINEARIZED_CURRENT_TRANSFER * input_current_amps * iabc_amps / diode_current_amps
            * profile.input_drive_ratio;
    if !iabc_amps.is_finite() || iabc_amps <= 0.0 {
        return 0.0;
    }
    let peak_output_current_amps = iabc_amps * CA3280_PEAK_OUTPUT_CURRENT_RATIO;
    let normalized = linear_output_current_amps / peak_output_current_amps;
    #[cfg(feature = "fast-math")]
    let shaped = crate::realtime_math::tanh(normalized);
    #[cfg(not(feature = "fast-math"))]
    let shaped = libm::tanhf(normalized);
    peak_output_current_amps * shaped * profile.transconductance_ratio
}

fn ota_output_current_amps(
    differential_input_volts: f32,
    iabc_amps: f32,
    profile: OtaHalfProfile,
) -> f32 {
    if !differential_input_volts.is_finite() || !iabc_amps.is_finite() || iabc_amps <= 0.0 {
        return 0.0;
    }
    let peak_output_current_amps = iabc_amps * CA3280_PEAK_OUTPUT_CURRENT_RATIO;
    let small_signal_output_current_amps =
        iabc_amps * CA3280_TRANSCONDUCTANCE_SIEMENS_PER_AMP * differential_input_volts;
    let normalized = small_signal_output_current_amps / peak_output_current_amps;
    #[cfg(feature = "fast-math")]
    let shaped = crate::realtime_math::tanh(normalized);
    #[cfg(not(feature = "fast-math"))]
    let shaped = libm::tanhf(normalized);
    peak_output_current_amps * shaped * profile.transconductance_ratio
}

fn final_voice_transfer(input: f32, control: f32, profile: OtaHalfProfile) -> f32 {
    if control <= 0.0 || !control.is_finite() || !input.is_finite() {
        return 0.0;
    }

    // A larger drive ratio means an earlier knee, matching the population
    // convention used by the other OTA profiles. Evaluate the sixth-order
    // norm on either side of unity to avoid overflow for arbitrary finite host
    // input while preserving odd symmetry and a finite current asymptote.
    let limit = FINAL_VCA_NOMINAL_LIMIT_VOLTS / profile.input_drive_ratio;
    sixth_order_limited(input, limit)
        * control.clamp(0.0, FINAL_VCA_MAXIMUM_CONTROL_RATIO)
        * FINAL_VCA_THEVENIN_GAIN
        * profile.transconductance_ratio
}

fn sixth_order_limited(input: f32, limit: f32) -> f32 {
    let magnitude = input.abs();
    let ratio = magnitude / limit;
    let limited_magnitude = if ratio <= 1.0 {
        let ratio_squared = ratio * ratio;
        let ratio_sixth = ratio_squared * ratio_squared * ratio_squared;
        let soft_base = 1.0 + ratio_sixth;
        if soft_base == 1.0 {
            magnitude
        } else {
            #[cfg(feature = "fast-math")]
            {
                magnitude * crate::realtime_math::inverse_sixth_root_one_plus(ratio_sixth)
            }
            #[cfg(not(feature = "fast-math"))]
            {
                magnitude / libm::powf(soft_base, 1.0 / FINAL_VCA_SOFT_KNEE_ORDER)
            }
        }
    } else {
        let inverse = 1.0 / ratio;
        let inverse_squared = inverse * inverse;
        let inverse_sixth = inverse_squared * inverse_squared * inverse_squared;
        #[cfg(not(feature = "fast-math"))]
        let soft_base = 1.0 + inverse_sixth;
        #[cfg(feature = "fast-math")]
        {
            limit * crate::realtime_math::inverse_sixth_root_one_plus(inverse_sixth)
        }
        #[cfg(not(feature = "fast-math"))]
        {
            limit / libm::powf(soft_base, 1.0 / FINAL_VCA_SOFT_KNEE_ORDER)
        }
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
    let log_argument =
        series_resistance_ohms * Q410_SATURATION_CURRENT_AMPS / Q410_THERMAL_VOLTAGE_VOLTS;
    #[cfg(feature = "fast-math")]
    let log_argument = crate::realtime_math::ln(log_argument);
    #[cfg(not(feature = "fast-math"))]
    let log_argument = libm::logf(log_argument);
    let z = drive_volts / Q410_THERMAL_VOLTAGE_VOLTS + log_argument;
    let mut normalized_current = if z >= 1.0 {
        #[cfg(feature = "fast-math")]
        {
            (z - crate::realtime_math::ln(z)).max(f32::MIN_POSITIVE)
        }
        #[cfg(not(feature = "fast-math"))]
        {
            (z - libm::logf(z)).max(f32::MIN_POSITIVE)
        }
    } else {
        #[cfg(feature = "fast-math")]
        {
            crate::realtime_math::exp(z).max(f32::MIN_POSITIVE)
        }
        #[cfg(not(feature = "fast-math"))]
        {
            libm::expf(z).max(f32::MIN_POSITIVE)
        }
    };
    for _ in 0..3 {
        #[cfg(feature = "fast-math")]
        let logarithm = crate::realtime_math::ln(normalized_current);
        #[cfg(not(feature = "fast-math"))]
        let logarithm = libm::logf(normalized_current);
        let residual = normalized_current + logarithm - z;
        let slope = 1.0 + 1.0 / normalized_current;
        normalized_current = (normalized_current - residual / slope).max(f32::MIN_POSITIVE);
    }
    normalized_current * Q410_THERMAL_VOLTAGE_VOLTS / series_resistance_ohms
}

fn ten_volt_control_current_amps(control: f32, emitter_resistance_ohms: f32) -> f32 {
    if !control.is_finite() || control <= 0.0 {
        return 0.0;
    }
    grounded_base_2n4250_collector_current_amps(
        rf_5_contract::hardware::normalized_control_volts(control),
        emitter_resistance_ohms,
    )
}

fn shared_voice_control_current_amps(control: f32, emitter_resistance_ohms: f32) -> f32 {
    ten_volt_control_current_amps(control, emitter_resistance_ohms)
        / SHARED_VOICE_CONTROL_CURRENT_BRANCHES
}

#[cfg(test)]
fn ten_volt_control_current_ratio(control: f32, emitter_resistance_ohms: f32) -> f32 {
    ten_volt_control_current_amps(control, emitter_resistance_ohms)
        / ten_volt_control_current_amps(1.0, emitter_resistance_ohms)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// V8.1's trimmed pot ADC reads a fully clockwise knob as code 120 (10 V).
    const PANEL_CEILING: f32 = 120.0 / 127.0;

    /// Three Newton steps are enough, stated where it can be checked.
    ///
    /// `x + ln(x) = z` is solved with a fixed three iterations and no
    /// convergence test. A comment claimed three were "sufficient over the
    /// admitted 0-5.3 V CEM3310 range"; this walks that range in two
    /// hundred thousand steps for both populated series resistances and
    /// puts a number on it. The residual is the equation's own, so this
    /// checks the solve rather than comparing it against a second
    /// implementation that would need checking in turn.
    ///
    /// It also keeps going to 20 V. Nothing drives the control input that
    /// hard -- both callers clamp well below it -- but a fixed iteration
    /// count is exactly the kind of thing that stops being enough when
    /// someone widens a range, and a solver that quietly stopped
    /// converging would otherwise show up as a timbre nobody could place.
    #[test]
    fn three_newton_steps_solve_the_transistor_across_the_admitted_range() {
        for resistance in [
            VCA_CONTROL_SERIES_RESISTANCE_OHMS,
            MASTER_VCA_CONTROL_SERIES_RESISTANCE_OHMS,
        ] {
            let offset =
                libm::logf(resistance * Q410_SATURATION_CURRENT_AMPS / Q410_THERMAL_VOLTAGE_VOLTS);
            let mut worst = 0.0_f32;
            let mut worst_at = 0.0_f32;
            for step in 1..=200_000_u32 {
                let drive = step as f32 / 200_000.0 * 20.0;
                let current = grounded_base_2n4250_collector_current_amps(drive, resistance);
                let normalized = current * resistance / Q410_THERMAL_VOLTAGE_VOLTS;
                let residual = (normalized + libm::logf(normalized)
                    - (drive / Q410_THERMAL_VOLTAGE_VOLTS + offset))
                    .abs();
                if residual > worst {
                    worst = residual;
                    worst_at = drive;
                }
            }
            assert!(worst <= 5.0e-4, "residuo {worst} en {worst_at} V");
        }
    }

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
                assert_eq!(
                    poly_mod_filter_envelope_current_amps(input, 0.0, voice),
                    0.0
                );
                assert_eq!(
                    poly_mod_oscillator_b_current_amps(input, 1.0, 0.0, voice),
                    0.0
                );
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
        assert!((232.0e-6..=244.0e-6).contains(&diode_current));
        assert!((280.0..=300.0).contains(&diode_resistance));

        for voice in 0..5 {
            let quarter = filter_envelope_cutoff_octaves(1.0, 0.25, voice);
            let half = filter_envelope_cutoff_octaves(1.0, 0.5, voice);
            let full = filter_envelope_cutoff_octaves(1.0, 120.0 / 127.0, voice);
            assert!(quarter > 0.0 && quarter < half && half < full);
            // R452 47.5k into Figure 3A's linearized transfer, no fitted gain.
            assert!((6.2..=6.7).contains(&full), "voice {voice} full {full}");
            assert_eq!(filter_envelope_cutoff_octaves(0.0, 1.0, voice), 0.0);
        }

        let factory_e_piano_amount = 32.0 / 127.0;
        for voice in 0..5 {
            let onset = filter_envelope_cutoff_octaves(1.0, factory_e_piano_amount, voice);
            assert!(
                (1.35..=1.55).contains(&onset),
                "voice {voice} 1-4 filter-envelope onset {onset} octaves"
            );
        }
    }

    #[test]
    fn q301_q303_and_q304_set_the_three_patch_amount_currents() {
        let direct = ten_volt_control_current_amps(
            PANEL_CEILING,
            FILTER_ENVELOPE_AMOUNT_CONTROL_RESISTANCE_OHMS,
        );
        let oscillator_b = ten_volt_control_current_amps(
            PANEL_CEILING,
            POLY_MOD_OSCILLATOR_B_CONTROL_RESISTANCE_OHMS,
        );
        let poly_envelope = ten_volt_control_current_amps(
            PANEL_CEILING,
            POLY_MOD_FILTER_ENVELOPE_CONTROL_RESISTANCE_OHMS,
        );
        assert!((1.7e-3..=1.9e-3).contains(&direct));
        assert!((1.55e-3..=1.75e-3).contains(&oscillator_b));
        assert!((3.0e-3..=3.2e-3).contains(&poly_envelope));
        assert!(poly_envelope > direct && direct > oscillator_b);

        assert!(
            (shared_voice_control_current_amps(
                PANEL_CEILING,
                FILTER_ENVELOPE_AMOUNT_CONTROL_RESISTANCE_OHMS,
            ) - direct / SHARED_VOICE_CONTROL_CURRENT_BRANCHES)
                .abs()
                < 1.0e-9
        );
        assert!(
            (shared_voice_control_current_amps(
                PANEL_CEILING,
                POLY_MOD_OSCILLATOR_B_CONTROL_RESISTANCE_OHMS,
            ) - oscillator_b / SHARED_VOICE_CONTROL_CURRENT_BRANCHES)
                .abs()
                < 1.0e-9
        );
        assert!(
            (shared_voice_control_current_amps(
                PANEL_CEILING,
                POLY_MOD_FILTER_ENVELOPE_CONTROL_RESISTANCE_OHMS,
            ) - poly_envelope / SHARED_VOICE_CONTROL_CURRENT_BRANCHES)
                .abs()
                < 1.0e-9
        );

        for resistance in [
            FILTER_ENVELOPE_AMOUNT_CONTROL_RESISTANCE_OHMS,
            POLY_MOD_OSCILLATOR_B_CONTROL_RESISTANCE_OHMS,
            POLY_MOD_FILTER_ENVELOPE_CONTROL_RESISTANCE_OHMS,
        ] {
            let quarter = ten_volt_control_current_ratio(0.25, resistance);
            let half = ten_volt_control_current_ratio(0.5, resistance);
            let full = ten_volt_control_current_ratio(1.0, resistance);
            assert!(quarter > 0.0 && quarter < half && half < full);
            assert!(half < 0.5);
            assert!((full - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn q302_q305_and_q306_set_the_three_audio_level_currents() {
        let oscillator =
            ten_volt_control_current_amps(PANEL_CEILING, OSCILLATOR_MIX_CONTROL_RESISTANCE_OHMS);
        let noise = ten_volt_control_current_amps(PANEL_CEILING, NOISE_MIX_CONTROL_RESISTANCE_OHMS);
        assert!((275.0e-6..=290.0e-6).contains(&oscillator));
        assert!((120.0e-6..=130.0e-6).contains(&noise));
        assert!(oscillator > noise);

        for resistance in [
            OSCILLATOR_MIX_CONTROL_RESISTANCE_OHMS,
            NOISE_MIX_CONTROL_RESISTANCE_OHMS,
        ] {
            let quarter = ten_volt_control_current_ratio(0.25, resistance);
            let half = ten_volt_control_current_ratio(0.5, resistance);
            let full = ten_volt_control_current_ratio(1.0, resistance);
            assert!(quarter > 0.0 && quarter < half && half < full);
            assert!(half < 0.5);
            assert!((full - 1.0).abs() < f32::EPSILON);
        }

        let input = 0.25;
        assert!(
            oscillator_mixer(input, 0.5, 0, MixerChannel::OscillatorA)
                < oscillator_mixer(input, 1.0, 0, MixerChannel::OscillatorA)
        );
        assert!(common_noise(input, 0.5) < common_noise(input, 1.0));
    }

    #[test]
    fn mixer_currents_reach_the_filter_through_populated_transimpedances() {
        assert!((90_900.0..=90_920.0).contains(&FILTER_FIRST_CELL_TRANSIMPEDANCE_OHMS));

        let source_current_amps = 5.0 / MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS;
        let input_conductance_siemens = 1.0 / MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS
            + 1.0 / MIXER_INPUT_SHUNT_RESISTANCE_OHMS
            + 1.0 / UNLINEARIZED_INPUT_RESISTANCE_OHMS;
        let differential_input_volts = source_current_amps / input_conductance_siemens;
        assert!((10.8e-3..=11.1e-3).contains(&differential_input_volts));

        for voice in 0..5 {
            for channel in [MixerChannel::OscillatorA, MixerChannel::OscillatorB] {
                let one_saw = oscillator_mixer(5.0, 1.0, voice, channel);
                assert!((0.80..=0.98).contains(&one_saw));
            }
        }

        let full_noise = common_noise(1.0, 1.0);
        assert!((0.84..=1.0).contains(&full_noise));
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
        assert!((1.30e-3..=1.35e-3).contains(&nominal));
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
        assert!((amplifier_envelope_control(2.0) - maximum).abs() < 1.0e-6);
    }

    #[test]
    fn q411_reconstructs_the_master_ca3280_control_current() {
        let maximum = grounded_base_2n4250_collector_current_amps(
            MASTER_VOLUME_MAXIMUM_CV_VOLTS,
            MASTER_VCA_CONTROL_SERIES_RESISTANCE_OHMS,
        );
        assert!((0.92e-3..=0.94e-3).contains(&maximum));
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
    fn linearizing_diodes_are_programmed_against_the_negative_rail() {
        // The ID terminal drives an internal mirror referred to the -15 V pin.
        for current in [MASTER_VCA_DIODE_CURRENT_AMPS, FINAL_VCA_DIODE_CURRENT_AMPS] {
            assert!((420.0e-6..=428.0e-6).contains(&current), "{current}");
        }
    }

    #[test]
    fn populated_master_vca_has_the_graph_derived_voltage_gain() {
        assert!((1.0..=1.04).contains(&MASTER_VCA_NOMINAL_VOLTAGE_GAIN));
        let measured = master_output(1.0e-4, 1.0) / 1.0e-4;
        assert!((measured / MASTER_VCA_NOMINAL_VOLTAGE_GAIN - 1.0).abs() < 1.0e-6);
        assert!((11.8..=12.0).contains(&MASTER_VCA_NOMINAL_LIMIT_VOLTS));
    }

    #[test]
    fn master_vca_transfer_is_odd_monotonic_and_bounded() {
        let limit = MASTER_VCA_NOMINAL_LIMIT_VOLTS / MASTER_VCA_PROFILE.input_drive_ratio;
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
    fn unlinearized_mixer_limits_before_the_linearized_final_vca() {
        let mixer_input = 50.0;
        let final_input = 2.5;
        let unlinearized = oscillator_mixer(mixer_input, 1.0, 2, MixerChannel::OscillatorA).abs();
        let linearized = final_voice(final_input, 1.0, 2).abs();
        let mixer_small_signal =
            oscillator_mixer(0.001, 1.0, 2, MixerChannel::OscillatorA).abs() / 0.001;
        let final_small_signal = final_voice(0.001, 1.0, 2).abs() / 0.001;
        let mixer_retained = unlinearized / (mixer_input * mixer_small_signal);
        let final_retained = linearized / (final_input * final_small_signal);
        assert!(final_retained > 0.97);
        assert!(mixer_retained < 0.55);
    }

    #[test]
    fn figure_3a_linear_range_scales_with_id_and_the_input_loop() {
        // Figure 3A: +/-ID x (10k + 10k) = +/-4 V at 200 uA.
        assert!((200.0e-6 * 20_000.0 - 4.0_f32).abs() < 1.0e-6);
        assert!((16.8..=17.1).contains(&FINAL_VCA_NOMINAL_LIMIT_VOLTS));
        assert!((1.50..=1.54).contains(&FINAL_VCA_THEVENIN_GAIN));
    }

    #[test]
    fn final_vca_knee_overlaps_the_cem3320_output_range() {
        fn retained_at_vpp(circuit_vpp: f32) -> f32 {
            let peak_volts = circuit_vpp * 0.5;
            final_voice(peak_volts, 1.0, 2) / (peak_volts * FINAL_VCA_THEVENIN_GAIN)
        }

        // The CEM3320 population clips between 10 and 14 Vpp. With 424 uA
        // of diode current across the 40 kohm input loop the final VCA stays
        // linear through that range and only rounds far beyond it.
        let at_10_vpp = retained_at_vpp(10.0);
        let at_14_vpp = retained_at_vpp(14.0);
        let at_40_vpp = retained_at_vpp(40.0);
        assert!(at_10_vpp > 0.999);
        assert!(at_14_vpp > 0.995);
        assert!(at_40_vpp < 0.9);
    }

    #[test]
    fn final_vca_preserves_small_signal_gain_and_has_a_finite_asymptote() {
        for (voice, profile) in FINAL_VCA_PROFILES.iter().copied().enumerate() {
            let small_signal_gain = final_voice(1.0e-4, 1.0, voice) / 1.0e-4;
            assert!((small_signal_gain / FINAL_VCA_THEVENIN_GAIN - 1.0).abs() < 1.0e-5);

            let expected_limit =
                FINAL_VCA_NOMINAL_LIMIT_VOLTS / profile.input_drive_ratio * FINAL_VCA_THEVENIN_GAIN;
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
    fn mixer_loading_preserves_the_single_150k_path_response() {
        for voice in 0..5 {
            for channel in [MixerChannel::OscillatorA, MixerChannel::OscillatorB] {
                assert_eq!(
                    oscillator_mixer_loaded(0.75, 1.0, 0.0, 0.0, 0.6, voice, channel),
                    oscillator_mixer(0.75, 0.6, voice, channel)
                );
            }
        }
    }

    #[test]
    fn parallel_waveform_paths_load_the_finite_mixer_input() {
        let one_path =
            oscillator_mixer_loaded(0.5, 1.0, 0.0, 0.0, 1.0, 2, MixerChannel::OscillatorA);
        let two_equal_paths =
            oscillator_mixer_loaded(1.0, 2.0, 0.0, 0.0, 1.0, 2, MixerChannel::OscillatorA);
        let unloaded_linear_sum = one_path * 2.0;
        assert!(two_equal_paths > one_path);
        assert!(two_equal_paths < unloaded_linear_sum);

        let one_path_conductance = 1.0 / MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS
            + 1.0 / MIXER_INPUT_SHUNT_RESISTANCE_OHMS
            + 1.0 / UNLINEARIZED_INPUT_RESISTANCE_OHMS;
        let two_path_conductance = 2.0 / MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS
            + 1.0 / MIXER_INPUT_SHUNT_RESISTANCE_OHMS
            + 1.0 / UNLINEARIZED_INPUT_RESISTANCE_OHMS;
        let expected_input_ratio = 2.0 * one_path_conductance / two_path_conductance;
        let small_one =
            oscillator_mixer_loaded(0.001, 1.0, 0.0, 0.0, 1.0, 2, MixerChannel::OscillatorA);
        let small_two =
            oscillator_mixer_loaded(0.002, 2.0, 0.0, 0.0, 1.0, 2, MixerChannel::OscillatorA);
        assert!((small_two / small_one - expected_input_ratio).abs() < 1.0e-5);
    }

    #[test]
    fn pulse_path_uses_its_populated_200k_conductance() {
        let loaded =
            oscillator_mixer_loaded(0.75, 0.75, 0.0, 0.0, 1.0, 2, MixerChannel::OscillatorA);
        let reference = oscillator_mixer(0.75, 1.0, 2, MixerChannel::OscillatorA);
        assert!(loaded > reference);
        assert!(loaded < reference * 1.12);
    }

    #[test]
    fn schematic_input_polarities_are_loaded_independently() {
        let positive =
            oscillator_mixer_loaded(5.0, 1.0, 0.0, 0.0, 1.0, 2, MixerChannel::OscillatorB);
        let negative =
            oscillator_mixer_loaded(0.0, 0.0, 5.0, 1.0, 1.0, 2, MixerChannel::OscillatorB);
        let cancelled =
            oscillator_mixer_loaded(5.0, 1.0, 5.0, 1.0, 1.0, 2, MixerChannel::OscillatorB);
        assert!(positive > 0.0);
        assert!((positive + negative).abs() < 1.0e-7);
        assert!(cancelled.abs() < 1.0e-7);

        let unequal_loading =
            oscillator_mixer_loaded(5.0, 1.0, 3.75, 0.75, 1.0, 2, MixerChannel::OscillatorB);
        assert_ne!(unequal_loading, 0.0);
    }

    #[test]
    fn absent_or_invalid_mixer_sources_are_silent() {
        for source in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                oscillator_mixer_loaded(1.0, source, 0.0, 0.0, 1.0, 0, MixerChannel::OscillatorA,),
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
            let low = poly_mod_oscillator_b_current_amps(0.7, 1.0, 0.25, voice).abs();
            let high = poly_mod_oscillator_b_current_amps(0.7, 1.0, 0.75, voice).abs();
            assert!(low < high);

            let linearized = poly_mod_filter_envelope_current_amps(1.0, 1.0, voice);
            let unlinearized = poly_mod_oscillator_b_current_amps(20.0, 1.0, 1.0, voice);
            let linearized_small = poly_mod_filter_envelope_current_amps(0.001, 1.0, voice) / 0.001;
            let unlinearized_small =
                poly_mod_oscillator_b_current_amps(0.001, 1.0, 1.0, voice) / 0.001;
            let linearized_retained = linearized / linearized_small;
            let unlinearized_retained = unlinearized / (20.0 * unlinearized_small);
            assert!(
                linearized_retained > 0.85,
                "voice {voice} linearized retained {linearized_retained}"
            );
            assert!(
                unlinearized_retained < linearized_retained,
                "voice {voice} linearized {linearized_retained}, unlinearized {unlinearized_retained}"
            );
        }
    }

    #[test]
    fn pmod_currents_share_the_populated_r4108_bus() {
        assert_eq!(POLY_MOD_BUS_LOAD_RESISTANCE_OHMS, 30_000.0);

        for voice in 0..5 {
            let oscillator_current = poly_mod_oscillator_b_current_amps(5.0, 1.0, 1.0, voice);
            let envelope_current = poly_mod_filter_envelope_current_amps(1.0, 1.0, voice);
            assert!(oscillator_current > 0.0);
            assert!(envelope_current > 0.0);

            let oscillator_bus = poly_mod_bus_voltage(0.0, oscillator_current);
            let envelope_bus = poly_mod_bus_voltage(envelope_current, 0.0);
            assert!((1.60..=1.95).contains(&oscillator_bus));
            assert!(
                (5.88..=7.30).contains(&envelope_bus),
                "voice {voice} envelope bus {envelope_bus} V",
            );
        }

        let overloaded = poly_mod_bus_voltage(1.0, 1.0);
        assert!(overloaded <= POLY_MOD_BUS_MINIMUM_OUTPUT_SWING_VOLTS);
        assert_eq!(poly_mod_bus_voltage(f32::NAN, 0.0), 0.0);
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
