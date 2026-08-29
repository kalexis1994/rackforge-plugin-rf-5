//! Band-limited numerical model of one CEM3340-class oscillator core.
//!
//! The chip topology and available outputs are source-backed. Mipmapped
//! periodic reconstruction, moving-edge PolyBLEP and four-times internal
//! oversampling are RF-5's numerical strategy, not claims about circuitry
//! inside the physical IC.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaveSelection {
    pub saw: bool,
    pub triangle: bool,
    pub pulse: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OscillatorSample {
    /// Conductance-weighted physical source voltage delivered to the positive
    /// oscillator-mixer input. SD431 routes saw here.
    pub mixer_positive_source_volts: f32,
    /// Selected positive-input conductance relative to one 150 kohm path.
    pub mixer_positive_source_conductance: f32,
    /// Conductance-weighted physical source voltage delivered to the negative
    /// oscillator-mixer input. SD431 routes pulse and oscillator-B triangle
    /// here, preserving their phase relationship to saw.
    pub mixer_negative_source_volts: f32,
    /// Selected negative-input conductance relative to one 150 kohm path.
    pub mixer_negative_source_conductance: f32,
    /// Conductance-weighted physical source voltage delivered to
    /// oscillator-B Poly Mod. U451 level-shifts only the triangle path.
    pub poly_mod_source_volts: f32,
    /// Sum of the selected Poly Mod source conductances relative to one
    /// 150 kohm path. All three sources meet one U428 input.
    pub poly_mod_source_conductance: f32,
    pub wrapped: bool,
    /// Fractional position of oscillator B's saw reset inside this internal
    /// sample. SD431 AC-couples that edge through C4107/Q401 into oscillator
    /// A's conventional hard-sync network.
    pub hard_sync_event: Option<HardSyncEvent>,
}

