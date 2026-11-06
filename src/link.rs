//! Link budget: EIRP, C/N0, C/N, Eb/N0 and combination of C/N contributions.

use crate::constants::BOLTZMANN_DBW;
use crate::db::{DB_PER_NEPER, from_db, to_db};

/// One-way radio link (uplink or downlink).
///
/// ```
/// use rf_r::link::Link;
///
/// let downlink = Link {
///     eirp_dbw: 50.0,
///     path_loss_db: 205.0,
///     extra_losses_db: 1.0,
///     g_over_t_db_k: 18.0,
/// };
/// assert!((downlink.cn0_dbhz() - 90.599).abs() < 1e-3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Link {
    /// Transmit EIRP, dBW.
    pub eirp_dbw: f64,
    /// Free-space path loss, dB (positive).
    pub path_loss_db: f64,
    /// Additional losses (atmosphere, rain, pointing, polarisation), dB (positive).
    pub extra_losses_db: f64,
    /// Receive figure of merit G/T, dB/K.
    pub g_over_t_db_k: f64,
}

impl Link {
    /// Carrier-to-noise-density ratio, dB·Hz. See [`cn0_dbhz`].
    #[must_use]
    #[inline]
    pub fn cn0_dbhz(&self) -> f64 {
        cn0_dbhz(
            self.eirp_dbw,
            self.path_loss_db + self.extra_losses_db,
            self.g_over_t_db_k,
        )
    }

    /// Carrier-to-noise ratio in `bandwidth_hz`, dB.
    #[must_use]
    #[inline]
    pub fn cn_db(&self, bandwidth_hz: f64) -> f64 {
        cn_db(self.cn0_dbhz(), bandwidth_hz)
    }

    /// Energy-per-bit to noise-density ratio at `bit_rate_bps`, dB.
    #[must_use]
    #[inline]
    pub fn ebn0_db(&self, bit_rate_bps: f64) -> f64 {
        ebn0_db(self.cn0_dbhz(), bit_rate_bps)
    }
}

/// Effective isotropic radiated power, dBW: `P + G - L_feed`.
#[must_use]
#[inline]
pub fn eirp_dbw(tx_power_dbw: f64, tx_gain_dbi: f64, feeder_loss_db: f64) -> f64 {
    tx_power_dbw + tx_gain_dbi - feeder_loss_db
}

/// Carrier-to-noise-density ratio, dB·Hz: `EIRP - L_total + G/T - k`.
///
/// `total_loss_db` is the sum of path loss and all additional losses.
#[must_use]
#[inline]
pub fn cn0_dbhz(eirp_dbw: f64, total_loss_db: f64, g_over_t_db_k: f64) -> f64 {
    eirp_dbw - total_loss_db + g_over_t_db_k - BOLTZMANN_DBW
}

/// Carrier-to-noise ratio, dB: `C/N0 - 10·log10(B)`.
#[must_use]
#[inline]
pub fn cn_db(cn0_dbhz: f64, bandwidth_hz: f64) -> f64 {
    cn0_dbhz - to_db(bandwidth_hz)
}

/// Energy-per-bit to noise-density ratio, dB: `C/N0 - 10·log10(Rb)`.
#[must_use]
#[inline]
pub fn ebn0_db(cn0_dbhz: f64, bit_rate_bps: f64) -> f64 {
    cn0_dbhz - to_db(bit_rate_bps)
}

/// Link margin, dB: achieved minus required.
#[must_use]
#[inline]
pub fn margin_db(achieved_db: f64, required_db: f64) -> f64 {
    achieved_db - required_db
}

/// Combines two carrier-to-noise (or C/I, C/IM) ratios in dB:
/// `1/(C/N)_tot = 1/(C/N)_1 + 1/(C/N)_2`.
///
/// `+inf` (no impairment) is the neutral element.
///
/// ```
/// // Two equal 20 dB contributions give 16.99 dB
/// let total = rf_r::link::combine_cn2_db(20.0, 20.0);
/// assert!((total - 16.9897).abs() < 1e-4);
/// ```
#[must_use]
#[inline]
pub fn combine_cn2_db(a_db: f64, b_db: f64) -> f64 {
    let (low, high) = if a_db <= b_db {
        (a_db, b_db)
    } else {
        (b_db, a_db)
    };
    if high == f64::INFINITY {
        return low;
    }
    low - from_db(low - high).ln_1p() * DB_PER_NEPER
}

