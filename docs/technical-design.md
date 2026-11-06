# rf-r technical design

Companion to the [arc42 architecture](arc42/architecture.md) and the [C4 diagrams](c4/).
The API reference generated from the code is `cargo doc --no-deps --open`.

## Overview

`rf-r` is a pure-computation Rust library. Every public function takes `f64` values (or `f64` slices) and returns `f64`.
Units are encoded in the names. Functions are `#[inline]`, allocate nothing, keep no state and do no I/O.
Edge inputs follow IEEE-754 (`-inf`, `+inf`, `NaN`) instead of returning `Result`. See the ADRs in [`adr/`](adr/) for why.

## Components

| Module | Public items | Depends on |
|---|---|---|
| `constants` | `SPEED_OF_LIGHT`, `BOLTZMANN`, `BOLTZMANN_DBW`, `T0_K`, `EARTH_RADIUS_M`, `GEO_RADIUS_M`, `GEO_ALTITUDE_M`, `FSPL_CONST_SI_DB`, `FSPL_CONST_KM_GHZ_DB`, `FOUR_PI_DB` | none |
| `db` | `to_db`, `from_db`, `amplitude_to_db`, `amplitude_from_db`, `watts_to_dbw`, `dbw_to_watts`, `watts_to_dbm`, `dbm_to_watts`, `dbw_to_dbm`, `dbm_to_dbw`, `log_sum`, `log_sum_all`, `log_diff`, `to_db_in_place`, `from_db_in_place` | none |
| `fspl` | `wavelength_m`, `frequency_hz`, `fspl_db`, `fspl_db_km_ghz`, `fspl_db_batch` | `constants`, `db` |
| `antenna` | `HPBW_FACTOR_DEG`, `aperture_gain_dbi`, `parabolic_gain_dbi`, `half_power_beamwidth_deg`, `g_over_t_db_k` | `db`, `fspl` |
| `noise` | `NoiseStage`, `noise_temperature_k`, `noise_figure_db`, `lossy_line_noise_temperature_k`, `noise_density_dbw_hz`, `noise_power_dbw`, `cascade_noise_temperature_k` | `constants`, `db` |
| `link` | `Link` (`cn0_dbhz`, `cn_db`, `ebn0_db`), `eirp_dbw`, `cn0_dbhz`, `cn_db`, `ebn0_db`, `margin_db`, `combine_cn2_db`, `combine_cn_db` | `constants`, `db` |
| `flux` | `spreading_loss_db`, `pfd_dbw_m2`, `eirp_from_pfd_dbw`, `pfd_in_bandwidth_dbw_m2`, `gain_one_square_meter_db`, `received_power_from_pfd_dbw` | `constants`, `db`, `fspl` |
| `geometry` | `slant_range_m`, `slant_range_geo_m`, `elevation_rad` | `constants` |
| `prelude` | re-exports all of the above | all |

## Data model

No persistent data. The two public structs are plain `Copy` value types:

| Struct | Fields |
|---|---|
| `link::Link` | `eirp_dbw`, `path_loss_db`, `extra_losses_db`, `g_over_t_db_k` |
| `noise::NoiseStage` | `gain_db`, `noise_temp_k` |

## Interfaces

Rust API only. No CLI, network or file interface. Formulas:

| Function | Formula |
|---|---|
| `to_db(x)` / `from_db(x)` | `10·log10(x)` / `10^(x/10)` |
| `log_sum_all(xs)` | `max + 10·log10(Σ 10^((xi − max)/10))` |
| `combine_cn_db(xs)` | `min − 10·log10(Σ 10^((min − xi)/10))` |
| `fspl_db(d, f)` | `20·log10(d·f) + 20·log10(4π/c)` |
| `parabolic_gain_dbi(D, f, η)` | `10·log10(η·(π·D·f/c)²)` |
| `half_power_beamwidth_deg(D, f)` | `70·λ/D` |
| `cn0_dbhz(EIRP, L, G/T)` | `EIRP − L + G/T − 10·log10(k)` |
| `pfd_dbw_m2(EIRP, d)` | `EIRP − 10·log10(4π·d²)` |
| `cascade_noise_temperature_k` | `T1 + T2/G1 + T3/(G1·G2) + …` |
| `slant_range_m(e, r)` | `sqrt(r² − Re²·cos²e) − Re·sin e` |

Example:

```rust
use rf_r::prelude::*;

let cn = Link {
    eirp_dbw: 52.0,
    path_loss_db: fspl_db(38_000e3, 12e9),
    extra_losses_db: 0.5,
    g_over_t_db_k: 20.0,
}
.cn_db(36e6);
```

## Key flows

**Total C/N of a bent-pipe satellite link**: uplink `Link::cn_db`, then downlink `Link::cn_db`, then
`combine_cn_db(&[up, down, c_over_i, c_over_im])`. Error paths: a `NaN` in any input gives a `NaN` total.
A `-inf` contribution (no carrier) gives `-inf`. `+inf` contributions (no impairment) are ignored.

**Flux to received power**: `pfd_dbw_m2` gives the flux at the receiver, then `received_power_from_pfd_dbw`
gives the power at the antenna output. This equals the Friis result `EIRP + G − FSPL`, which is covered by a unit test.

## Configuration

None. The library has no settings, environment variables or feature flags.

## Build, test and deploy

| Task | Command |
|---|---|
| Build | `cargo build --release` |
| Test (unit, integration, doc) | `cargo test --all-features` |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` |
| Format check | `cargo fmt --all --check` |
| Coverage | `cargo llvm-cov --all-features --fail-under-lines 100` |
| Benchmarks | `cargo bench` |
| Dependency audit | `cargo audit` |
| Docs checks | `make docs-check` |
| Oldest supported Rust (1.85) | `make msrv` |
| Everything CI runs | `make check` |

CI (`.github/workflows/ci.yml`) runs these on every push and pull request to `main`.
Dependabot (`.github/dependabot.yml`) opens weekly update PRs for Cargo dependencies and GitHub Actions.

## Operations

Not applicable: no running service, logs or metrics. Use benchmarks to watch for performance regressions.

## Security

- No I/O, no parsing and no `unsafe` (`unsafe_code = "forbid"`). The attack surface is limited to the
  numeric values passed in by the caller.
- No runtime dependencies. Dev dependencies (`criterion` and its tree) are pinned in `Cargo.lock`
  and scanned with `cargo audit` in CI.
- CI runs gitleaks to catch committed secrets. Third-party actions are pinned to commit SHAs, and the workflow token is read-only.

## Known limitations

- Units are not type-checked ([ADR 0001](adr/0001-plain-f64-unit-suffixed-api.md)).
- Spherical Earth in `geometry`.
- `pfd_in_bandwidth_dbw_m2` assumes a flat spectrum over the occupied bandwidth.
- `half_power_beamwidth_deg` uses the empirical 70·λ/D factor, which depends on the illumination taper.
- No atmospheric, rain or scintillation models yet.
