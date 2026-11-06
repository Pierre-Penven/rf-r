//! Earth–satellite geometry (spherical Earth).

use crate::constants::{EARTH_RADIUS_M, GEO_RADIUS_M};

/// Slant range from an Earth station to a satellite, m.
///
/// `d = sqrt(r² - Re²·cos²(e)) - Re·sin(e)`, with `r` the orbit radius from
/// Earth's centre, `Re` the Earth radius and `e` the elevation angle.
/// Returns `NaN` if the orbit radius is smaller than the Earth radius.
///
/// ```
/// use rf_r::geometry::slant_range_m;
/// use rf_r::constants::{GEO_ALTITUDE_M, GEO_RADIUS_M};
///
/// let zenith = slant_range_m(90f64.to_radians(), GEO_RADIUS_M);
/// assert!((zenith - GEO_ALTITUDE_M).abs() < 1e-6);
/// ```
#[must_use]
#[inline]
pub fn slant_range_m(elevation_rad: f64, orbit_radius_m: f64) -> f64 {
    let (sin_el, cos_el) = elevation_rad.sin_cos();
    let horizontal = EARTH_RADIUS_M * cos_el;
    (orbit_radius_m * orbit_radius_m - horizontal * horizontal).sqrt() - EARTH_RADIUS_M * sin_el
}

/// Slant range to a geostationary satellite, m. See [`slant_range_m`].
#[must_use]
#[inline]
pub fn slant_range_geo_m(elevation_rad: f64) -> f64 {
    slant_range_m(elevation_rad, GEO_RADIUS_M)
}

/// Elevation angle, rad, at which a satellite at `orbit_radius_m` is seen at
/// slant range `slant_range_m` (inverse of [`slant_range_m`]).
///
/// `sin(e) = (r² - Re² - d²) / (2·Re·d)`. Returns `NaN` for geometrically
/// impossible inputs.
#[must_use]
#[inline]
pub fn elevation_rad(slant_range_m: f64, orbit_radius_m: f64) -> f64 {
    let numerator = orbit_radius_m * orbit_radius_m
        - EARTH_RADIUS_M * EARTH_RADIUS_M
        - slant_range_m * slant_range_m;
    (numerator / (2.0 * EARTH_RADIUS_M * slant_range_m)).asin()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::GEO_ALTITUDE_M;
    use crate::test_util::assert_close;

    #[test]
    fn slant_range_at_zenith_is_altitude() {
        assert_close(slant_range_geo_m(90f64.to_radians()), GEO_ALTITUDE_M, 1e-6);
    }

    #[test]
    fn slant_range_at_horizon_is_tangent_length() {
        let expected = (GEO_RADIUS_M * GEO_RADIUS_M - EARTH_RADIUS_M * EARTH_RADIUS_M).sqrt();
        assert_close(slant_range_geo_m(0.0), expected, 1e-6);
    }

    #[test]
    fn slant_range_geo_reference_value() {
        // ≈ 41 679 km at 0° elevation, ≈ 38 612 km at 30° elevation
        assert_close(slant_range_geo_m(0.0) / 1e3, 41_679.0, 1.0);
        assert_close(slant_range_geo_m(30f64.to_radians()) / 1e3, 38_611.7, 0.1);
    }

    #[test]
    fn slant_range_decreases_with_elevation() {
        let low = slant_range_geo_m(10f64.to_radians());
        let high = slant_range_geo_m(60f64.to_radians());
        assert!(low > high);
    }

    #[test]
    fn slant_range_for_orbit_below_earth_surface_is_nan() {
        assert!(slant_range_m(0.0, EARTH_RADIUS_M / 2.0).is_nan());
    }

    #[test]
    fn elevation_inverts_slant_range() {
        let leo_radius = EARTH_RADIUS_M + 550e3;
        for &deg in &[5.0_f64, 25.0, 45.0, 89.0] {
            let range = slant_range_m(deg.to_radians(), leo_radius);
            assert_close(elevation_rad(range, leo_radius).to_degrees(), deg, 1e-9);
        }
    }

    #[test]
    fn elevation_for_impossible_range_is_nan() {
        // Range shorter than the altitude cannot reach the orbit
        assert!(elevation_rad(1_000.0, GEO_RADIUS_M).is_nan());
    }
}
