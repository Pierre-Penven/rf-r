//! Antenna gain, beamwidth and figure of merit (G/T).

use crate::db::to_db;
use crate::fspl::wavelength_m;
use std::f64::consts::PI;

/// Empirical factor for the 3 dB beamwidth of a parabolic reflector, degrees.
pub const HPBW_FACTOR_DEG: f64 = 70.0;

/// Gain in dBi of an aperture antenna: `10·log10(η·4π·A / λ²)`.
///
/// `efficiency` is the aperture efficiency as a fraction (e.g. `0.65`).
#[must_use]
#[inline]
pub fn aperture_gain_dbi(area_m2: f64, freq_hz: f64, efficiency: f64) -> f64 {
    let wavelength = wavelength_m(freq_hz);
    to_db(efficiency * 4.0 * PI * area_m2 / (wavelength * wavelength))
}

/// Gain in dBi of a circular parabolic reflector: `10·log10(η·(π·D/λ)²)`.
///
/// ```
/// // 1.2 m dish at 12 GHz, 65 % efficiency ≈ 41.7 dBi
/// let gain = rf_r::antenna::parabolic_gain_dbi(1.2, 12e9, 0.65);
/// assert!((gain - 41.7).abs() < 0.1);
/// ```
#[must_use]
#[inline]
pub fn parabolic_gain_dbi(diameter_m: f64, freq_hz: f64, efficiency: f64) -> f64 {
    let electrical_size = PI * diameter_m / wavelength_m(freq_hz);
    to_db(efficiency * electrical_size * electrical_size)
}

/// Approximate 3 dB (half-power) beamwidth of a parabolic reflector, degrees:
/// `70·λ / D`.
#[must_use]
#[inline]
pub fn half_power_beamwidth_deg(diameter_m: f64, freq_hz: f64) -> f64 {
    HPBW_FACTOR_DEG * wavelength_m(freq_hz) / diameter_m
}

/// Receive figure of merit G/T in dB/K: `G - 10·log10(T_sys)`.
#[must_use]
#[inline]
pub fn g_over_t_db_k(gain_dbi: f64, system_temp_k: f64) -> f64 {
    gain_dbi - to_db(system_temp_k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn parabolic_gain_matches_aperture_gain_of_disc() {
        let diameter = 2.4;
        let area = PI * diameter * diameter / 4.0;
        assert_close(
            parabolic_gain_dbi(diameter, 14e9, 0.6),
            aperture_gain_dbi(area, 14e9, 0.6),
            1e-10,
        );
    }

    #[test]
    fn parabolic_gain_reference_value() {
        // 1.2 m, 12 GHz, η = 0.65 → 10·log10(0.65·(π·1.2/0.02498)²) = 41.70 dBi
        assert_close(parabolic_gain_dbi(1.2, 12e9, 0.65), 41.70, 0.01);
    }

    #[test]
    fn doubling_diameter_adds_6_db() {
        let delta = parabolic_gain_dbi(2.0, 12e9, 0.6) - parabolic_gain_dbi(1.0, 12e9, 0.6);
        assert_close(delta, 6.0206, 1e-4);
    }

    #[test]
    fn zero_efficiency_gives_negative_infinity() {
        assert!(parabolic_gain_dbi(1.0, 12e9, 0.0) < f64::MIN);
    }

    #[test]
    fn negative_efficiency_gives_nan() {
        assert!(aperture_gain_dbi(1.0, 12e9, -0.5).is_nan());
    }

    #[test]
    fn beamwidth_reference_value() {
        // 70 · 0.02498 / 1.2 ≈ 1.457°
        assert_close(half_power_beamwidth_deg(1.2, 12e9), 1.4573, 1e-3);
    }

    #[test]
    fn beamwidth_of_zero_diameter_is_infinite() {
        assert!(half_power_beamwidth_deg(0.0, 12e9).is_infinite());
    }

    #[test]
    fn g_over_t_reference_value() {
        assert_close(g_over_t_db_k(40.0, 100.0), 20.0, 1e-12);
    }

    #[test]
    fn g_over_t_with_zero_temperature_is_infinite() {
        assert!(g_over_t_db_k(40.0, 0.0) > f64::MAX);
    }
}
