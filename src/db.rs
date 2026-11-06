//! Decibel conversions and power summation in the log domain.
//!
//! Conversions use `ln`/`exp` with precomputed factors, which is faster than
//! `log10`/`powf` while matching them to within a few ULPs.

use std::f64::consts::LN_10;

/// `10 / ln(10)`: converts a natural log of a power ratio to dB.
pub(crate) const DB_PER_NEPER: f64 = 10.0 / LN_10;
/// `ln(10) / 10`: converts dB to the natural log of a power ratio.
const NEPER_PER_DB: f64 = LN_10 / 10.0;
/// Offset between dBm and dBW.
const DBM_PER_DBW_OFFSET: f64 = 30.0;

/// Converts a linear power ratio to dB: `10·log10(x)`.
///
/// `0.0` gives `-inf`; negative input gives `NaN`.
///
/// ```
/// assert!((rf_r::db::to_db(100.0) - 20.0).abs() < 1e-12);
/// ```
#[must_use]
#[inline]
pub fn to_db(linear: f64) -> f64 {
    linear.ln() * DB_PER_NEPER
}

/// Converts dB to a linear power ratio: `10^(dB/10)`.
///
/// ```
/// assert!((rf_r::db::from_db(30.0) - 1000.0).abs() < 1e-9);
/// ```
#[must_use]
#[inline]
pub fn from_db(db: f64) -> f64 {
    (db * NEPER_PER_DB).exp()
}

/// Converts a linear amplitude (voltage, field) ratio to dB: `20·log10(x)`.
#[must_use]
#[inline]
pub fn amplitude_to_db(linear: f64) -> f64 {
    2.0 * to_db(linear)
}

/// Converts dB to a linear amplitude ratio: `10^(dB/20)`.
#[must_use]
#[inline]
pub fn amplitude_from_db(db: f64) -> f64 {
    from_db(0.5 * db)
}

/// Converts watts to dBW.
#[must_use]
#[inline]
pub fn watts_to_dbw(watts: f64) -> f64 {
    to_db(watts)
}

/// Converts dBW to watts.
#[must_use]
#[inline]
pub fn dbw_to_watts(dbw: f64) -> f64 {
    from_db(dbw)
}

/// Converts watts to dBm.
#[must_use]
#[inline]
pub fn watts_to_dbm(watts: f64) -> f64 {
    dbw_to_dbm(to_db(watts))
}

/// Converts dBm to watts.
#[must_use]
#[inline]
pub fn dbm_to_watts(dbm: f64) -> f64 {
    from_db(dbm_to_dbw(dbm))
}

/// Converts dBW to dBm.
#[must_use]
#[inline]
pub fn dbw_to_dbm(dbw: f64) -> f64 {
    dbw + DBM_PER_DBW_OFFSET
}

/// Converts dBm to dBW.
#[must_use]
#[inline]
pub fn dbm_to_dbw(dbm: f64) -> f64 {
    dbm - DBM_PER_DBW_OFFSET
}

/// Power sum of two dB values: `10·log10(10^(a/10) + 10^(b/10))`.
///
/// Computed as `max + 10·log10(1 + 10^((min-max)/10))`, which never overflows
/// and stays accurate for widely separated inputs. `-inf` is the neutral element.
///
/// ```
/// // Two equal powers add 3.01 dB
/// assert!((rf_r::db::log_sum(10.0, 10.0) - 13.0103).abs() < 1e-4);
/// ```
#[must_use]
#[inline]
pub fn log_sum(a_db: f64, b_db: f64) -> f64 {
    let (high, low) = if a_db >= b_db {
        (a_db, b_db)
    } else {
        (b_db, a_db)
    };
    if low == f64::NEG_INFINITY {
        return high;
    }
    high + from_db(low - high).ln_1p() * DB_PER_NEPER
}

