//! GEO Ku-band link budget: uplink, downlink, interference and total C/N.
//!
//! Run with `cargo run --example geo_link_budget`.

use rf_r::prelude::*;

fn main() {
    let elevation_deg = 30.0_f64;
    let bandwidth_hz = 36e6;
    let range_m = slant_range_geo_m(elevation_deg.to_radians());

    let uplink_eirp = eirp_dbw(
        watts_to_dbw(100.0),
        parabolic_gain_dbi(2.4, 14e9, 0.65),
        1.0,
    );
    let uplink = Link {
        eirp_dbw: uplink_eirp,
        path_loss_db: fspl_db(range_m, 14e9),
        extra_losses_db: 1.0,
        g_over_t_db_k: 3.0,
    };

    let rx_temp_k = cascade_noise_temperature_k(&[NoiseStage {
        gain_db: 60.0,
        noise_temp_k: noise_temperature_k(0.9),
    }]) + 50.0;
    let downlink = Link {
        eirp_dbw: 50.0,
        path_loss_db: fspl_db(range_m, 11.7e9),
        extra_losses_db: 0.5,
        g_over_t_db_k: g_over_t_db_k(parabolic_gain_dbi(1.2, 11.7e9, 0.65), rx_temp_k),
    };

    let uplink_cn = uplink.cn_db(bandwidth_hz);
    let downlink_cn = downlink.cn_db(bandwidth_hz);
    let carrier_to_interference_db = 22.0;
    let total_cn = combine_cn_db(&[uplink_cn, downlink_cn, carrier_to_interference_db]);

    println!("Slant range          : {:>10.1} km", range_m / 1e3);
    println!("Uplink EIRP          : {uplink_eirp:>10.2} dBW");
    println!(
        "Uplink PFD           : {:>10.2} dBW/m²",
        pfd_dbw_m2(uplink_eirp - 1.0, range_m)
    );
    println!("Uplink FSPL          : {:>10.2} dB", uplink.path_loss_db);
    println!("Uplink C/N           : {uplink_cn:>10.2} dB");
    println!("Downlink FSPL        : {:>10.2} dB", downlink.path_loss_db);
    println!(
        "Downlink G/T         : {:>10.2} dB/K",
        downlink.g_over_t_db_k
    );
    println!("Downlink C/N         : {downlink_cn:>10.2} dB");
    println!("C/I                  : {carrier_to_interference_db:>10.2} dB");
    println!("Total C/N            : {total_cn:>10.2} dB");
    println!(
        "Margin vs 9.0 dB     : {:>10.2} dB",
        margin_db(total_cn, 9.0)
    );
}
