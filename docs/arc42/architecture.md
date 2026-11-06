# rf-r architecture (arc42)

Single-file arc42 description of `rf-r`, kept short because the system is one library crate.
For implementation detail see the [technical design](../technical-design.md); for diagrams see [C4](../c4/).

## 1. Introduction and goals

`rf-r` is a Rust library for RF engineering and satellite link budget calculations: dB conversions,
log sums, free-space path loss, antenna gain, noise, C/N and flux density.

Top requirements:
- Cover the everyday link budget chain: EIRP → path loss → flux → G/T → C/N0 → C/N → combined C/N.
- Be usable in hot loops (Monte-Carlo, sweeps, coverage maps) with no per-call overhead.

Quality goals, in priority order:

| # | Quality goal | Meaning |
|---|---|---|
| 1 | Correctness | Results match the textbook formulas to ≤ 1e-9 dB; no overflow or underflow in log sums |
| 2 | Performance | Scalar functions run in a few ns, with no allocation and no error branching |
| 3 | Ease of use | Units are in the names; one `prelude` import; every function has a runnable example |

## 2. Constraints

- Rust edition 2024. Minimum supported Rust version (MSRV) 1.85, declared as `rust-version` in `Cargo.toml` and checked in CI ([ADR 0004](../adr/0004-minimum-supported-rust-version-1-85.md)).
  The dev-only `criterion` dependency needs 1.86, so tests and benchmarks run on stable.
- No runtime dependencies; dev-only dependencies for benchmarks (`criterion`).
- `unsafe` code is forbidden (`[lints.rust] unsafe_code = "forbid"`).
- MIT licence.

## 3. Context and scope

`rf-r` is a library linked into the caller's program. It has no I/O, no network, no files,
no configuration and no global state. See the [C4 context diagram](../c4/1-context.md).

| Neighbour | Interface |
|---|---|
| Application developer's Rust program | Calls public functions; Cargo dependency |
| GitHub | Distribution of the source as a Cargo Git dependency (not on crates.io yet) |

## 4. Solution strategy

- **Plain `f64` API with unit-suffixed names** instead of newtypes: zero friction in numeric code
  and auto-vectorisable loops ([ADR 0001](../adr/0001-plain-f64-unit-suffixed-api.md)).
- **`ln`/`exp` with precomputed factors** for dB conversion, faster than `log10`/`powf`
  ([ADR 0002](../adr/0002-ln-exp-db-conversion.md)).
- **Max/min-shifted log sums** so power sums and C/N combinations never overflow
  ([ADR 0003](../adr/0003-stable-log-domain-sums.md)).
- **IEEE-754 semantics instead of `Result`**: edge inputs give `-inf`/`+inf`/`NaN`, documented per function.
- Physical constants in dB are precomputed `const`s, each verified against its definition in a test.

## 5. Building block view

One crate, one module per topic. See the [C4 component diagram](../c4/3-component.md).

| Module | Responsibility |
|---|---|
| `constants` | c, k, k in dBW/K/Hz, T0, Earth and GEO radii, FSPL and 4π constants in dB |
| `db` | linear ↔ dB, W/dBW/dBm, `log_sum`, `log_sum_all`, `log_diff`, in-place batches |
| `fspl` | wavelength, free-space path loss, batch FSPL |
| `antenna` | aperture and parabolic gain, 3 dB beamwidth, G/T |
| `noise` | NF ↔ noise temperature, lossy line, kT / kTB, Friis cascade (`NoiseStage`) |
| `link` | EIRP, C/N0, C/N, Eb/N0, margin, `Link` struct, C/N combination |
| `flux` | spreading loss, PFD, PFD in a reference bandwidth, gain of 1 m², received power from PFD |
| `geometry` | slant range ↔ elevation for any orbit radius |
| `prelude` | glob re-export of all the above |

There are no dependency cycles. `constants` and `db` depend on nothing else in the crate. `fspl` uses both.
`antenna` and `flux` use `fspl` and `db` (`flux` also uses `constants`). `link` and `noise` use `db` and `constants`.
`geometry` uses only `constants`.

## 6. Runtime view

Typical downlink C/N calculation (see `examples/geo_link_budget.rs`):