/// Power sum of many dB values (stable, max-shifted).
///
/// Returns `-inf` for an empty slice and `NaN` if any element is `NaN`.
///
/// ```
/// let total = rf_r::db::log_sum_all(&[0.0, 0.0, 0.0, 0.0]);
/// assert!((total - 6.0206).abs() < 1e-4);
/// ```
#[must_use]
pub fn log_sum_all(values_db: &[f64]) -> f64 {
    let Some(max) = nan_aware_max(values_db) else {
        return f64::NAN;
    };
    if !max.is_finite() {
        return max;
    }
    let shifted_sum: f64 = values_db.iter().map(|&v| from_db(v - max)).sum();
    max + to_db(shifted_sum)
}

/// Power difference: `10·log10(10^(a/10) - 10^(b/10))`.
///
/// Removes a known contribution (e.g. noise) from a measured total.
/// Returns `-inf` when `a == b` and `NaN` when `b > a`.
///
/// ```
/// let signal = rf_r::db::log_diff(13.0103, 10.0);
/// assert!((signal - 10.0).abs() < 1e-3);
/// ```
#[must_use]
#[inline]
pub fn log_diff(a_db: f64, b_db: f64) -> f64 {
    if b_db == f64::NEG_INFINITY {
        return a_db;
    }
    a_db + (-from_db(b_db - a_db)).ln_1p() * DB_PER_NEPER
}

/// Converts every element from linear power ratio to dB in place.
pub fn to_db_in_place(values: &mut [f64]) {
    for v in values {
        *v = to_db(*v);
    }
}

/// Converts every element from dB to linear power ratio in place.
pub fn from_db_in_place(values: &mut [f64]) {
    for v in values {
        *v = from_db(*v);
    }
}

