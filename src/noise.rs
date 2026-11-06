//! Noise temperature, noise figure and noise power.

use crate::constants::{BOLTZMANN_DBW, T0_K};
use crate::db::{from_db, to_db};

/// One stage of a receive chain, used by [`cascade_noise_temperature_k`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseStage {
    /// Stage gain in dB (negative for a lossy element).
    pub gain_db: f64,
    /// Stage equivalent input noise temperature, K.
    pub noise_temp_k: f64,
}

/// Equivalent noise temperature from a noise figure: `T = T0·(F - 1)`.
///
/// ```
/// // NF = 1 dB ≈ 75 K
/// let t = rf_r::noise::noise_temperature_k(1.0);
/// assert!((t - 75.09).abs() < 0.01);
/// ```
#[must_use]
#[inline]
pub fn noise_temperature_k(noise_figure_db: f64) -> f64 {
    T0_K * (from_db(noise_figure_db) - 1.0)
}

/// Noise figure from an equivalent noise temperature: `NF = 10·log10(1 + T/T0)`.
///
/// Negative temperatures below `-T0` give `NaN`.
#[must_use]
#[inline]
pub fn noise_figure_db(noise_temp_k: f64) -> f64 {
    to_db(1.0 + noise_temp_k / T0_K)
}

/// Noise temperature added by a passive loss at physical temperature
/// `physical_temp_k`: `T = (L - 1)·T_phys`, referred to the loss input.
#[must_use]
#[inline]
pub fn lossy_line_noise_temperature_k(loss_db: f64, physical_temp_k: f64) -> f64 {
    (from_db(loss_db) - 1.0) * physical_temp_k
}

/// Noise power spectral density in dBW/Hz: `10·log10(k·T)`.
#[must_use]
#[inline]
pub fn noise_density_dbw_hz(noise_temp_k: f64) -> f64 {
    BOLTZMANN_DBW + to_db(noise_temp_k)
}

/// Noise power in dBW: `10·log10(k·T·B)`.
///
/// ```
/// // 290 K in 1 Hz ≈ -204 dBW (-174 dBm/Hz)
/// let n = rf_r::noise::noise_power_dbw(290.0, 1.0);
/// assert!((n + 203.98).abs() < 0.01);
/// ```
#[must_use]
#[inline]
pub fn noise_power_dbw(noise_temp_k: f64, bandwidth_hz: f64) -> f64 {
    BOLTZMANN_DBW + to_db(noise_temp_k * bandwidth_hz)
}

/// Friis cascade: `T = T1 + T2/G1 + T3/(G1·G2) + …`, referred to the first stage input.
///
/// Returns `0.0` for an empty chain.
#[must_use]
pub fn cascade_noise_temperature_k(stages: &[NoiseStage]) -> f64 {
    let mut total_temp_k = 0.0;
    let mut preceding_gain = 1.0;
    for stage in stages {
        total_temp_k += stage.noise_temp_k / preceding_gain;
        preceding_gain *= from_db(stage.gain_db);
    }
    total_temp_k
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn noise_temperature_of_zero_db_figure_is_zero() {
        assert_close(noise_temperature_k(0.0), 0.0, 1e-12);
    }

    #[test]
    fn noise_temperature_of_3_db_figure_is_about_t0() {
        assert_close(noise_temperature_k(3.010_299_956_639_812), T0_K, 1e-9);
    }

    #[test]
    fn noise_figure_and_temperature_round_trip() {
        for &nf in &[0.3, 1.0, 2.5, 10.0] {
            assert_close(noise_figure_db(noise_temperature_k(nf)), nf, 1e-12);
        }
    }

    #[test]
    fn noise_figure_below_minus_t0_is_nan() {
        assert!(noise_figure_db(-2.0 * T0_K).is_nan());
    }

    #[test]
    fn noise_figure_at_minus_t0_is_negative_infinity() {
        assert!(noise_figure_db(-T0_K) < f64::MIN);
    }

    #[test]
    fn lossy_line_temperature_reference_value() {
        // 3 dB loss at 290 K adds ≈ 288.6 K
        assert_close(lossy_line_noise_temperature_k(3.0, 290.0), 288.626, 1e-3);
        assert_close(lossy_line_noise_temperature_k(0.0, 290.0), 0.0, 1e-12);
    }

    #[test]
    fn noise_density_at_t0_is_minus_174_dbm_hz() {
        assert_close(noise_density_dbw_hz(T0_K) + 30.0, -173.975, 1e-3);
    }

    #[test]
    fn noise_power_scales_with_bandwidth() {
        let delta = noise_power_dbw(100.0, 36e6) - noise_density_dbw_hz(100.0);
        assert_close(delta, 10.0 * 36e6_f64.log10(), 1e-10);
    }

    #[test]
    fn noise_power_at_zero_temperature_is_negative_infinity() {
        assert!(noise_power_dbw(0.0, 1e6) < f64::MIN);
    }

    #[test]
    fn noise_power_with_negative_bandwidth_is_nan() {
        assert!(noise_power_dbw(290.0, -1.0).is_nan());
    }

    #[test]
    fn cascade_of_empty_chain_is_zero() {
        assert_close(cascade_noise_temperature_k(&[]), 0.0, 0.0);
    }

    #[test]
    fn cascade_of_single_stage_is_its_temperature() {
        let lna = NoiseStage {
            gain_db: 60.0,
            noise_temp_k: 50.0,
        };
        assert_close(cascade_noise_temperature_k(&[lna]), 50.0, 0.0);
    }

    #[test]
    fn cascade_follows_friis_formula() {
        let stages = [
            NoiseStage {
                gain_db: 20.0,
                noise_temp_k: 50.0,
            },
            NoiseStage {
                gain_db: 10.0,
                noise_temp_k: 500.0,
            },
            NoiseStage {
                gain_db: 0.0,
                noise_temp_k: 1000.0,
            },
        ];
        // 50 + 500/100 + 1000/1000
        assert_close(cascade_noise_temperature_k(&stages), 56.0, 1e-9);
    }

    #[test]
    fn cascade_with_lossy_first_stage_amplifies_later_noise() {
        let loss_db = 1.0;
        let stages = [
            NoiseStage {
                gain_db: -loss_db,
                noise_temp_k: lossy_line_noise_temperature_k(loss_db, 290.0),
            },
            NoiseStage {
                gain_db: 60.0,
                noise_temp_k: 50.0,
            },
        ];
        let expected = lossy_line_noise_temperature_k(loss_db, 290.0) + 50.0 * from_db(loss_db);
        assert_close(cascade_noise_temperature_k(&stages), expected, 1e-9);
    }
}
