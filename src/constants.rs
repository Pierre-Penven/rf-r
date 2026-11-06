//! Physical and engineering constants used in RF calculations.

/// Speed of light in vacuum, m/s (exact, SI).
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Boltzmann constant, J/K (exact, SI 2019).
pub const BOLTZMANN: f64 = 1.380_649e-23;

/// Boltzmann constant in dBW/K/Hz: `10·log10(k)`.
pub const BOLTZMANN_DBW: f64 = -228.599_167_173_217_67;

/// Standard reference noise temperature, K.
pub const T0_K: f64 = 290.0;

/// WGS-84 Earth equatorial radius, m.
pub const EARTH_RADIUS_M: f64 = 6_378_137.0;

/// Geostationary orbit radius from Earth's centre, m.
pub const GEO_RADIUS_M: f64 = 42_164_172.0;

/// Geostationary altitude above the equator, m.
pub const GEO_ALTITUDE_M: f64 = GEO_RADIUS_M - EARTH_RADIUS_M;

/// `20·log10(4π/c)`: free-space path loss constant for metres and hertz.
pub const FSPL_CONST_SI_DB: f64 = -147.552_216_778_116_64;

/// `20·log10(4π·10⁹·10³/c)`: free-space path loss constant for km and GHz (≈ 92.45 dB).
pub const FSPL_CONST_KM_GHZ_DB: f64 = 92.447_783_221_883_37;

/// `10·log10(4π)`: spreading-loss constant of an isotropic sphere.
pub const FOUR_PI_DB: f64 = 10.992_098_640_220_963;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;
    use std::f64::consts::PI;

    #[test]
    fn boltzmann_dbw_matches_linear_constant() {
        assert_close(BOLTZMANN_DBW, 10.0 * BOLTZMANN.log10(), 1e-10);
    }

    #[test]
    fn fspl_si_constant_matches_definition() {
        assert_close(
            FSPL_CONST_SI_DB,
            20.0 * (4.0 * PI / SPEED_OF_LIGHT).log10(),
            1e-10,
        );
    }

    #[test]
    fn fspl_km_ghz_constant_matches_definition() {
        assert_close(
            FSPL_CONST_KM_GHZ_DB,
            20.0 * (4.0 * PI * 1e12 / SPEED_OF_LIGHT).log10(),
            1e-10,
        );
    }

    #[test]
    fn four_pi_db_matches_definition() {
        assert_close(FOUR_PI_DB, 10.0 * (4.0 * PI).log10(), 1e-12);
    }

    #[test]
    fn geo_altitude_is_about_35786_km() {
        assert_close(GEO_ALTITUDE_M, 35_786_035.0, 1.0);
    }
}