/// Maximum of `values`, `-inf` when empty, `None` when any element is `NaN`.
pub(crate) fn nan_aware_max(values: &[f64]) -> Option<f64> {
    values.iter().try_fold(f64::NEG_INFINITY, |max, &v| {
        if v.is_nan() { None } else { Some(max.max(v)) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn to_db_then_from_db_round_trips() {
        for &x in &[1e-20, 1e-3, 0.5, 1.0, 2.0, 1e6, 1e30] {
            assert_close(from_db(to_db(x)) / x, 1.0, 1e-12);
        }
    }

    #[test]
    fn to_db_matches_log10_reference() {
        for &x in &[1e-12, 0.123, 7.0, 12_345.678] {
            assert_close(to_db(x), 10.0 * x.log10(), 1e-12);
        }
    }

    #[test]
    fn to_db_known_values() {
        assert_close(to_db(1.0), 0.0, 0.0);
        assert_close(to_db(10.0), 10.0, 1e-12);
        assert_close(to_db(2.0), 3.010_299_956_639_812, 1e-12);
    }

    #[test]
    fn to_db_of_zero_is_negative_infinity() {
        assert!(to_db(0.0).is_infinite() && to_db(0.0) < 0.0);
    }

    #[test]
    fn to_db_of_negative_or_nan_is_nan() {
        assert!(to_db(-1.0).is_nan());
        assert!(to_db(f64::NAN).is_nan());
    }

    #[test]
    fn from_db_of_infinities() {
        assert_close(from_db(f64::NEG_INFINITY), 0.0, 0.0);
        assert!(from_db(f64::INFINITY).is_infinite());
    }

    #[test]
    fn amplitude_conversions_use_20_log() {
        assert_close(amplitude_to_db(10.0), 20.0, 1e-12);
        assert_close(amplitude_from_db(20.0), 10.0, 1e-12);
        assert_close(amplitude_from_db(amplitude_to_db(3.7)), 3.7, 1e-12);
    }

    #[test]
    fn power_unit_conversions() {
        assert_close(watts_to_dbm(1.0), 30.0, 1e-12);
        assert_close(dbm_to_watts(0.0), 1e-3, 1e-15);
        assert_close(watts_to_dbw(100.0), 20.0, 1e-12);
        assert_close(dbw_to_watts(-30.0), 1e-3, 1e-15);
        assert_close(dbw_to_dbm(0.0), 30.0, 0.0);
        assert_close(dbm_to_dbw(30.0), 0.0, 0.0);
    }

    #[test]
    fn log_sum_matches_linear_sum() {
        assert_close(log_sum(0.0, 0.0), to_db(2.0), 1e-12);
        assert_close(log_sum(10.0, 0.0), to_db(11.0), 1e-12);
    }

    #[test]
    fn log_sum_is_commutative() {
        assert_close(log_sum(0.0, 10.0), log_sum(10.0, 0.0), 0.0);
    }

    #[test]
    fn log_sum_does_not_overflow_for_huge_values() {
        assert_close(log_sum(4000.0, 4000.0), 4000.0 + to_db(2.0), 1e-9);
    }

    #[test]
    fn log_sum_is_accurate_for_widely_separated_values() {
        assert_close(log_sum(100.0, -300.0), 100.0, 1e-12);
    }

    #[test]
    fn log_sum_with_negative_infinity_is_identity() {
        assert_close(log_sum(5.0, f64::NEG_INFINITY), 5.0, 0.0);
        assert_close(log_sum(f64::NEG_INFINITY, 5.0), 5.0, 0.0);
        assert!(log_sum(f64::NEG_INFINITY, f64::NEG_INFINITY) < f64::MIN);
    }

    #[test]
    fn log_sum_propagates_nan() {
        assert!(log_sum(f64::NAN, 0.0).is_nan());
        assert!(log_sum(0.0, f64::NAN).is_nan());
    }

    #[test]
    fn log_sum_all_of_empty_slice_is_negative_infinity() {
        assert!(log_sum_all(&[]) < f64::MIN);
    }

    #[test]
    fn log_sum_all_of_single_value_is_that_value() {
        assert_close(log_sum_all(&[7.0]), 7.0, 1e-12);
    }

    #[test]
    fn log_sum_all_matches_pairwise_log_sum() {
        let expected = log_sum(log_sum(1.0, 2.0), 3.0);
        assert_close(log_sum_all(&[1.0, 2.0, 3.0]), expected, 1e-12);
    }

    #[test]
    fn log_sum_all_does_not_overflow_for_huge_values() {
        assert_close(log_sum_all(&[1000.0, 1000.0]), 1000.0 + to_db(2.0), 1e-9);
    }

    #[test]
    fn log_sum_all_with_positive_infinity_is_infinity() {
        assert!(log_sum_all(&[f64::INFINITY, 0.0]) > f64::MAX);
    }

    #[test]
    fn log_sum_all_with_nan_is_nan() {
        assert!(log_sum_all(&[f64::NAN]).is_nan());
        assert!(log_sum_all(&[1.0, f64::NAN, 2.0]).is_nan());
    }

    #[test]
    fn log_diff_inverts_log_sum() {
        assert_close(log_diff(log_sum(3.0, 1.0), 1.0), 3.0, 1e-12);
    }

    #[test]
    fn log_diff_of_equal_values_is_negative_infinity() {
        assert!(log_diff(3.0, 3.0) < f64::MIN);
    }

    #[test]
    fn log_diff_with_larger_subtrahend_is_nan() {
        assert!(log_diff(3.0, 4.0).is_nan());
    }

    #[test]
    fn log_diff_of_negative_infinity_is_identity() {
        assert_close(log_diff(3.0, f64::NEG_INFINITY), 3.0, 0.0);
    }

    #[test]
    fn in_place_conversions_apply_to_every_element() {
        let mut values = [1.0, 10.0, 100.0];
        to_db_in_place(&mut values);
        for (v, expected) in values.iter().zip([0.0, 10.0, 20.0]) {
            assert_close(*v, expected, 1e-12);
        }
        from_db_in_place(&mut values);
        for (v, expected) in values.iter().zip([1.0, 10.0, 100.0]) {
            assert_close(*v, expected, 1e-12);
        }
    }

    #[test]
    fn in_place_conversions_accept_empty_slice() {
        let mut empty: [f64; 0] = [];
        to_db_in_place(&mut empty);
        from_db_in_place(&mut empty);
        assert_eq!(empty.as_slice(), &[] as &[f64]);
    }

    #[test]
    fn nan_aware_max_cases() {
        assert_eq!(nan_aware_max(&[]), Some(f64::NEG_INFINITY));
        assert_eq!(nan_aware_max(&[1.0, 3.0, 2.0]), Some(3.0));
        assert_eq!(nan_aware_max(&[1.0, f64::NAN]), None);
    }
}
