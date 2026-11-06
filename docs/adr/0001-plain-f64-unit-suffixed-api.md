# 0001. Plain f64 API with unit-suffixed names

Date: 2026-11-06
Status: Accepted

## Context

RF calculations mix many units (dB, dBW, dBm, dBi, dB/K, dB·Hz, Hz, m, K). Mistakes between them are common.
The library must also be fast enough for sweeps and Monte-Carlo runs over millions of points.

## Options considered

1. **Newtypes per unit** (`Db(f64)`, `Hertz(f64)`, ...): type-safe, but heavy in numeric code. Every
   formula needs wrapping and unwrapping, and slices of newtypes are awkward to pass to other libraries.
2. **A units crate** (e.g. `uom`): very safe, but adds a dependency, long compile times and verbose code,
   and does not model dB-domain quantities well.
3. **Plain `f64` with the unit in every function and parameter name** (`fspl_db(distance_m, freq_hz)`).

## Decision

Option 3. Every public function takes and returns `f64`. Units are encoded in names. SI base units are the
default, and other units are explicitly suffixed (`_km`, `_ghz`, `_deg`).

## Consequences

- No overhead, simple loops that the compiler can vectorise, and easy interop with `&[f64]` data.
- The compiler cannot catch unit mix-ups. Naming discipline and documentation must do that instead.
- A typed wrapper layer can be added later on top, without breaking this API.
