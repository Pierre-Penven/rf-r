# 0002. dB conversion via ln/exp with precomputed factors

Date: 2026-11-06
Status: Accepted

## Context

`to_db` and `from_db` are the most-called functions in any link budget code. The direct forms are
`10·log10(x)` and `10f64.powf(x / 10)`.

## Options considered

1. `10.0 * x.log10()` and `10f64.powf(x / 10.0)`.
2. `x.ln() * (10 / ln 10)` and `(x * (ln 10 / 10)).exp()`, with the factors as `const`.

## Decision

Option 2. Measured with `cargo bench` on the development machine:

| Function | Option 1 | Option 2 |
|---|---|---|
| to dB | 4.3 ns | 2.6 ns |
| from dB | 7.2 ns | 2.6 ns |

## Consequences

- Faster conversions everywhere they are used.
- Results can differ from option 1 by a few ULPs. Tests compare against `log10` with a 1e-12 tolerance,
  so tests never assert exact bit equality on dB values.
