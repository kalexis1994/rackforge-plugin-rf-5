//! Bounded elementary functions for the portable real-time profile.
//!
//! The 4x reference path retains `libm`. The distributed WebAssembly component
//! uses these reduced-range polynomials on every host. Their error stays far
//! below the component/model uncertainty while keeping the shared portable
//! audio callback comfortably bounded.

const LN_2: f32 = core::f32::consts::LN_2;

#[inline(always)]
pub(crate) fn exp2(value: f32) -> f32 {
    if !value.is_finite() || !(-126.0..=127.0).contains(&value) {
        return libm::exp2f(value);
    }

    // Centre the residual on zero so a sixth-order exponential polynomial has
    // sub-ppm error across the complete reduced interval [-0.5, 0.5].
    let exponent = if value >= 0.0 {
        (value + 0.5) as i32
    } else {
        (value - 0.5) as i32
    };
    let fraction = value - exponent as f32;
    let polynomial = 1.0
        + fraction
            * (LN_2
                + fraction
                    * (0.240_226_5
                        + fraction
                            * (0.055_504_11
                                + fraction
                                    * (0.009_618_129
                                        + fraction
                                            * (0.001_333_355_8 + fraction * 0.000_154_035_3)))));
    let scale = f32::from_bits(((exponent + 127) as u32) << 23);
    scale * polynomial
}

#[inline(always)]
pub(crate) fn tpt_coefficient(normalized_cutoff: f32) -> f32 {
    let angle = core::f32::consts::PI * normalized_cutoff.clamp(0.0, 0.45);
    let squared = angle * angle;
    let sine = angle
        * (1.0
            + squared
                * (-1.0 / 6.0
                    + squared
                        * (1.0 / 120.0
                            + squared
                                * (-1.0 / 5_040.0
                                    + squared * (1.0 / 362_880.0 - squared / 39_916_800.0)))));
    let cosine = 1.0
        + squared
            * (-1.0 / 2.0
                + squared
                    * (1.0 / 24.0
                        + squared
                            * (-1.0 / 720.0
                                + squared
                                    * (1.0 / 40_320.0
                                        + squared
                                            * (-1.0 / 3_628_800.0 + squared / 479_001_600.0)))));
    // tan(x) / (1 + tan(x)) = sin(x) / (sin(x) + cos(x)).
    sine / (sine + cosine)
}

#[inline(always)]
pub(crate) fn ln(value: f32) -> f32 {
    if !value.is_finite() || value < f32::MIN_POSITIVE {
        return libm::logf(value);
    }
    let bits = value.to_bits();
    let exponent = ((bits >> 23) & 0xff) as i32 - 127;
    let mantissa = f32::from_bits((bits & 0x007f_ffff) | 0x3f80_0000);
    let reduced = (mantissa - 1.0) / (mantissa + 1.0);
    let squared = reduced * reduced;
    let series = reduced
        * (1.0
            + squared
                * (1.0 / 3.0
                    + squared
                        * (1.0 / 5.0
                            + squared * (1.0 / 7.0 + squared * (1.0 / 9.0 + squared / 11.0)))));
    exponent as f32 * LN_2 + 2.0 * series
}

#[inline(always)]
pub(crate) fn exp(value: f32) -> f32 {
    exp2(value / LN_2)
}

#[inline(always)]
pub(crate) fn tanh(value: f32) -> f32 {
    if !value.is_finite() {
        return libm::tanhf(value);
    }
    let magnitude = value.abs();
    if magnitude >= 10.0 {
        return value.signum();
    }
    if magnitude <= 0.25 {
        // Avoid cancellation in exp(2x)-1 around the OTA's small-signal
        // region, where conductance-loading tests depend on relative gain.
        let squared = value * value;
        return value
            * (1.0
                + squared
                    * (-1.0 / 3.0
                        + squared
                            * (2.0 / 15.0
                                + squared * (-17.0 / 315.0 + squared * 62.0 / 2_835.0))));
    }
    let exponential = exp(2.0 * magnitude);
    value.signum() * (exponential - 1.0) / (exponential + 1.0)
}

#[inline(always)]
#[cfg(test)]
pub(crate) fn sixth_root(value: f32) -> f32 {
    exp2(ln(value) / (6.0 * LN_2))
}

/// Evaluate `(1 + excess)^(-1/6)` on the only interval used by the
/// sixth-order OTA limiter.
///
/// Reducing the domain to `[0, 1]` avoids a logarithm/exponential pair in
/// every voice and master VCA. The degree-seven minimax-style fit stays below
/// 5e-7 relative error in `f32`, which is tighter than the existing bounded
/// elementary-function contract.
#[inline(always)]
pub(crate) fn inverse_sixth_root_one_plus(excess: f32) -> f32 {
    let x = excess.clamp(0.0, 1.0);
    0.999_999_9
        + x * (-0.166_656_51
            + x * (0.097_032_495
                + x * (-0.068_691_954
                    + x * (0.049_057_283
                        + x * (-0.029_592_155 + x * (0.012_096_441 + x * -0.002_346_836_7))))))
}

