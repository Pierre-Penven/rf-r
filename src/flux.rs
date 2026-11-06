//! Power flux density (PFD) and related conversions.

use crate::constants::FOUR_PI_DB;
use crate::db::{amplitude_to_db, to_db};
use crate::fspl::wavelength_m;

/// Spreading loss of an isotropic sphere, dB m²: `10·log10(4π·d²)`.
#[must_use]
#[inline]
pub fn spreading_loss_db(distance_m: f64) -> f64 {
    FOUR_PI_DB + amplitude_to_db(distance_m)
}

/// Power flux density, dBW/m²: `EIRP - 10·log10(4π·d²)`.
///
/// ```
/// // 50 dBW EIRP from GEO ≈ -112 dBW/m²
/// let pfd = rf_r::flux::pfd_dbw_m2(50.0, 35_786e3);
/// assert!((pfd + 112.06).abs() < 0.01);
/// ```
#[must_use]
#[inline]
pub fn pfd_dbw_m2(eirp_dbw: f64, distance_m: f64) -> f64 {
    eirp_dbw - spreading_loss_db(distance_m)
}

/// EIRP required to produce a given flux density at `distance_m`, dBW.
///
/// Used e.g. to convert a transponder saturation flux density into uplink EIRP.
#[must_use]
#[inline]
pub fn eirp_from_pfd_dbw(pfd_dbw_m2: f64, distance_m: f64) -> f64 {
    pfd_dbw_m2 + spreading_loss_db(distance_m)
}

/// Power flux density in a reference bandwidth, dBW/m²/`ref_bandwidth_hz`.
///
/// Assumes a flat spectrum over `occupied_bandwidth_hz`. When the reference
/// bandwidth is wider than the occupied bandwidth, the whole carrier falls
/// inside it and the total PFD is returned (typical ITU limits use 4 kHz or 1 MHz).
#[must_use]
#[inline]
pub fn pfd_in_bandwidth_dbw_m2(
    total_pfd_dbw_m2: f64,
    occupied_bandwidth_hz: f64,
    ref_bandwidth_hz: f64,
) -> f64 {
    let captured_bandwidth_hz = ref_bandwidth_hz.min(occupied_bandwidth_hz);
    total_pfd_dbw_m2 - to_db(occupied_bandwidth_hz / captured_bandwidth_hz)
}

/// Gain of an ideal 1 m² aperture, dBi: `10·log10(4π/λ²)`.
#[must_use]
#[inline]
pub fn gain_one_square_meter_db(freq_hz: f64) -> f64 {
    FOUR_PI_DB - amplitude_to_db(wavelength_m(freq_hz))
}

/// Power received by an antenna of gain `rx_gain_dbi` in a flux density, dBW:
/// `PFD + G - G_1m²`.
#[must_use]
#[inline]
pub fn received_power_from_pfd_dbw(pfd_dbw_m2: f64, rx_gain_dbi: f64, freq_hz: f64) -> f64 {
    pfd_dbw_m2 + rx_gain_dbi - gain_one_square_meter_db(freq_hz)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fspl::fspl_db;
    use crate::test_util::assert_close;
    use std::f64::consts::PI;

    #[test]
    fn spreading_loss_matches_definition() {
        let distance = 38_000e3;
        assert_close(
            spreading_loss_db(distance),
            10.0 * (4.0 * PI * distance * distance).log10(),
            1e-10,
        );
    }

    #[test]
    fn spreading_loss_at_zero_distance_is_negative_infinity() {
        assert!(spreading_loss_db(0.0) < f64::MIN);
    }

    #[test]
    fn pfd_geo_reference_value() {
        // 10·log10(4π·35 786 000²) = 162.066 dB m²
        assert_close(pfd_dbw_m2(50.0, 35_786e3), 50.0 - 162.066_36, 1e-5);
    }

    #[test]
    fn eirp_from_pfd_inverts_pfd() {
        let distance = 37_500e3;
        assert_close(
            eirp_from_pfd_dbw(pfd_dbw_m2(48.0, distance), distance),
            48.0,
            1e-10,
        );
    }

    #[test]
    fn pfd_in_4_khz_of_36_mhz_carrier() {
        // 10·log10(36e6 / 4e3) = 39.54 dB
        assert_close(pfd_in_bandwidth_dbw_m2(-110.0, 36e6, 4e3), -149.542, 1e-3);
    }

    #[test]
    fn pfd_in_bandwidth_wider_than_carrier_is_total_pfd() {
        assert_close(pfd_in_bandwidth_dbw_m2(-110.0, 2e3, 1e6), -110.0, 0.0);
    }

    #[test]
    fn pfd_in_equal_bandwidth_is_total_pfd() {
        assert_close(pfd_in_bandwidth_dbw_m2(-110.0, 1e6, 1e6), -110.0, 0.0);
    }

    #[test]
    fn pfd_in_bandwidth_with_negative_bandwidth_is_nan() {
        assert!(pfd_in_bandwidth_dbw_m2(-110.0, 1e6, -4e3).is_nan());
    }

    #[test]
    fn gain_of_one_square_meter_reference_value() {
        // 12 GHz: 10·log10(4π / 0.024983²) = 43.04 dB
        assert_close(gain_one_square_meter_db(12e9), 43.04, 0.01);
    }

    #[test]
    fn received_power_from_pfd_equals_friis() {
        let (eirp, gain, distance, freq) = (52.0, 40.0, 38_000e3, 11.7e9);
        let via_flux = received_power_from_pfd_dbw(pfd_dbw_m2(eirp, distance), gain, freq);
        let via_friis = eirp + gain - fspl_db(distance, freq);
        assert_close(via_flux, via_friis, 1e-10);
    }
}
