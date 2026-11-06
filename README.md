# rf-r

[![CI](https://github.com/Pierre-Penven/rf-r/actions/workflows/ci.yml/badge.svg)](https://github.com/Pierre-Penven/rf-r/actions/workflows/ci.yml)

Fast RF and satellite link budget calculations in Rust: dB conversions, log sums, free-space path loss,
antenna gain, noise, C/N, total C/N of a satellite link, and power flux density.

It is for engineers and developers who need link budget maths in their own tools, sweeps or simulations.
Every function is a few nanoseconds, allocates nothing and has no dependencies.

## Requirements

- Rust stable toolchain, edition 2024 (install from <https://rustup.rs>)
- Linux, macOS or Windows

## Installation

The crate is not on crates.io yet. Add it from GitHub:

```bash
cargo add rf-r --git https://github.com/Pierre-Penven/rf-r
```

or in `Cargo.toml`:

```toml
[dependencies]
rf-r = { git = "https://github.com/Pierre-Penven/rf-r" }
```

## Usage

Import everything with the prelude:

```rust
use rf_r::prelude::*;

fn main() {
    // dB conversions and power sum
    let ten_watts_dbw = watts_to_dbw(10.0); // 10.0 dBW
    let two_carriers = log_sum(10.0, 10.0); // 13.01 dB

    // GEO Ku-band downlink at 30° elevation
    let range_m = slant_range_geo_m(30f64.to_radians());
    let downlink = Link {
        eirp_dbw: 50.0,
        path_loss_db: fspl_db(range_m, 11.7e9),
        extra_losses_db: 0.5,
        g_over_t_db_k: g_over_t_db_k(parabolic_gain_dbi(1.2, 11.7e9, 0.65), 125.0),
    };
    let downlink_cn = downlink.cn_db(36e6);

    // Total C/N with a 16 dB uplink and 22 dB C/I
    let total_cn = combine_cn_db(&[16.0, downlink_cn, 22.0]);

    // Flux density at the ground
    let pfd = pfd_dbw_m2(50.0, range_m);

    println!("{ten_watts_dbw:.2} dBW, {two_carriers:.2} dB");
    println!("downlink C/N {downlink_cn:.2} dB, total C/N {total_cn:.2} dB, PFD {pfd:.2} dBW/m²");
}
```

Output:

```text
10.00 dBW, 13.01 dB
downlink C/N 17.50 dB, total C/N 13.08 dB, PFD -112.73 dBW/m²
```

### What is available

| Topic | Functions |
|---|---|
| dB | `to_db`, `from_db`, `amplitude_to_db`, `amplitude_from_db`, `watts_to_dbw`, `dbw_to_watts`, `watts_to_dbm`, `dbm_to_watts`, `dbw_to_dbm`, `dbm_to_dbw` |
| Log sums | `log_sum`, `log_sum_all`, `log_diff`, `to_db_in_place`, `from_db_in_place` |
| Path loss | `wavelength_m`, `frequency_hz`, `fspl_db`, `fspl_db_km_ghz`, `fspl_db_batch` |
| Antenna | `parabolic_gain_dbi`, `aperture_gain_dbi`, `half_power_beamwidth_deg`, `g_over_t_db_k` |
| Noise | `noise_temperature_k`, `noise_figure_db`, `lossy_line_noise_temperature_k`, `noise_density_dbw_hz`, `noise_power_dbw`, `cascade_noise_temperature_k` |
| Link | `Link`, `eirp_dbw`, `cn0_dbhz`, `cn_db`, `ebn0_db`, `margin_db`, `combine_cn2_db`, `combine_cn_db` |
| Flux | `spreading_loss_db`, `pfd_dbw_m2`, `eirp_from_pfd_dbw`, `pfd_in_bandwidth_dbw_m2`, `gain_one_square_meter_db`, `received_power_from_pfd_dbw` |
| Geometry | `slant_range_m`, `slant_range_geo_m`, `elevation_rad` |

Full API documentation with an example for every function: `cargo doc --open` in your project.

### Units

Units are in the names: `_db`, `_dbw`, `_dbm`, `_dbi`, `_db_k` (G/T), `_dbhz`, `_hz`, `_m`, `_km`, `_k` (kelvin), `_rad`, `_deg`.
Inputs are SI (m, Hz, W, K, rad) unless the name says otherwise. Losses are positive numbers.

### Edge cases

Functions never panic on bad values. They follow floating-point rules:
`to_db(0.0)` is `-inf`, a negative linear value gives `NaN`, and `NaN` inputs give `NaN`.
`log_sum` and `combine_cn_db` stay accurate for any finite input, even very large or very far apart values.
The only panic is in `fspl_db_batch` when the input and output slices have different lengths.

### Full example

```bash
git clone https://github.com/Pierre-Penven/rf-r
cd rf-r
cargo run --example geo_link_budget
```

prints an uplink + downlink + C/I budget, ending with a total C/N of 13.18 dB and a margin of 4.18 dB.

## Troubleshooting

| Problem | Fix |
|---|---|
| ``error: failed to parse manifest ... feature `edition2024` is required`` | Update Rust: `rustup update stable` |
| A result is `NaN` | An input is negative or `NaN`, e.g. a negative distance or bandwidth |
| A result is `inf` or `-inf` | An input is zero, e.g. zero bandwidth or zero distance |
| Results off by 30 dB | dBW and dBm mixed up: use `dbw_to_dbm` / `dbm_to_dbw` |
| Results off by 60 dB in path loss | Distance in km passed to `fspl_db`: use metres, or `fspl_db_km_ghz` |

## Contributing and help

Report bugs and request features at <https://github.com/Pierre-Penven/rf-r/issues>.

To run the checks before a pull request:

```bash
make check   # fmt, clippy, tests, docs, coverage, audit, docs checks
```

or individually: `cargo test --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo bench`.

## Licence and documentation

MIT, see [LICENSE](LICENSE).

- [Architecture (arc42)](docs/arc42/architecture.md)
- [C4 diagrams](docs/c4/1-context.md)
- [Technical design](docs/technical-design.md)
- [Architecture decisions](docs/adr/)