#[inline(always)]
pub(crate) fn inverse_sixteenth_root_one_plus(excess: f32) -> f32 {
    let x = excess.clamp(0.0, 1.0);
    0.999_999_76
        + x * (-0.062_485_214
            + x * (0.032_987_72
                + x * (-0.021_489_795
                    + x * (0.013_101_168 + x * (-0.005_720_104 + x * 0.001_209_935_2)))))
}

#[inline(always)]
pub(crate) fn inverse_thirty_second_root_one_plus(excess: f32) -> f32 {
    let x = excess.clamp(0.0, 1.0);
    0.999_999_9
        + x * (-0.031_243_07
            + x * (0.016_012_35
                + x * (-0.010_283_758
                    + x * (0.006_220_442_2 + x * (-0.002_704_453_7 + x * 0.000_570_751_7)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp2_tracks_libm_across_every_filter_and_pitch_octave() {
        let mut maximum_relative_error = 0.0_f32;
        for step in -24_000..=24_000 {
            let value = step as f32 / 2_000.0;
            let reference = libm::exp2f(value);
            let relative_error = (exp2(value) - reference).abs() / reference;
            maximum_relative_error = maximum_relative_error.max(relative_error);
        }
        assert!(maximum_relative_error <= 4.0e-7, "{maximum_relative_error}");
    }

    #[test]
    fn tpt_coefficient_tracks_libm_across_the_supported_cutoff_range() {
        let mut maximum_error = 0.0_f32;
        for step in 0..=45_000 {
            let normalized = step as f32 / 100_000.0;
            let g = libm::tanf(core::f32::consts::PI * normalized);
            let reference = g / (1.0 + g);
            maximum_error = maximum_error.max((tpt_coefficient(normalized) - reference).abs());
        }
        assert!(maximum_error <= 3.0e-7, "{maximum_error}");
    }

    #[test]
    fn logarithm_tracks_libm_across_the_transistor_solver_domain() {
        let mut maximum_absolute_error = 0.0_f32;
        for step in -20_000..=20_000 {
            let value = exp2(step as f32 / 1_000.0);
            maximum_absolute_error =
                maximum_absolute_error.max((ln(value) - libm::logf(value)).abs());
        }
        assert!(maximum_absolute_error <= 1.0e-6, "{maximum_absolute_error}");
    }

    #[test]
    fn tanh_tracks_libm_across_the_ota_input_domain() {
        let mut maximum_error = 0.0_f32;
        for step in -20_000..=20_000 {
            let value = step as f32 / 1_000.0;
            maximum_error = maximum_error.max((tanh(value) - libm::tanhf(value)).abs());
        }
        assert!(maximum_error <= 5.0e-7, "{maximum_error}");
    }

    #[test]
    fn sixth_root_tracks_libm_across_both_vca_branches() {
        let mut maximum_relative_error = 0.0_f32;
        for step in 0..=20_000 {
            let value = exp2(step as f32 / 1_000.0);
            let reference = libm::powf(value, 1.0 / 6.0);
            maximum_relative_error =
                maximum_relative_error.max((sixth_root(value) - reference).abs() / reference);
        }
        assert!(maximum_relative_error <= 5.0e-7, "{maximum_relative_error}");
    }

    #[test]
    fn reduced_vca_inverse_root_tracks_the_exact_curve() {
        let mut maximum_relative_error = 0.0_f32;
        for step in 0..=20_000 {
            let excess = step as f32 / 20_000.0;
            let reference = 1.0 / libm::powf(1.0 + excess, 1.0 / 6.0);
            maximum_relative_error = maximum_relative_error
                .max((inverse_sixth_root_one_plus(excess) - reference).abs() / reference);
        }
        assert!(maximum_relative_error <= 5.0e-7, "{maximum_relative_error}");
    }

    #[test]
    fn reduced_filter_inverse_roots_track_the_exact_curves() {
        let mut maximum_sixteenth_error = 0.0_f32;
        let mut maximum_thirty_second_error = 0.0_f32;
        for step in 0..=20_000 {
            let excess = step as f32 / 20_000.0;
            let exact_sixteenth = 1.0 / libm::powf(1.0 + excess, 1.0 / 16.0);
            let exact_thirty_second = 1.0 / libm::powf(1.0 + excess, 1.0 / 32.0);
            maximum_sixteenth_error = maximum_sixteenth_error.max(
                (inverse_sixteenth_root_one_plus(excess) - exact_sixteenth).abs() / exact_sixteenth,
            );
            maximum_thirty_second_error = maximum_thirty_second_error.max(
                (inverse_thirty_second_root_one_plus(excess) - exact_thirty_second).abs()
                    / exact_thirty_second,
            );
        }
        assert!(
            maximum_sixteenth_error <= 5.0e-7,
            "{maximum_sixteenth_error}"
        );
        assert!(
            maximum_thirty_second_error <= 5.0e-7,
            "{maximum_thirty_second_error}"
        );
    }
}
