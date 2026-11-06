# CLAUDE.md

Guidance for Claude Code when working in this repository.

## Project

`rf-r` is a Rust library crate for **high-performance RF and satellite link budget calculations**.

Scope:
- **dB math**: linear ↔ dB (power and amplitude), W ↔ dBW ↔ dBm, log sum (power addition in dB), log difference
- **Free-space path loss** (FSPL), wavelength ↔ frequency
- **Antennas**: parabolic gain from diameter/efficiency, 3 dB beamwidth, G/T
- **Noise**: noise figure ↔ noise temperature, noise power `kTB`
- **Link budget**: EIRP, C/N0, C/N, Eb/N0, combined C/N (uplink + downlink + C/I + IM: `1/(C/N)tot = Σ 1/(C/N)i`)
- **Flux**: power flux density (PFD, dBW/m²), PFD in a reference bandwidth (per 4 kHz, per MHz), spreading loss, gain of 1 m², saturation flux density ↔ EIRP
- **Geometry**: slant range from elevation angle (GEO/LEO), Earth/GEO constants

## Skills

At the start of every task, load **all** relevant skills before planning or coding, without waiting to be asked, and follow each one's checklist:

| Skill | Load when |
|---|---|
| `rust-development` | always (any code change) |
| `unit-testing-coverage` | always (any code change) |
| `clean-code` | always (any code change) |
| `secure-development` | adding or updating dependencies (`cargo audit`), CI, input validation |
| `application-documentation` | README, crate docs, architecture or design notes |
| `requirements-analysis` | scoping a new feature or calculation (e.g. rain attenuation, ITU-R models) |

## Commands

```bash
cargo build
cargo test --all-features                                  # unit + integration + doc tests
cargo fmt --all                                            # format
cargo clippy --all-targets --all-features -- -D warnings   # lint (must be clean)
cargo doc --no-deps --open                                 # API docs
cargo bench                                                # criterion benchmarks (benches/)
cargo test <name>                                          # run a single test
make check                                                 # everything CI runs: fmt, clippy, test, doc, coverage, audit, docs checks
make msrv                                                  # build the library with the oldest supported Rust (1.85)
make docs-check                                            # required docs, arc42 sections, C4 diagrams, broken links
bash scripts/docs/new_adr.sh "title"                       # new architecture decision record in docs/adr/
```

Before reporting work done, run `cargo fmt --all`, then `make check`. All of it must pass, including 100 % line coverage and `cargo audit`.
When public API, modules or design change, update the README, `docs/` (arc42, C4, technical design) and add an ADR for significant decisions.

## Layout

```
src/
  lib.rs        crate docs, module declarations, `prelude`
  constants.rs  c, k (linear and dBW/K/Hz), T0 = 290 K, Earth radius, GEO radius, FSPL constants
  db.rs         to_db/from_db, amplitude, dBW/dBm, log_sum, log_sum_all, log_diff, in-place batch
  fspl.rs       wavelength, fspl_db (m, Hz), fspl_db_km_ghz, batch FSPL
  antenna.rs    parabolic gain, beamwidth, G/T
  noise.rs      NF ↔ T, noise power
  link.rs       EIRP, C/N0, C/N, Eb/N0, combine_cn, `Link` struct
  flux.rs       PFD, spectral PFD, spreading loss, gain of 1 m²
  geometry.rs   slant range, elevation
benches/        criterion benchmarks
tests/          integration tests (full link budget scenarios vs. reference values)
examples/       worked link budgets (e.g. GEO Ku-band downlink)
docs/           arc42/architecture.md, c4/ (Mermaid), technical-design.md, adr/
scripts/docs/   docs checks (check_docs.sh, check_links.py) and new_adr.sh
.github/        CI workflow (ci.yml) and Dependabot config
```

## Design rules

**Performance first**
- Core functions take and return plain `f64`; mark them `#[inline]` and `#[must_use]`. No allocation, no `Result` in hot math.
- dB conversions use `ln`/`exp` with precomputed factors (`10/ln10`, `ln10/10`) rather than `log10`/`powf`.
- Precompute physical constants in dB (e.g. `BOLTZMANN_DBW = -228.5992`, `FSPL_CONST_SI_DB = 20·log10(4π/c)`) as `const`; verify each against its definition in a unit test.
- Provide slice-based batch variants (`*_batch(input, ..., out)`, `*_in_place`) for hot loops; keep them auto-vectorisable (simple loops, no branches inside).
- Release/bench profiles use `lto = "fat"`, `codegen-units = 1`.
- No runtime dependencies unless clearly justified. Optional heavy features (e.g. `rayon` parallel batches, `serde`) go behind Cargo features.
- Any optimisation must be backed by a criterion benchmark.

**Numerical correctness**
- `log_sum` / `log_sum_all` must be **max-shifted** (`max + 10·log10(Σ 10^((xi-max)/10))`) so they never overflow and stay accurate for widely separated values; use `ln_1p` for the two-term case.
- Follow IEEE-754 semantics and document them: `to_db(0) = -inf`, negative linear → `NaN`, `-inf` dB is the neutral element of `log_sum`.
- Combining C/N ratios is an inverse power sum: `cn_tot = -log_sum_all(-cn_i)`.

**Compatibility**
- MSRV is Rust 1.85 (`rust-version` in Cargo.toml). Don't use std APIs or language features newer than that in `src/`; raising it needs an ADR.
- Not published to crates.io yet; don't add publish steps until decided.

**Units and naming**
- Units are encoded in names: `_db`, `_dbw`, `_dbm`, `_dbi`, `_db_k` (G/T), `_dbhz`, `_hz`, `_m`, `_km`, `_k` (kelvin), `_rad`, `_deg`, `_w`.
- SI base units (m, Hz, W, K, rad) by default; convenience variants (km, GHz, deg) are explicitly suffixed.
- Losses are positive dB values and are subtracted.

**API and code quality**
- Library: no `unwrap()`/`expect()` outside tests. If a validated/builder API needs errors, use a dedicated error enum (`thiserror`) — never in the scalar fast path.
- `#![forbid(unsafe_code)]`-equivalent via `[lints.rust] unsafe_code = "forbid"`; `missing_docs = "warn"`; clippy `pedantic`.
- Every public item has `///` docs with the formula and a runnable doc-test example; document `# Panics` (e.g. batch length mismatch).

## Testing

- Unit tests in `#[cfg(test)] mod tests` at the bottom of each module; shared `assert_close(actual, expected, tol)` helper.
- Check against: the defining formula (e.g. `20·log10(4πdf/c)`), textbook reference values (e.g. GEO Ku-band FSPL ≈ 205 dB, 1 km @ 1 GHz ≈ 92.45 dB), and identities (round trips, doubling distance = +6.02 dB, equal powers sum = +3.01 dB).
- Always include edge cases: `0`, negative, `NaN`, `±inf`, very large/small dB values (overflow safety of `log_sum`), empty slices, slice length mismatch.
- Integration tests in `tests/` run complete uplink/downlink budgets and compare total C/N with hand-computed values.
