# 0003. Max/min-shifted log-domain sums

Date: 2026-11-06
Status: Accepted

## Context

Power sums in dB (`10·log10(Σ 10^(xi/10))`) and C/N combinations (`-10·log10(Σ 10^(-xi/10))`) are central to
link budgets. The direct formulas overflow or underflow `f64` above about 3080 dB, and lose precision when
the inputs are very far apart.

## Options considered

1. Direct formula: simple, but fails at extreme values.
2. Shift by the maximum (power sum) or the minimum (C/N combination) before converting to linear,
   and use `ln_1p` for the two-term case.

## Decision

Option 2, in `db::log_sum`, `db::log_sum_all`, `db::log_diff`, `link::combine_cn2_db` and `link::combine_cn_db`.
The slice functions return `NaN` as soon as any input is `NaN`. The neutral elements are `-inf` for power
sums and `+inf` for C/N combinations.

## Consequences

- Correct results for any finite input (tested at 4000 dB and with values 400 dB apart).
- One extra pass over the slice to find the maximum or minimum. This is cheap compared to the `exp` calls.