impl OscillatorSample {
    /// Unloaded differential source voltage, useful for oscillator-only
    /// spectral probes. The signal path itself loads both inputs separately.
    pub fn mixer_differential_source_volts(self) -> f32 {
        self.mixer_positive_source_volts - self.mixer_negative_source_volts
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HardSyncEvent {
    /// Position inside the current internal sample, from zero to one.
    pub offset: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Vco {
    phase: f32,
    profile_index: usize,
    previous_pulse_width: f32,
    pulse_width_initialized: bool,
    previous_pulse_harmonic_limit: f32,
    pulse_spectrum_initialized: bool,
    pulse_transition_from_harmonic_limit: f32,
    pulse_transition_remaining: u16,
}

impl Default for Vco {
    fn default() -> Self {
        Self {
            phase: 0.0,
            profile_index: 0,
            previous_pulse_width: 0.5,
            pulse_width_initialized: false,
            previous_pulse_harmonic_limit: 1.0,
            pulse_spectrum_initialized: false,
            pulse_transition_from_harmonic_limit: 1.0,
            pulse_transition_remaining: 0,
        }
    }
}

const OUTPUT_PROFILE_COUNT: usize = 10;
const SAW_UPPER_VOLTS: [f32; OUTPUT_PROFILE_COUNT] = [
    9.40, 9.55, 9.72, 9.88, 10.0, 10.12, 10.28, 10.42, 10.55, 10.60,
];
const SAW_LOWER_VOLTS: [f32; OUTPUT_PROFILE_COUNT] = [
    -0.025, 0.012, -0.018, 0.006, 0.0, 0.021, -0.009, 0.016, -0.004, 0.025,
];
const TRIANGLE_UPPER_VOLTS: [f32; OUTPUT_PROFILE_COUNT] =
    [4.85, 4.91, 4.96, 5.02, 5.08, 5.13, 4.88, 4.94, 5.05, 5.15];
const TRIANGLE_LOWER_VOLTS: [f32; OUTPUT_PROFILE_COUNT] = [
    -0.015, 0.007, -0.011, 0.004, 0.0, 0.013, -0.006, 0.009, -0.003, 0.015,
];
const TRIANGLE_SYMMETRY: [f32; OUTPUT_PROFILE_COUNT] = [
    0.450, 0.472, 0.489, 0.507, 0.529, 0.550, 0.462, 0.481, 0.518, 0.541,
];
const TRIANGLE_OUTPUT_IMPEDANCE_OHMS: [f32; OUTPUT_PROFILE_COUNT] = [
    65.0, 78.0, 91.0, 100.0, 112.0, 125.0, 138.0, 150.0, 84.0, 106.0,
];

// The voice board uses 150 kohm inputs for saw/triangle and 200 kohm for
// pulse. Each open-emitter pulse output is pulled toward -5 V through 10
// kohm before its ground-referenced 4016 selector. The selector's input clamp
// bounds the low state near one diode drop below ground. In the high state the
// pull-down draws more than the CEM3340 data sheet's 0.6 mA breakpoint, so its
// 1.3 kohm effective output resistance must be solved together with the 10
// kohm board load. Values below retain circuit volts and are expressed
// relative to one 150 kohm input conductance. Loading by the CA3280 input
// itself is applied at the VCA boundary, where every selected path is known.
const SAW_TRIANGLE_MIXER_CONDUCTANCE: f32 = 1.0;
const TRIANGLE_MIXER_LOAD_RESISTANCE_OHMS: f32 = 150_000.0;
const PULSE_MIXER_CONDUCTANCE: f32 = 150_000.0 / 200_000.0;
const PULSE_POSITIVE_SUPPLY_VOLTS: f32 = 15.0;
const PULSE_PULLDOWN_VOLTS: f32 = -5.0;
const PULSE_PULLDOWN_RESISTANCE_OHMS: f32 = 10_000.0;
const PULSE_HIGH_HEADROOM_VOLTS: f32 = 0.3;
const PULSE_HIGH_OUTPUT_RESISTANCE_OHMS: f32 = 1_300.0;
const PULSE_HIGH_CURRENT_BREAKPOINT_AMPS: f32 = 0.6e-3;
const PULSE_LOWER_VOLTS: f32 = -0.6;
const PULSE_UPPER_VOLTS: f32 =
    cem3340_loaded_pulse_high_volts(PULSE_PULLDOWN_VOLTS, PULSE_PULLDOWN_RESISTANCE_OHMS);
// SD431 derives 2.27 V TRI REF and U451 subtracts it from OSC B's raw
// positive-going triangle before the Poly Mod amount OTA.
const TRIANGLE_POLY_MOD_REFERENCE_VOLTS: f32 = 2.27;
// The populated 1 nF timing capacitor turns the CEM3340 data-sheet typical
// 570 uA charge/discharge boundary into 57 kHz:
// f = 3 I / (2 Vcc C), with Vcc = 15 V. Poly Mod can command considerably
// more exponential current than the core can use, so this physical ceiling
// must precede the numerical Nyquist guard below.
const CEM3340_TYPICAL_MAXIMUM_FREQUENCY_HZ: f32 = 57_000.0;
// The ordinary reset is reconstructed across 1.25 host samples in the
// portable four-times profile. This retains a visibly narrower, brighter edge than the
// earlier two-host-sample residual without introducing a raw discontinuity.
const SAW_POLY_BLEP_WIDTH: f32 = 5.0;
// Conventional hard sync introduces a second, non-periodic reset edge. Keep
// the admitted wider residual for the sync slave, where the extra reset is
// not periodic with the slave and therefore cannot use the normal treatment.
const SYNC_SAW_POLY_BLEP_WIDTH: f32 = 8.0;
// A moving PWM threshold needs a time-domain correction. Keep that correction
// two host samples wide in every active render profile.
#[cfg(feature = "host-rate")]
const PWM_POLY_BLEP_WIDTH: f32 = 2.0;
#[cfg(all(not(feature = "host-rate"), feature = "two-times"))]
const PWM_POLY_BLEP_WIDTH: f32 = 4.0;
#[cfg(not(any(feature = "host-rate", feature = "two-times")))]
const PWM_POLY_BLEP_WIDTH: f32 = 8.0;

// Highest admitted partial as a fraction of the active oscillator rate. The
// wavetable ends at 90% of host Nyquist in every profile, independently of
// whether the voice oscillator itself runs at one, two or four times host.
#[cfg(feature = "host-rate")]
const PULSE_HARMONIC_RATE_FRACTION: f32 = 0.45;
#[cfg(all(not(feature = "host-rate"), feature = "two-times"))]
const PULSE_HARMONIC_RATE_FRACTION: f32 = 0.225;
#[cfg(not(any(feature = "host-rate", feature = "two-times")))]
const PULSE_HARMONIC_RATE_FRACTION: f32 = 0.1125;

// A pitch S/H can replace several octaves in about 1.75 us. The physical
// comparator remains phase-continuous, while a band-limited representation
// must replace its harmonic basis. Crossfade the two complete periodic
// reconstructions for half a millisecond. Carrying only their first-sample
// difference as a decaying offset creates a non-physical unipolar pulse: after
// playing high notes that pulse reappears once on every card reassigned to a
// low note. Scaling by the active oscillator rate keeps this basis transition
// identical at the host boundary in every render profile.
#[cfg(feature = "host-rate")]
const PULSE_TRANSITION_INTERNAL_SAMPLES: u16 = 24;
#[cfg(all(not(feature = "host-rate"), feature = "two-times"))]
const PULSE_TRANSITION_INTERNAL_SAMPLES: u16 = 48;
#[cfg(not(any(feature = "host-rate", feature = "two-times")))]
const PULSE_TRANSITION_INTERNAL_SAMPLES: u16 = 96;

include!(concat!(env!("OUT_DIR"), "/pulse_wavetable.rs"));

/// CEM3340 pulse-output high level under a resistive pull-down.
///
/// Below 0.6 mA the data sheet specifies `V+ - 0.9 V`. Above that breakpoint
/// the open-emitter output behaves as `V+ - 0.3 V - 1.3 kohm * Ipull-down`.
/// Solving both branches here lets every board-level CEM3340 path share the
/// same electrical boundary while retaining its own pull-down voltage.
pub const fn cem3340_loaded_pulse_high_volts(
    pull_down_voltage_volts: f32,
    pull_down_resistance_ohms: f32,
) -> f32 {
    assert!(pull_down_resistance_ohms > 0.0);

    let low_current_high_volts = PULSE_POSITIVE_SUPPLY_VOLTS - 0.9;
    let low_current_amps =
        (low_current_high_volts - pull_down_voltage_volts) / pull_down_resistance_ohms;
    if low_current_amps < PULSE_HIGH_CURRENT_BREAKPOINT_AMPS {
        return low_current_high_volts;
    }

    let load_ratio = PULSE_HIGH_OUTPUT_RESISTANCE_OHMS / pull_down_resistance_ohms;
    (PULSE_POSITIVE_SUPPLY_VOLTS - PULSE_HIGH_HEADROOM_VOLTS + load_ratio * pull_down_voltage_volts)
        / (1.0 + load_ratio)
}

impl Vco {
    pub fn with_phase(phase: f32) -> Self {
        Self::with_phase_and_profile(phase, 0)
    }

    pub fn with_phase_and_profile(phase: f32, profile_index: usize) -> Self {
        let phase = if phase.is_finite() { phase % 1.0 } else { 0.0 };
        Self {
            phase: if phase < 0.0 { phase + 1.0 } else { phase },
            profile_index: profile_index % OUTPUT_PROFILE_COUNT,
            previous_pulse_width: 0.5,
            pulse_width_initialized: false,
            ..Self::default()
        }
    }

    /// Apply the negative pulse from the external CEM3340 hard-sync circuit.
    ///
    /// SD431 follows the data sheet's Figure 5 conventional-sync topology,
    /// forcing the triangle core back to the beginning of its cycle rather
    /// than using the bidirectional hard-sync input on pin 6.
    pub fn hard_sync_reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn phase(self) -> f32 {
        self.phase
    }

    /// Forget only the numerical pulse-spectrum transition at a keyboard
    /// pitch boundary.
    ///
    /// The CEM3340 core keeps running and therefore its phase must survive a
    /// voice reassignment. The mipmapped pulse reconstruction is not hardware
    /// state, however: carrying its previous-note basis into the newly gated
    /// note creates one short chirp per physical card after a large pitch
    /// jump. The first sample at the admitted pitch establishes a fresh safe
    /// harmonic basis without touching phase or PWM comparator state.
    pub(crate) fn admit_pitch_step(&mut self) {
        self.pulse_spectrum_initialized = false;
        self.pulse_transition_remaining = 0;
    }

    pub fn next(
        &mut self,
        frequency: f32,
        sample_rate: f32,
        pulse_width: f32,
        waves: WaveSelection,
    ) -> OscillatorSample {
        self.next_with_sync(frequency, sample_rate, pulse_width, waves, false, None)
    }

    /// Advance the powered oscillator core while none of its waveform outputs
    /// can reach the final VCA.
    ///
    /// Phase, triangle-output loading, PWM comparator memory and conventional
    /// sync timing remain live. Only waveform reconstruction is skipped. This
    /// is the electrically unobservable part of a closed voice-card path and
    /// avoids evaluating its filter and reconstruction FIR indefinitely.
    pub(crate) fn advance_silent(
        &mut self,
        frequency: f32,
        sample_rate: f32,
        pulse_width: f32,
        triangle_selected: bool,
        external_sync: Option<HardSyncEvent>,
    ) -> Option<HardSyncEvent> {
        let frequency = if frequency.is_finite() {
            frequency.clamp(0.0, CEM3340_TYPICAL_MAXIMUM_FREQUENCY_HZ)
        } else {
            0.0
        };
        let frequency =
            triangle_loaded_frequency(frequency, self.profile_index, triangle_selected);
        let increment = (frequency / sample_rate.max(1.0)).clamp(0.0, 0.49);
        let pulse_width = if pulse_width.is_finite() {
            pulse_width.clamp(0.0, 1.0)
        } else {
            0.5
        };
        let hard_sync_event = saw_hard_sync_event(self.phase, increment);
        self.previous_pulse_width = pulse_width;
        self.pulse_width_initialized = true;
        // Mipmap transitions are numerical output state, not capacitor state.
        // Establish a fresh safe basis when this output becomes observable.
        self.pulse_spectrum_initialized = false;
        self.pulse_transition_remaining = 0;
        let _ = self.advance_with_sync(increment, external_sync);
        hard_sync_event
    }

    pub fn next_with_sync(
        &mut self,
        frequency: f32,
        sample_rate: f32,
        pulse_width: f32,
        waves: WaveSelection,
        hard_sync_active: bool,
        external_sync: Option<HardSyncEvent>,
    ) -> OscillatorSample {
        let profile = self.profile_index;
        let frequency = if frequency.is_finite() {
            frequency.clamp(0.0, CEM3340_TYPICAL_MAXIMUM_FREQUENCY_HZ)
        } else {
            0.0
        };
        let frequency = triangle_loaded_frequency(frequency, profile, waves.triangle);
        let increment = (frequency / sample_rate.max(1.0)).clamp(0.0, 0.49);
        let pulse_width = if pulse_width.is_finite() {
            pulse_width.clamp(0.0, 1.0)
        } else {
            0.5
        };
        let previous_pulse_width = if self.pulse_width_initialized {
            self.previous_pulse_width
        } else {
            pulse_width
        };
        let phase = self.phase;
        let mut mixer_positive_source_volts = 0.0;
        let mut mixer_positive_source_conductance = 0.0;
        let mut mixer_negative_source_volts = 0.0;
        let mut mixer_negative_source_conductance = 0.0;
        let mut poly_mod_source_volts = 0.0;
        let mut poly_mod_source_conductance = 0.0;

        if waves.saw {
            let centered = if hard_sync_active {
                band_limited_sync_saw(phase, increment)
            } else {
                band_limited_saw(phase, increment)
            };
            let half_range = (SAW_UPPER_VOLTS[profile] - SAW_LOWER_VOLTS[profile]) * 0.5;
            let midpoint = (SAW_UPPER_VOLTS[profile] + SAW_LOWER_VOLTS[profile]) * 0.5;
            let source_volts = centered * half_range + midpoint;
            mixer_positive_source_volts += source_volts;
            poly_mod_source_volts += source_volts;
            mixer_positive_source_conductance += SAW_TRIANGLE_MIXER_CONDUCTANCE;
            poly_mod_source_conductance += SAW_TRIANGLE_MIXER_CONDUCTANCE;
        }
        if waves.triangle {
            let centered = band_limited_triangle(phase, increment, TRIANGLE_SYMMETRY[profile]);
            let half_range = (TRIANGLE_UPPER_VOLTS[profile] - TRIANGLE_LOWER_VOLTS[profile]) * 0.5;
            let midpoint = (TRIANGLE_UPPER_VOLTS[profile] + TRIANGLE_LOWER_VOLTS[profile]) * 0.5;
            let raw_source_volts = centered * half_range + midpoint;
            mixer_negative_source_volts += raw_source_volts;
            poly_mod_source_volts += raw_source_volts - TRIANGLE_POLY_MOD_REFERENCE_VOLTS;
            mixer_negative_source_conductance += SAW_TRIANGLE_MIXER_CONDUCTANCE;
            poly_mod_source_conductance += SAW_TRIANGLE_MIXER_CONDUCTANCE;
        }
        if waves.pulse {
            let centered = self.band_limited_pulse_continuous(
                phase,
                increment,
                previous_pulse_width,
                pulse_width,
            );
            let half_range = (PULSE_UPPER_VOLTS - PULSE_LOWER_VOLTS) * 0.5;
            let midpoint = (PULSE_UPPER_VOLTS + PULSE_LOWER_VOLTS) * 0.5;
            let equivalent_source_volts =
                (centered * half_range + midpoint) * PULSE_MIXER_CONDUCTANCE;
            mixer_negative_source_volts += equivalent_source_volts;
            poly_mod_source_volts += equivalent_source_volts;
            mixer_negative_source_conductance += PULSE_MIXER_CONDUCTANCE;
            poly_mod_source_conductance += PULSE_MIXER_CONDUCTANCE;
        }

        let hard_sync_event = saw_hard_sync_event(phase, increment);
        self.previous_pulse_width = pulse_width;
        self.pulse_width_initialized = true;
        let wrapped = self.advance_with_sync(increment, external_sync);

        OscillatorSample {
            mixer_positive_source_volts,
            mixer_positive_source_conductance,
            mixer_negative_source_volts,
            mixer_negative_source_conductance,
            poly_mod_source_volts,
            poly_mod_source_conductance,
            wrapped,
            hard_sync_event,
        }
    }

    fn band_limited_pulse_continuous(
        &mut self,
        phase: f32,
        phase_increment: f32,
        previous_pulse_width: f32,
        pulse_width: f32,
    ) -> f32 {
        if pulse_width <= 0.0 || pulse_width >= 1.0 {
            self.pulse_spectrum_initialized = false;
            self.pulse_transition_remaining = 0;
            return pulse_width * 2.0 - 1.0;
        }
        if (pulse_width - previous_pulse_width).abs() > f32::EPSILON {
            self.pulse_spectrum_initialized = false;
            self.pulse_transition_remaining = 0;
            return moving_threshold_pulse(
                phase,
                phase_increment,
                previous_pulse_width,
                pulse_width,
            );
        }

        let harmonic_limit = pulse_safe_harmonics(phase_increment);
        let raw = band_limited_pulse_at_harmonic_limit(phase, pulse_width, harmonic_limit);
        if self.pulse_spectrum_initialized {
            let previous_limit = self.previous_pulse_harmonic_limit;
            let ratio =
                harmonic_limit.max(previous_limit) / harmonic_limit.min(previous_limit).max(1.0);
            if ratio >= 1.25 {
                self.pulse_transition_from_harmonic_limit = previous_limit;
                self.pulse_transition_remaining = PULSE_TRANSITION_INTERNAL_SAMPLES;
            }
        } else {
            self.pulse_spectrum_initialized = true;
        }
        self.previous_pulse_harmonic_limit = harmonic_limit;

        if self.pulse_transition_remaining > 0 {
            let previous = band_limited_pulse_at_harmonic_limit(
                phase,
                pulse_width,
                self.pulse_transition_from_harmonic_limit,
            );
            let blend =
                f32::from(PULSE_TRANSITION_INTERNAL_SAMPLES - self.pulse_transition_remaining)
                    / f32::from(PULSE_TRANSITION_INTERNAL_SAMPLES);
            self.pulse_transition_remaining -= 1;
            previous + (raw - previous) * blend
        } else {
            raw
        }
    }

    fn advance_with_sync(&mut self, increment: f32, sync_event: Option<HardSyncEvent>) -> bool {
        if let Some(event) = sync_event {
            let offset = if event.offset.is_finite() {
                event.offset.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let wrapped = self.advance_phase(increment * offset);
            self.hard_sync_reset();
            return wrapped | self.advance_phase(increment * (1.0 - offset));
        }
        self.advance_phase(increment)
    }

    fn advance_phase(&mut self, increment: f32) -> bool {
        let advanced = self.phase + increment;
        if advanced >= 1.0 {
            self.phase = advanced - 1.0;
            true
        } else {
            self.phase = advanced;
            false
        }
    }
}

fn triangle_loaded_frequency(frequency: f32, profile: usize, triangle_selected: bool) -> f32 {
    if !triangle_selected {
        return frequency;
    }

    frequency * triangle_load_frequency_ratio(profile)
}

pub(crate) fn triangle_load_frequency_ratio(profile: usize) -> f32 {
    // The triangle buffer also drives the internal comparator, so its finite
    // output impedance lets an external load pull oscillator frequency. The
    // CEM3340 sheet gives the first-order reduction directly as Rout/Rload.
    let pull = TRIANGLE_OUTPUT_IMPEDANCE_OHMS[profile % OUTPUT_PROFILE_COUNT]
        / TRIANGLE_MIXER_LOAD_RESISTANCE_OHMS;
    1.0 - pull
}

fn saw_hard_sync_event(phase: f32, increment: f32) -> Option<HardSyncEvent> {
    if increment <= 0.0 {
        return None;
    }
    let distance = 1.0 - phase;
    let offset = distance / increment;
    (offset <= 1.0).then_some(HardSyncEvent {
        offset: offset.max(0.0),
    })
}

fn band_limited_saw(phase: f32, increment: f32) -> f32 {
    let naive = phase * 2.0 - 1.0;
    naive - poly_blep(phase, (increment * SAW_POLY_BLEP_WIDTH).min(0.5))
}

fn band_limited_sync_saw(phase: f32, increment: f32) -> f32 {
    let naive = phase * 2.0 - 1.0;
    naive - poly_blep(phase, (increment * SYNC_SAW_POLY_BLEP_WIDTH).min(0.5))
}

#[cfg(test)]
pub(crate) fn band_limited_pulse(
    phase: f32,
    phase_increment: f32,
    previous_pulse_width: f32,
    pulse_width: f32,
) -> f32 {
    if pulse_width <= 0.0 || pulse_width >= 1.0 {
        return pulse_width * 2.0 - 1.0;
    }
    if (pulse_width - previous_pulse_width).abs() > f32::EPSILON {
        return moving_threshold_pulse(phase, phase_increment, previous_pulse_width, pulse_width);
    }
    // Keep the highest generated partial below 90% of the active profile's
    // host Nyquist boundary. This is compile-time profile data, so the audio
    // loop pays no branch for portable 1x versus reference 2x/4x rendering.
    let safe_harmonics = pulse_safe_harmonics(phase_increment);
    band_limited_pulse_at_harmonic_limit(phase, pulse_width, safe_harmonics)
}

fn pulse_safe_harmonics(phase_increment: f32) -> f32 {
    (PULSE_HARMONIC_RATE_FRACTION / phase_increment.max(f32::MIN_POSITIVE)).max(1.0)
}

fn band_limited_pulse_at_harmonic_limit(phase: f32, pulse_width: f32, safe_harmonics: f32) -> f32 {
    let lower_level = pulse_harmonic_level(safe_harmonics as u32);
    let upper_level = (lower_level + 1).min(PULSE_HARMONIC_LEVELS - 1);
    let upper_harmonics = f32::from(PULSE_HARMONIC_COUNTS[upper_level]);
    // The nominal table boundary is 90% of host Nyquist. Introduce the next
    // table only after all of its partials lie below true Nyquist, then finish
    // the crossfade at the conservative boundary. This avoids octave-spaced
    // timbre steps without admitting a folded partial.
    let transition_start = upper_harmonics * 0.9;
    let blend = if upper_level == lower_level {
        0.0
    } else {
        ((safe_harmonics - transition_start) / (upper_harmonics - transition_start)).clamp(0.0, 1.0)
    };
    let lower = band_limited_pulse_from_table(phase, pulse_width, lower_level);
    let upper = band_limited_pulse_from_table(phase, pulse_width, upper_level);
    lower + (upper - lower) * blend
}

fn moving_threshold_pulse(
    phase: f32,
    phase_increment: f32,
    previous_pulse_width: f32,
    pulse_width: f32,
) -> f32 {
    let naive = if phase < pulse_width { 1.0 } else { -1.0 };
    let rising_correction = poly_blep(phase, pwm_blep_width(phase_increment));
    let threshold_velocity = phase_increment - (pulse_width - previous_pulse_width);
    if threshold_velocity >= 0.0 {
        let falling_phase = if phase >= pulse_width {
            phase - pulse_width
        } else {
            phase + (1.0 - pulse_width)
        };
        naive + rising_correction
            - poly_blep(falling_phase, pwm_blep_width(threshold_velocity.abs()))
    } else {
        let rising_threshold_phase = if pulse_width >= phase {
            pulse_width - phase
        } else {
            pulse_width + (1.0 - phase)
        };
        naive
            + rising_correction
            + poly_blep(
                rising_threshold_phase,
                pwm_blep_width(threshold_velocity.abs()),
            )
    }
}

fn pulse_harmonic_level(safe_harmonics: u32) -> usize {
    let safe = safe_harmonics.clamp(1, 1_024);
    match safe {
        1..=32 => (safe - 1) as usize,
        33..=64 => (31 + (safe - 32) / 2) as usize,
        65..=128 => (47 + (safe - 64) / 4) as usize,
        129..=256 => (63 + (safe - 128) / 8) as usize,
        257..=512 => (79 + (safe - 256) / 16) as usize,
        _ => (95 + (safe - 512) / 32) as usize,
    }
}

fn band_limited_pulse_from_table(phase: f32, pulse_width: f32, level: usize) -> f32 {
    let delayed_phase = if phase >= pulse_width {
        phase - pulse_width
    } else {
        phase + (1.0 - pulse_width)
    };
    band_limited_saw_lookup(delayed_phase, level) - band_limited_saw_lookup(phase, level)
        + 2.0 * pulse_width
        - 1.0
}

fn band_limited_saw_lookup(phase: f32, level: usize) -> f32 {
    // Oscillator and delayed-edge phases are already normalized in the audio
    // path. Keep the general wrapping fallback for probes and tests without
    // paying for a floating-point remainder on every oversampled oscillator
    // evaluation.
    let wrapped_phase = if (0.0..1.0).contains(&phase) {
        phase
    } else {
        let remainder = phase % 1.0;
        if remainder < 0.0 {
            remainder + 1.0
        } else {
            remainder
        }
    };
    let position = wrapped_phase * PULSE_TABLE_SIZE as f32;
    let base_index = position as usize;
    let fraction = position - base_index as f32;
    let mask = PULSE_TABLE_SIZE - 1;
    let index = base_index & mask;
    let table = &BAND_LIMITED_SAW_TABLES[level];
    let current = table[index];
    let next = table[(index + 1) & mask];
    // At 4096 samples per periodic table, linear interpolation is already far
    // below the admitted oscillator/model uncertainty. It preserves the same
    // mip level and harmonic content while halving table reads and removing
    // the cubic polynomial from the four-times-oversampled hot path.
    current + (next - current) * fraction
}

fn pwm_blep_width(increment: f32) -> f32 {
    (increment * PWM_POLY_BLEP_WIDTH).min(0.5)
}

pub(crate) fn naive_triangle(phase: f32, symmetry: f32) -> f32 {
    let symmetry = symmetry.clamp(0.01, 0.99);
    if phase < symmetry {
        -1.0 + 2.0 * phase / symmetry
    } else {
        1.0 - 2.0 * (phase - symmetry) / (1.0 - symmetry)
    }
}

pub(crate) fn band_limited_triangle(phase: f32, increment: f32, symmetry: f32) -> f32 {
    let symmetry = symmetry.clamp(0.01, 0.99);
    let rising_slope = 2.0 / symmetry;
    let falling_slope = -2.0 / (1.0 - symmetry);
    let correction_width = blamp_width(increment);
    let peak_phase = if phase >= symmetry {
        phase - symmetry
    } else {
        phase + (1.0 - symmetry)
    };

    naive_triangle(phase, symmetry)
        + 0.5 * (rising_slope - falling_slope) * poly_blamp(phase, correction_width)
        + 0.5 * (falling_slope - rising_slope) * poly_blamp(peak_phase, correction_width)
}

/// Periodic integral of the PolyBLEP residual around one discontinuity.
///
/// A triangle has no value discontinuity, but each corner changes slope. The
/// integrated residual rounds only that local slope transition and returns to
/// exactly zero outside the two-sided correction window.
fn poly_blamp(phase: f32, width: f32) -> f32 {
    if width <= 0.0 {
        return 0.0;
    }
    if phase < width {
        let distance = 1.0 - phase / width;
        return width * distance * distance * distance / 3.0;
    }
    if phase > 1.0 - width {
        let distance = (phase - (1.0 - width)) / width;
        return width * distance * distance * distance / 3.0;
    }
    0.0
}

fn blamp_width(increment: f32) -> f32 {
    increment.min(0.5)
}

fn poly_blep(phase: f32, increment: f32) -> f32 {
    if increment <= 0.0 {
        return 0.0;
    }
    if phase < increment {
        let x = phase / increment;
        return x + x - x * x - 1.0;
    }
    if phase > 1.0 - increment {
        let x = (phase - 1.0) / increment;
        return x * x + x + x + 1.0;
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAW: WaveSelection = WaveSelection {
        saw: true,
        triangle: false,
        pulse: false,
    };
    const PULSE: WaveSelection = WaveSelection {
        saw: false,
        triangle: false,
        pulse: true,
    };

    #[test]
    fn phase_wrap_is_reported_and_bounded() {
        let mut oscillator = Vco::with_phase(0.99);
        let sample = oscillator.next(1_000.0, 48_000.0, 0.5, SAW);
        assert!(sample.wrapped);
        assert!((0.0..1.0).contains(&oscillator.phase()));
    }

    #[test]
    fn triangle_blamp_changes_only_the_corner_windows() {
        let increment = 0.005;
        let width = blamp_width(increment);
        let symmetry = 0.5;
        for phase in [width + 0.01, symmetry - width - 0.01] {
            assert_eq!(
                band_limited_triangle(phase, increment, symmetry),
                naive_triangle(phase, symmetry)
            );
        }

        let expected_corner_delta = 4.0 * width / 3.0;
        assert!(
            (band_limited_triangle(0.0, increment, symmetry) - (-1.0 + expected_corner_delta))
                .abs()
                < 1.0e-6
        );
        assert!(
            (band_limited_triangle(symmetry, increment, symmetry) - (1.0 - expected_corner_delta))
                .abs()
                < 1.0e-6
        );
    }

    #[test]
    fn triangle_blamp_is_continuous_at_both_asymmetric_corners() {
        let increment = 0.01;
        let epsilon = 1.0e-5;
        for symmetry in [0.45, 0.5, 0.55] {
            let before_peak = band_limited_triangle(symmetry - epsilon, increment, symmetry);
            let after_peak = band_limited_triangle(symmetry + epsilon, increment, symmetry);
            assert!((before_peak - after_peak).abs() < 1.0e-5);

            let before_wrap = band_limited_triangle(1.0 - epsilon, increment, symmetry);
            let after_wrap = band_limited_triangle(epsilon, increment, symmetry);
            assert!((before_wrap - after_wrap).abs() < 1.0e-5);
        }
    }

    #[test]
    fn triangle_blamp_preserves_dc_and_profile_bounds() {
        const SAMPLE_COUNT: usize = 65_536;
        for symmetry in TRIANGLE_SYMMETRY {
            let mut sum = 0.0;
            let mut minimum = f32::INFINITY;
            let mut maximum = f32::NEG_INFINITY;
            for index in 0..SAMPLE_COUNT {
                let phase = index as f32 / SAMPLE_COUNT as f32;
                let sample = band_limited_triangle(phase, 0.01, symmetry);
                sum += sample;
                minimum = minimum.min(sample);
                maximum = maximum.max(sample);
            }
            assert!((sum / SAMPLE_COUNT as f32).abs() < 2.0e-5);
            assert!(minimum >= -1.0);
            assert!(maximum <= 1.0);
        }
    }

    #[test]
    fn triangle_blamp_preserves_a4_corner_level_at_supported_rates() {
        for host_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let increment = 440.0 / (host_rate * 4.0);
            for symmetry in TRIANGLE_SYMMETRY {
                let naive_peak = naive_triangle(symmetry, symmetry);
                let corrected_peak = band_limited_triangle(symmetry, increment, symmetry);
                assert!(naive_peak - corrected_peak < 0.004);
            }
        }
    }

    #[test]
    fn pulse_width_changes_duty_cycle() {
        let mut narrow = Vco::default();
        let mut wide = Vco::default();
        let mut narrow_positive = 0;
        let mut wide_positive = 0;
        for _ in 0..1_000 {
            narrow_positive += (narrow
                .next(100.0, 10_000.0, 0.25, PULSE)
                .mixer_negative_source_volts
                > 0.0) as usize;
            wide_positive += (wide
                .next(100.0, 10_000.0, 0.75, PULSE)
                .mixer_negative_source_volts
                > 0.0) as usize;
        }
        assert!(narrow_positive < wide_positive);
    }

    #[test]
    fn moving_pwm_is_phase_continuous_symmetric_and_bounded() {
        let rising = band_limited_pulse(0.50, 0.0, 0.49, 0.51);
        let falling = band_limited_pulse(0.50, 0.0, 0.51, 0.49);
        let stationary_high = band_limited_pulse(0.50, 0.0, 0.51, 0.51);
        let stationary_low = band_limited_pulse(0.50, 0.0, 0.49, 0.49);

        assert!((0.0..1.0).contains(&rising));
        assert!((-1.0..0.0).contains(&falling));
        assert!((rising + falling).abs() < 1.0e-6);
        assert!(stationary_high.is_finite());
        assert!(stationary_low.is_finite());
        assert!(stationary_high.abs() <= 1.0);
        assert!(stationary_low.abs() <= 1.0);
    }

    #[test]
    fn pulse_mipmap_selects_the_largest_safe_harmonic_table() {
        for harmonic in 1..=32 {
            assert_eq!(
                PULSE_HARMONIC_COUNTS[pulse_harmonic_level(harmonic)],
                harmonic as u16
            );
        }
        for harmonic in [33, 34, 63, 64, 65, 68, 127, 128, 129, 136, 255, 256, 1_024] {
            let selected = u32::from(PULSE_HARMONIC_COUNTS[pulse_harmonic_level(harmonic)]);
            assert!(selected <= harmonic);
            let next = pulse_harmonic_level(harmonic) + 1;
            assert!(
                next == PULSE_HARMONIC_LEVELS || u32::from(PULSE_HARMONIC_COUNTS[next]) > harmonic
            );
        }
    }

    #[test]
    fn large_pitch_step_crossfades_periodic_pulse_bases_without_a_dc_ramp() {
        let mut oscillator = Vco::with_phase(0.317);
        let pulse_width = 0.5;
        let high_increment = 4_000.0 / 48_000.0;
        let low_increment = 110.0 / 48_000.0;
        let high_limit = pulse_safe_harmonics(high_increment);
        let low_limit = pulse_safe_harmonics(low_increment);

        let _ = oscillator.band_limited_pulse_continuous(
            0.317,
            high_increment,
            pulse_width,
            pulse_width,
        );
        let first_phase = 0.401;
        let first = oscillator.band_limited_pulse_continuous(
            first_phase,
            low_increment,
            pulse_width,
            pulse_width,
        );
        let previous_first =
            band_limited_pulse_at_harmonic_limit(first_phase, pulse_width, high_limit);
        assert!((first - previous_first).abs() < 1.0e-6);
        assert_eq!(
            oscillator.pulse_transition_remaining,
            PULSE_TRANSITION_INTERNAL_SAMPLES - 1
        );

        let second_phase = 0.409;
        let second = oscillator.band_limited_pulse_continuous(
            second_phase,
            low_increment,
            pulse_width,
            pulse_width,
        );
        let previous_second =
            band_limited_pulse_at_harmonic_limit(second_phase, pulse_width, high_limit);
        let current_second =
            band_limited_pulse_at_harmonic_limit(second_phase, pulse_width, low_limit);
        let expected_second = previous_second
            + (current_second - previous_second) / f32::from(PULSE_TRANSITION_INTERNAL_SAMPLES);
        assert!((second - expected_second).abs() < 1.0e-6);

        for index in 2..PULSE_TRANSITION_INTERNAL_SAMPLES {
            let phase = (second_phase + f32::from(index) * low_increment) % 1.0;
            let _ = oscillator.band_limited_pulse_continuous(
                phase,
                low_increment,
                pulse_width,
                pulse_width,
            );
        }
        assert_eq!(oscillator.pulse_transition_remaining, 0);

        let settled_phase = 0.731;
        let settled = oscillator.band_limited_pulse_continuous(
            settled_phase,
            low_increment,
            pulse_width,
            pulse_width,
        );
        let expected_settled =
            band_limited_pulse_at_harmonic_limit(settled_phase, pulse_width, low_limit);
        assert!((settled - expected_settled).abs() < 1.0e-6);
    }

    #[test]
    fn admitted_keyboard_pitch_discards_only_previous_note_reconstruction_state() {
        let mut oscillator = Vco::with_phase(0.317);
        let pulse_width = 0.5;
        let high_increment = 4_000.0 / 48_000.0;
        let low_increment = 110.0 / 48_000.0;
        let phase = oscillator.phase();

        let _ = oscillator.band_limited_pulse_continuous(
            phase,
            high_increment,
            pulse_width,
            pulse_width,
        );
        oscillator.admit_pitch_step();
        assert_eq!(oscillator.phase(), phase);

        let admitted = oscillator.band_limited_pulse_continuous(
            phase,
            low_increment,
            pulse_width,
            pulse_width,
        );
        let expected = band_limited_pulse_at_harmonic_limit(
            phase,
            pulse_width,
            pulse_safe_harmonics(low_increment),
        );
        assert!((admitted - expected).abs() < 1.0e-6);
        assert_eq!(oscillator.pulse_transition_remaining, 0);
    }

    #[test]
    fn pulse_table_lookup_wraps_at_the_phase_boundary() {
        for level in [0, PULSE_HARMONIC_LEVELS / 2, PULSE_HARMONIC_LEVELS - 1] {
            assert_eq!(
                band_limited_saw_lookup(1.0, level),
                band_limited_saw_lookup(0.0, level)
            );
            assert_eq!(
                band_limited_saw_lookup(2.0, level),
                band_limited_saw_lookup(0.0, level)
            );
        }
    }

    #[test]
    fn static_pwm_endpoints_cancel_both_band_limited_edges_exactly() {
        for pulse_width in [0.0, 1.0] {
            let expected = pulse_width * 2.0 - 1.0;
            for phase in [0.0, 0.001, 0.25, 0.75, 0.999] {
                assert_eq!(
                    band_limited_pulse(phase, 0.01, pulse_width, pulse_width),
                    expected
                );
            }
        }
    }

    #[test]
    fn invalid_pwm_control_recovers_to_the_physical_midpoint() {
        let mut oscillator = Vco::default();
        let sample = oscillator.next(440.0, 48_000.0, f32::NAN, PULSE);
        assert!(sample.mixer_negative_source_volts.is_finite());
        assert_eq!(oscillator.previous_pulse_width, 0.5);
    }

    #[test]
    fn pulse_dc_endpoints_are_stable_while_saw_sync_clock_keeps_running() {
        for (width, expected) in [
            (0.0, PULSE_LOWER_VOLTS * PULSE_MIXER_CONDUCTANCE),
            (1.0, PULSE_UPPER_VOLTS * PULSE_MIXER_CONDUCTANCE),
        ] {
            let mut oscillator = Vco::default();
            let mut sync_edges = 0;
            for _ in 0..2_000 {
                let sample = oscillator.next(440.0, 48_000.0, width, PULSE);
                assert!((sample.mixer_negative_source_volts - expected).abs() < 1.0e-6);
                sync_edges += usize::from(sample.hard_sync_event.is_some());
            }
            assert!((17..=19).contains(&sync_edges));
        }
    }

    #[test]
    fn one_and_ninety_nine_percent_remain_complementary_pulses() {
        let mut narrow = Vco::default();
        let mut wide = Vco::default();
        let mut narrow_sum = 0.0;
        let mut wide_sum = 0.0;
        let mut narrow_edges = 0;
        let mut wide_edges = 0;
        for _ in 0..10_000 {
            let narrow_sample = narrow.next(100.0, 10_000.0, 0.01, PULSE);
            let wide_sample = wide.next(100.0, 10_000.0, 0.99, PULSE);
            narrow_sum += narrow_sample.mixer_negative_source_volts;
            wide_sum += wide_sample.mixer_negative_source_volts;
            narrow_edges += usize::from(narrow_sample.hard_sync_event.is_some());
            wide_edges += usize::from(wide_sample.hard_sync_event.is_some());
        }
        let narrow_mean = narrow_sum / 10_000.0;
        let wide_mean = wide_sum / 10_000.0;
        let midpoint = (PULSE_UPPER_VOLTS + PULSE_LOWER_VOLTS) * 0.5 * PULSE_MIXER_CONDUCTANCE;
        let half_range = (PULSE_UPPER_VOLTS - PULSE_LOWER_VOLTS) * 0.5 * PULSE_MIXER_CONDUCTANCE;
        assert!((narrow_mean + wide_mean - 2.0 * midpoint).abs() < 2.0e-3);
        assert!(((narrow_mean - midpoint) / half_range + 0.98).abs() < 0.01);
        assert!(((wide_mean - midpoint) / half_range - 0.98).abs() < 0.01);
        assert!((narrow_edges as isize - 100).abs() <= 2);
        assert!((wide_edges as isize - 100).abs() <= 2);
    }

    #[test]
    fn external_hard_sync_resets_either_triangle_branch() {
        for phase in [0.20, 0.80] {
            let mut oscillator = Vco::with_phase(phase);
            oscillator.hard_sync_reset();
            assert_eq!(oscillator.phase(), 0.0);
        }
    }

    #[test]
    fn hard_sync_retains_the_slave_saw_ac_excursion() {
        let mut master = Vco::with_phase(0.317);
        let mut slave = Vco::with_phase(0.731);
        let silent = WaveSelection::default();
        let saw = WaveSelection {
            saw: true,
            ..WaveSelection::default()
        };
        let mut sum = 0.0_f64;
        let mut square_sum = 0.0_f64;
        let frames = 48_000;
        for _ in 0..frames {
            let master_sample = master.next(880.0, 48_000.0, 0.472_441, silent);
            let slave_sample = slave.next_with_sync(
                1_046.502_3,
                48_000.0,
                0.5,
                saw,
                true,
                master_sample.hard_sync_event,
            );
            let value = f64::from(slave_sample.mixer_positive_source_volts);
            sum += value;
            square_sum += value * value;
        }
        let mean = sum / f64::from(frames);
        let ac_rms = (square_sum / f64::from(frames) - mean * mean).sqrt();
        assert!(ac_rms > 2.0, "hard-sync saw AC RMS collapsed to {ac_rms}");
    }

    #[test]
    fn saw_reset_reports_the_hard_sync_edge_independently_of_pulse_width() {
        let mut oscillator = Vco::with_phase(0.45);
        let before_wrap = oscillator.next(1_000.0, 10_000.0, 0.01, SAW);
        assert_eq!(before_wrap.hard_sync_event, None);

        let mut oscillator = Vco::with_phase(0.95);
        let wrap = oscillator.next(1_000.0, 10_000.0, 0.99, SAW);
        assert!((wrap.hard_sync_event.unwrap().offset - 0.5).abs() < 1.0e-6);
    }

    #[test]
    fn sync_edges_exist_even_when_pulse_is_not_selected_for_audio() {
        let mut oscillator = Vco::with_phase(0.95);
        let sample = oscillator.next(1_000.0, 10_000.0, 0.50, WaveSelection::default());
        assert_eq!(sample.mixer_positive_source_volts, 0.0);
        assert_eq!(sample.mixer_negative_source_volts, 0.0);
        assert_eq!(sample.poly_mod_source_volts, 0.0);
        assert!(sample.hard_sync_event.is_some());
    }

    #[test]
    fn saw_reset_retains_its_fractional_position() {
        let mut oscillator = Vco::with_phase(0.95);
        let sample = oscillator.next(4_000.0, 10_000.0, 0.10, WaveSelection::default());
        assert!((sample.hard_sync_event.unwrap().offset - 0.125).abs() < 1.0e-6);
    }

    #[test]
    fn external_sync_is_applied_at_its_sub_sample_position() {
        let mut oscillator = Vco::with_phase_and_profile(0.20, 0);
        let sample = oscillator.next_with_sync(
            1_000.0,
            10_000.0,
            0.5,
            SAW,
            true,
            Some(HardSyncEvent { offset: 0.25 }),
        );

        let expected_phase = 0.10 * 0.75;
        assert!((oscillator.phase() - expected_phase).abs() < 1.0e-6);

        let centered = band_limited_sync_saw(0.20, 0.10);
        let expected_audio = centered * (SAW_UPPER_VOLTS[0] - SAW_LOWER_VOLTS[0]) * 0.5
            + (SAW_UPPER_VOLTS[0] + SAW_LOWER_VOLTS[0]) * 0.5;
        assert!((sample.mixer_positive_source_volts - expected_audio).abs() < 1.0e-6);
    }

    #[test]
    fn invalid_sync_offsets_cannot_break_phase_bounds() {
        let mut oscillator = Vco::with_phase(0.25);
        for _ in 0..1_000 {
            let sample = oscillator.next_with_sync(
                9_000.0,
                48_000.0,
                0.5,
                SAW,
                true,
                Some(HardSyncEvent { offset: f32::NAN }),
            );
            assert!(sample.mixer_differential_source_volts().is_finite());
            assert!((0.0..1.0).contains(&oscillator.phase()));
        }
    }

    #[test]
    fn all_wave_combinations_stay_finite() {
        let mut oscillator = Vco::default();
        let waves = WaveSelection {
            saw: true,
            triangle: true,
            pulse: true,
        };
        for _ in 0..10_000 {
            let sample = oscillator.next(12_000.0, 48_000.0, 0.37, waves);
            assert!(sample.mixer_positive_source_volts.is_finite());
            assert!(sample.mixer_negative_source_volts.is_finite());
            assert!(sample.poly_mod_source_volts.is_finite());
            assert!(sample.mixer_positive_source_volts.abs() <= 11.0);
            assert!(sample.mixer_negative_source_volts.abs() <= 16.0);
            assert!(sample.poly_mod_source_volts.abs() <= 25.0);
        }
    }

    #[test]
    fn all_output_profiles_stay_inside_data_sheet_limits() {
        for profile in 0..OUTPUT_PROFILE_COUNT {
            assert!((9.4..=10.6).contains(&SAW_UPPER_VOLTS[profile]));
            assert!((-0.025..=0.025).contains(&SAW_LOWER_VOLTS[profile]));
            assert!((4.85..=5.15).contains(&TRIANGLE_UPPER_VOLTS[profile]));
            assert!((-0.015..=0.015).contains(&TRIANGLE_LOWER_VOLTS[profile]));
            assert!((0.45..=0.55).contains(&TRIANGLE_SYMMETRY[profile]));
            assert!((65.0..=150.0).contains(&TRIANGLE_OUTPUT_IMPEDANCE_OHMS[profile]));
        }
    }

    #[test]
    fn pulse_high_level_solves_the_populated_ten_kilohm_load() {
        let pull_down_current_amps =
            (PULSE_UPPER_VOLTS - PULSE_PULLDOWN_VOLTS) / PULSE_PULLDOWN_RESISTANCE_OHMS;
        assert!(pull_down_current_amps > PULSE_HIGH_CURRENT_BREAKPOINT_AMPS);

        let data_sheet_high_volts = PULSE_POSITIVE_SUPPLY_VOLTS
            - PULSE_HIGH_HEADROOM_VOLTS
            - PULSE_HIGH_OUTPUT_RESISTANCE_OHMS * pull_down_current_amps;
        assert!((PULSE_UPPER_VOLTS - data_sheet_high_volts).abs() < 1.0e-6);
        assert!((PULSE_UPPER_VOLTS - 12.433_628).abs() < 1.0e-5);
    }

    #[test]
    fn pulse_high_solver_selects_both_data_sheet_current_regions() {
        let ground_loaded = cem3340_loaded_pulse_high_volts(0.0, 10_000.0);
        let lightly_loaded = cem3340_loaded_pulse_high_volts(0.0, 100_000.0);

        assert!((ground_loaded - 13.008_85).abs() < 1.0e-5);
        assert!((lightly_loaded - 14.1).abs() < 1.0e-6);
    }

    #[test]
    fn triangle_load_reproduces_the_data_sheet_frequency_pull() {
        let frequency = 1_000.0;
        let profile = 7;
        let loaded = triangle_loaded_frequency(frequency, profile, true);
        let fractional_pull = 1.0 - loaded / frequency;

        assert!((fractional_pull - 150.0 / 150_000.0).abs() < 1.0e-7);
        let cents = 1_200.0 * libm::log2f(loaded / frequency);
        assert!((-1.74..-1.72).contains(&cents));
    }

    #[test]
    fn only_a_selected_triangle_loads_the_oscillator_core() {
        let mut saw = Vco::with_phase_and_profile(0.0, 7);
        let mut pulse = Vco::with_phase_and_profile(0.0, 7);
        let mut triangle = Vco::with_phase_and_profile(0.0, 7);
        let triangle_wave = WaveSelection {
            saw: false,
            triangle: true,
            pulse: false,
        };

        saw.next(1_000.0, 48_000.0, 0.5, SAW);
        pulse.next(1_000.0, 48_000.0, 0.5, PULSE);
        triangle.next(1_000.0, 48_000.0, 0.5, triangle_wave);

        assert_eq!(saw.phase(), pulse.phase());
        assert!(triangle.phase() < saw.phase());
        let expected = triangle_loaded_frequency(1_000.0, 7, true) / 48_000.0;
        assert!((triangle.phase() - expected).abs() < 1.0e-7);
    }

    #[test]
    fn triangle_audio_is_raw_while_poly_mod_is_level_shifted() {
        let profile = 4;
        let mut low = Vco::with_phase_and_profile(0.0, profile);
        let mut high = Vco::with_phase_and_profile(TRIANGLE_SYMMETRY[profile], profile);
        let waves = WaveSelection {
            saw: false,
            triangle: true,
            pulse: false,
        };
        let low = low.next(0.0, 48_000.0, 0.5, waves);
        let high = high.next(0.0, 48_000.0, 0.5, waves);
        let triangle_low = low.mixer_negative_source_volts;
        let triangle_high = high.mixer_negative_source_volts;
        let saw_peak_to_peak = SAW_UPPER_VOLTS[profile] - SAW_LOWER_VOLTS[profile];
        let triangle_peak_to_peak = triangle_high - triangle_low;
        assert!((triangle_low - TRIANGLE_LOWER_VOLTS[profile]).abs() < 1.0e-6);
        assert!((triangle_high - TRIANGLE_UPPER_VOLTS[profile]).abs() < 1.0e-6);
        assert!(
            (low.poly_mod_source_volts
                - (TRIANGLE_LOWER_VOLTS[profile] - TRIANGLE_POLY_MOD_REFERENCE_VOLTS))
                .abs()
                < 1.0e-6
        );
        assert!(
            (high.poly_mod_source_volts
                - (TRIANGLE_UPPER_VOLTS[profile] - TRIANGLE_POLY_MOD_REFERENCE_VOLTS))
                .abs()
                < 1.0e-6
        );
        assert!((triangle_peak_to_peak / saw_peak_to_peak - 0.508).abs() < 0.01);
    }

    #[test]
    fn poly_mod_preserves_documented_waveform_polarities() {
        let mut saw = Vco::with_phase_and_profile(0.25, 4);
        let mut triangle = Vco::with_phase_and_profile(0.25, 4);
        let mut pulse = Vco::with_phase_and_profile(0.25, 4);
        let saw_sample = saw.next(0.0, 48_000.0, 0.5, SAW);
        let triangle_sample = triangle.next(
            0.0,
            48_000.0,
            0.5,
            WaveSelection {
                saw: false,
                triangle: true,
                pulse: false,
            },
        );
        let pulse_sample = pulse.next(0.0, 48_000.0, 0.5, PULSE);
        assert!(saw_sample.poly_mod_source_volts >= 0.0);
        assert!(pulse_sample.poly_mod_source_volts >= -0.5);
        assert!(
            (triangle_sample.mixer_negative_source_volts
                - triangle_sample.poly_mod_source_volts
                - TRIANGLE_POLY_MOD_REFERENCE_VOLTS)
                .abs()
                < 1.0e-6
        );
    }

    #[test]
    fn loaded_pulse_and_saw_reach_comparable_mixer_excursions() {
        let saw_peak_to_peak = SAW_UPPER_VOLTS[4] - SAW_LOWER_VOLTS[4];
        let pulse_peak_to_peak = (PULSE_UPPER_VOLTS - PULSE_LOWER_VOLTS) * PULSE_MIXER_CONDUCTANCE;
        let ratio = pulse_peak_to_peak / saw_peak_to_peak;
        assert!((0.95..1.05).contains(&ratio));
    }

    #[test]
    fn selected_waveforms_report_their_populated_mixer_conductances() {
        let mut oscillator = Vco::default();
        let saw = oscillator.next(0.0, 48_000.0, 0.5, SAW);
        let pulse = oscillator.next(0.0, 48_000.0, 0.5, PULSE);
        let all = oscillator.next(
            0.0,
            48_000.0,
            0.5,
            WaveSelection {
                saw: true,
                triangle: true,
                pulse: true,
            },
        );
        let disconnected = oscillator.next(0.0, 48_000.0, 0.5, WaveSelection::default());

        assert_eq!(saw.mixer_positive_source_conductance, 1.0);
        assert_eq!(saw.mixer_negative_source_conductance, 0.0);
        assert_eq!(pulse.mixer_positive_source_conductance, 0.0);
        assert_eq!(pulse.mixer_negative_source_conductance, 0.75);
        assert_eq!(all.mixer_positive_source_conductance, 1.0);
        assert_eq!(all.mixer_negative_source_conductance, 1.75);
        assert_eq!(all.poly_mod_source_conductance, 2.75);
        assert_eq!(disconnected.mixer_positive_source_conductance, 0.0);
        assert_eq!(disconnected.mixer_negative_source_conductance, 0.0);
        assert_eq!(disconnected.poly_mod_source_conductance, 0.0);
    }

    #[test]
    fn band_limited_saw_is_periodic_finite_and_bounded() {
        for increment in [0.001, 0.01, 0.10, 0.49] {
            let below_wrap = band_limited_saw(1.0 - 1.0e-6, increment);
            let at_wrap = band_limited_saw(0.0, increment);
            assert!((below_wrap - at_wrap).abs() < 1.0e-3);
            for index in 0..10_000 {
                let sample = band_limited_saw(index as f32 / 10_000.0, increment);
                assert!(sample.is_finite());
                assert!(sample.abs() <= 1.0);
            }
        }
    }
}
