//! Free-space path loss and wavelength.

use crate::constants::{FSPL_CONST_KM_GHZ_DB, FSPL_CONST_SI_DB, SPEED_OF_LIGHT};
use crate::db::amplitude_to_db;

/// Wavelength in metres for a frequency in Hz: `λ = c / f`.
#[must_use]
#[inline]
pub fn wavelength_m(freq_hz: f64) -> f64 {
    SPEED_OF_LIGHT / freq_hz
}

/// Frequency in Hz for a wavelength in metres: `f = c / λ`.
#[must_use]
#[inline]
pub fn frequency_hz(wavelength_m: f64) -> f64 {
    SPEED_OF_LIGHT / wavelength_m
}

/// Free-space path loss in dB: `20·log10(4π·d·f / c)`.
///
/// Zero distance gives `-inf`; negative distance or frequency gives `NaN`.
///
/// ```
/// // 1 km at 1 GHz ≈ 92.45 dB
/// let loss = rf_r::fspl::fspl_db(1_000.0, 1e9);
/// assert!((loss - 92.4478).abs() < 1e-3);
/// ```
#[must_use]
#[inline]
pub fn fspl_db(distance_m: f64, freq_hz: f64) -> f64 {
    amplitude_to_db(distance_m * freq_hz) + FSPL_CONST_SI_DB
}

/// Free-space path loss in dB with distance in km and frequency in GHz:
/// `92.45 + 20·log10(d_km) + 20·log10(f_GHz)`.
#[must_use]
#[inline]
pub fn fspl_db_km_ghz(distance_km: f64, freq_ghz: f64) -> f64 {
    amplitude_to_db(distance_km * freq_ghz) + FSPL_CONST_KM_GHZ_DB
}

/// Computes the free-space path loss for many distances at one frequency.
///
/// # Panics
///
/// Panics if `distances_m` and `out_db` have different lengths.
///
/// ```
/// let mut out = [0.0; 2];
/// rf_r::fspl::fspl_db_batch(&[1e3, 2e3], 1e9, &mut out);
/// assert!((out[1] - out[0] - 6.0206).abs() < 1e-4);
/// ```
pub fn fspl_db_batch(distances_m: &[f64], freq_hz: f64, out_db: &mut [f64]) {
    assert_eq!(
        distances_m.len(),
        out_db.len(),
        "distances_m and out_db must have equal length"
    );
    let frequency_term_db = amplitude_to_db(freq_hz) + FSPL_CONST_SI_DB;
    for (out, &distance) in out_db.iter_mut().zip(distances_m) {
        *out = amplitude_to_db(distance) + frequency_term_db;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;
    use std::f64::consts::PI;

    #[test]
    fn wavelength_and_frequency_round_trip() {
        assert_close(wavelength_m(SPEED_OF_LIGHT), 1.0, 0.0);
        assert_close(frequency_hz(wavelength_m(12e9)), 12e9, 1e-3);
    }

    #[test]
    fn wavelength_at_zero_frequency_is_infinite() {
        assert!(wavelength_m(0.0).is_infinite());
    }

    #[test]
    fn fspl_matches_definition() {
        let (distance, freq) = (35_786e3, 11.7e9);
        let reference = 20.0 * (4.0 * PI * distance * freq / SPEED_OF_LIGHT).log10();
        assert_close(fspl_db(distance, freq), reference, 1e-10);
    }

    #[test]
    fn fspl_km_ghz_matches_si_variant() {
        assert_close(
            fspl_db_km_ghz(35_786.0, 11.7),
            fspl_db(35_786e3, 11.7e9),
            1e-10,
        );
    }

    #[test]
    fn fspl_geo_ku_band_reference_value() {
        // Textbook value: ≈ 205.1 dB at 12 GHz over 35 786 km
        assert_close(fspl_db(35_786e3, 12e9), 205.1, 0.05);
    }

    #[test]
    fn fspl_doubling_distance_adds_6_db() {
        assert_close(fspl_db(2e6, 1e9) - fspl_db(1e6, 1e9), 6.0206, 1e-4);
    }

    #[test]
    fn fspl_at_zero_distance_is_negative_infinity() {
        assert!(fspl_db(0.0, 1e9) < f64::MIN);
    }

    #[test]
    fn fspl_with_negative_input_is_nan() {
        assert!(fspl_db(-1.0, 1e9).is_nan());
        assert!(fspl_db(1.0, -1e9).is_nan());
    }

    #[test]
    fn fspl_batch_matches_scalar() {
        let distances = [1e3, 1e5, 4e7];
        let mut out = [0.0; 3];
        fspl_db_batch(&distances, 20e9, &mut out);
        for (loss, &distance) in out.iter().zip(&distances) {
            assert_close(*loss, fspl_db(distance, 20e9), 1e-10);
        }
    }

    #[test]
    fn fspl_batch_accepts_empty_slices() {
        let mut out: [f64; 0] = [];
        fspl_db_batch(&[], 1e9, &mut out);
        assert_eq!(out.as_slice(), &[] as &[f64]);
    }

    #[test]
    #[should_panic(expected = "distances_m and out_db must have equal length")]
    fn fspl_batch_panics_on_length_mismatch() {
        let mut out = [0.0; 2];
        fspl_db_batch(&[1.0], 1e9, &mut out);
    }
}
