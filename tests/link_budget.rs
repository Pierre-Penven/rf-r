//! End-to-end GEO Ku-band link budget checked against values computed
//! independently from the textbook formulas (Python, `math.log10`).

use rf_r::prelude::*;

const ELEVATION_DEG: f64 = 30.0;
const BANDWIDTH_HZ: f64 = 36e6;
const UPLINK_FREQ_HZ: f64 = 14e9;
const DOWNLINK_FREQ_HZ: f64 = 11.7e9;
const CARRIER_TO_INTERFERENCE_DB: f64 = 22.0;

#[track_caller]
fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected}, got {actual} (tolerance {tolerance})"
    );
}

fn slant_range() -> f64 {
    slant_range_geo_m(ELEVATION_DEG.to_radians())
}

fn uplink_eirp() -> f64 {
    // 100 W into a 2.4 m dish, 1 dB feeder loss
    eirp_dbw(
        watts_to_dbw(100.0),
        parabolic_gain_dbi(2.4, UPLINK_FREQ_HZ, 0.65),
        1.0,
    )
}

fn uplink() -> Link {
    Link {
        eirp_dbw: uplink_eirp(),
        path_loss_db: fspl_db(slant_range(), UPLINK_FREQ_HZ),
        extra_losses_db: 1.0,
        g_over_t_db_k: 3.0,
    }
}

fn downlink() -> Link {
    // 1.2 m dish, 50 K antenna + 75 K LNB
    let system_temp_k = 50.0 + 75.0;
    Link {
        eirp_dbw: 50.0,
        path_loss_db: fspl_db(slant_range(), DOWNLINK_FREQ_HZ),
        extra_losses_db: 0.5,
        g_over_t_db_k: g_over_t_db_k(
            parabolic_gain_dbi(1.2, DOWNLINK_FREQ_HZ, 0.65),
            system_temp_k,
        ),
    }
}

#[test]
fn slant_range_at_30_degrees() {
    assert_close(slant_range(), 38_611_732.051_574_45, 1e-3);
}

#[test]
fn uplink_eirp_matches_reference() {
    assert_close(uplink_eirp(), 68.062_502_509_549_56, 1e-9);
}

#[test]
fn uplink_cn_matches_reference() {
    assert_close(uplink().cn_db(BANDWIDTH_HZ), 15.993_915_065_326_69, 1e-9);
}

#[test]
fn downlink_g_over_t_matches_reference() {
    assert_close(downlink().g_over_t_db_k, 20.513_958_987_547_845, 1e-9);
}

#[test]
fn downlink_cn_matches_reference() {
    assert_close(downlink().cn_db(BANDWIDTH_HZ), 17.504_215_021_966_502, 1e-9);
}

#[test]
fn total_cn_matches_reference() {
    let total = combine_cn_db(&[
        uplink().cn_db(BANDWIDTH_HZ),
        downlink().cn_db(BANDWIDTH_HZ),
        CARRIER_TO_INTERFERENCE_DB,
    ]);
    assert_close(total, 13.077_772_203_994_307, 1e-9);
}

#[test]
fn uplink_flux_at_satellite_matches_reference() {
    // PFD after the 1 dB atmospheric loss
    let pfd = pfd_dbw_m2(uplink_eirp() - 1.0, slant_range());
    assert_close(pfd, -95.663_981_804_990_91, 1e-9);
}