/// Combines any number of carrier-to-noise (or C/I, C/IM) ratios in dB:
/// `1/(C/N)_tot = Σ 1/(C/N)_i`.
///
/// Stable (min-shifted). Returns `+inf` for an empty slice and `NaN` if any
/// element is `NaN`.
///
/// ```
/// // Uplink 25 dB, downlink 18 dB, C/I 22 dB, C/IM 24 dB
/// let total = rf_r::link::combine_cn_db(&[25.0, 18.0, 22.0, 24.0]);
/// assert!((total - 15.331).abs() < 1e-3);
/// ```
#[must_use]
pub fn combine_cn_db(values_db: &[f64]) -> f64 {
    let mut min = f64::INFINITY;
    for &v in values_db {
        if v.is_nan() {
            return f64::NAN;
        }
        min = min.min(v);
    }
    if !min.is_finite() {
        return min;
    }
    let shifted_sum: f64 = values_db.iter().map(|&v| from_db(min - v)).sum();
    min - to_db(shifted_sum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    fn reference_combine(values_db: &[f64]) -> f64 {
        -10.0
            * values_db
                .iter()
                .map(|v| 10f64.powf(-v / 10.0))
                .sum::<f64>()
                .log10()
    }

    #[test]
    fn eirp_adds_power_and_gain_and_subtracts_loss() {
        assert_close(eirp_dbw(20.0, 45.0, 2.0), 63.0, 0.0);
    }

    #[test]
    fn cn0_reference_value() {
        // 50 - 206 + 18 + 228.6 = 90.6
        assert_close(cn0_dbhz(50.0, 206.0, 18.0), 90.599_167, 1e-6);
    }

    #[test]
    fn cn_subtracts_bandwidth() {
        assert_close(cn_db(90.0, 1e6), 30.0, 1e-12);
    }

    #[test]
    fn cn_with_zero_bandwidth_is_infinite() {
        assert!(cn_db(90.0, 0.0) > f64::MAX);
    }

    #[test]
    fn cn_with_negative_bandwidth_is_nan() {
        assert!(cn_db(90.0, -1.0).is_nan());
    }

    #[test]
    fn ebn0_subtracts_bit_rate() {
        assert_close(ebn0_db(80.0, 10e6), 10.0, 1e-12);
    }

    #[test]
    fn margin_is_achieved_minus_required() {
        assert_close(margin_db(12.0, 9.5), 2.5, 0.0);
        assert_close(margin_db(9.0, 9.5), -0.5, 0.0);
    }

    #[test]
    fn link_struct_matches_free_functions() {
        let link = Link {
            eirp_dbw: 52.0,
            path_loss_db: 205.5,
            extra_losses_db: 0.7,
            g_over_t_db_k: 19.0,
        };
        let expected_cn0 = cn0_dbhz(52.0, 206.2, 19.0);
        assert_close(link.cn0_dbhz(), expected_cn0, 1e-12);
        assert_close(link.cn_db(36e6), cn_db(expected_cn0, 36e6), 1e-12);
        assert_close(link.ebn0_db(50e6), ebn0_db(expected_cn0, 50e6), 1e-12);
    }

    #[test]
    fn combine_two_equal_values_loses_3_db() {
        assert_close(
            combine_cn2_db(20.0, 20.0),
            20.0 - 3.010_299_956_639_812,
            1e-12,
        );
    }

    #[test]
    fn combine_two_matches_reference_and_is_commutative() {
        let expected = reference_combine(&[15.0, 22.0]);
        assert_close(combine_cn2_db(15.0, 22.0), expected, 1e-12);
        assert_close(combine_cn2_db(22.0, 15.0), expected, 1e-12);
    }

    #[test]
    fn combine_two_with_infinity_is_identity() {
        assert_close(combine_cn2_db(12.0, f64::INFINITY), 12.0, 0.0);
        assert_close(combine_cn2_db(f64::INFINITY, 12.0), 12.0, 0.0);
    }

    #[test]
    fn combine_two_with_nan_is_nan() {
        assert!(combine_cn2_db(f64::NAN, 10.0).is_nan());
        assert!(combine_cn2_db(10.0, f64::NAN).is_nan());
    }

    #[test]
    fn combine_many_matches_reference() {
        let values = [25.0, 18.0, 22.0, 24.0];
        assert_close(combine_cn_db(&values), reference_combine(&values), 1e-12);
    }

    #[test]
    fn combine_many_matches_pairwise() {
        let expected = combine_cn2_db(combine_cn2_db(10.0, 14.0), 30.0);
        assert_close(combine_cn_db(&[10.0, 14.0, 30.0]), expected, 1e-12);
    }

    #[test]
    fn combine_many_of_empty_slice_is_infinity() {
        assert!(combine_cn_db(&[]) > f64::MAX);
    }

    #[test]
    fn combine_many_of_single_value_is_that_value() {
        assert_close(combine_cn_db(&[13.0]), 13.0, 1e-12);
    }

    #[test]
    fn combine_many_with_negative_infinity_is_negative_infinity() {
        assert!(combine_cn_db(&[10.0, f64::NEG_INFINITY]) < f64::MIN);
    }

    #[test]
    fn combine_many_with_nan_is_nan() {
        assert!(combine_cn_db(&[10.0, f64::NAN]).is_nan());
    }

    #[test]
    fn combine_many_does_not_underflow_for_huge_ratios() {
        // Naive 10^(-4000/10) underflows to zero
        assert_close(combine_cn_db(&[4000.0, 4000.0]), 4000.0 - 3.0103, 1e-4);
    }

    #[test]
    fn combined_cn_is_never_better_than_worst_contribution() {
        let values = [8.0, 30.0, 40.0];
        let total = combine_cn_db(&values);
        assert!(total < 8.0 && total > 7.9);
    }
}