1. `slant_range_geo_m(elevation)` → distance.
2. `fspl_db(distance, freq)` → path loss.
3. `parabolic_gain_dbi(...)`, `cascade_noise_temperature_k(...)`, `g_over_t_db_k(...)` → G/T.
4. Build a `Link { eirp, path_loss, extra_losses, g_over_t }` and call `cn_db(bandwidth)`.
5. `combine_cn_db(&[uplink, downlink, c_over_i])` → total C/N; `margin_db(total, required)`.

All steps are pure function calls; no state is kept between them.

## 7. Deployment view

Not applicable at runtime: the crate is compiled into the caller's binary.
Source is hosted at <https://github.com/Pierre-Penven/rf-r>; CI runs on GitHub Actions (`.github/workflows/ci.yml`).
The crate is not published to crates.io yet (decision of 2026-11-06); users add it as a Cargo Git dependency.

## 8. Crosscutting concepts

- **Units**: SI base units by default (m, Hz, W, K, rad); other units are explicitly suffixed (`_km`, `_ghz`, `_deg`).
  Losses are positive dB values that are subtracted.
- **Error handling**: no `Result` in the scalar path; IEEE-754 special values, documented per function.
  Batch functions panic on mismatched slice lengths (documented under `# Panics`).
- **Performance**: `#[inline]` and `#[must_use]` on all scalar functions; fat LTO and one codegen unit in release and bench profiles.
- **Testing**: unit tests per module, integration tests against independently computed reference values, doc tests, 100 % line coverage enforced in CI.
- **Security**: no I/O and no `unsafe`; supply chain is checked with `cargo audit`, gitleaks and Dependabot.
- **Logging, configuration, persistence**: none, by design.

## 9. Architecture decisions

| ADR | Decision |
|---|---|
| [0001](../adr/0001-plain-f64-unit-suffixed-api.md) | Plain `f64` API with unit-suffixed names |
| [0002](../adr/0002-ln-exp-db-conversion.md) | dB conversion via `ln`/`exp` with precomputed factors |
| [0003](../adr/0003-stable-log-domain-sums.md) | Max/min-shifted log-domain sums |
| [0004](../adr/0004-minimum-supported-rust-version-1-85.md) | Minimum supported Rust version 1.85 |

## 10. Quality requirements

| Scenario | Target | How it is checked |
|---|---|---|
| Full GEO Ku link budget vs reference | ≤ 1e-9 dB difference | `tests/link_budget.rs` |
| `log_sum(4000, 4000)` | Finite, equals 4003.0103 dB | unit test in `db` |
| `to_db` / `from_db` latency | Faster than `10·log10` / `powf` | `cargo bench` (`benches/rf_bench.rs`) |
| Code coverage | 100 % of lines | `cargo llvm-cov --fail-under-lines 100` in CI |

## 11. Risks and technical debt

| Risk | Impact | Mitigation |
|---|---|---|
| Plain `f64` lets callers mix units (e.g. km passed as m) | Wrong results, no compile error | Unit suffixes in every name; possible typed wrapper layer later |
| MSRV drift: a change uses a Rust feature newer than 1.85 | Users on 1.85 can no longer build | CI `msrv` job builds the library with 1.85; clippy checks API use against `rust-version` |
| Spherical Earth model in `geometry` | Up to ~0.1 % slant-range error vs WGS-84 ellipsoid | Documented; ellipsoidal model if needed |
| No propagation models yet (rain, gases, scintillation) | Users add these losses by hand | Planned: ITU-R P.618 / P.676 / P.838 |

## 12. Glossary

| Term | Meaning |
|---|---|
| dB, dBW, dBm, dBi | Ratio in decibels; power relative to 1 W; relative to 1 mW; antenna gain relative to an isotropic radiator |
| EIRP | Effective isotropic radiated power: transmit power + antenna gain − feeder loss |
| FSPL | Free-space path loss: `20·log10(4π·d·f/c)` |
| PFD | Power flux density at the receiver, dBW/m² |
| G/T | Receive figure of merit: antenna gain minus system noise temperature, dB/K |
| C/N0, C/N | Carrier-to-noise density (dB·Hz) and carrier-to-noise in a bandwidth (dB) |
| C/I, C/IM | Carrier-to-interference and carrier-to-intermodulation ratios |
| Eb/N0 | Energy per bit over noise density |
| Log sum | Power addition of dB values: `10·log10(Σ 10^(xi/10))` |
| GEO | Geostationary orbit, ≈ 35 786 km altitude |
| Slant range | Straight-line distance from Earth station to satellite |
