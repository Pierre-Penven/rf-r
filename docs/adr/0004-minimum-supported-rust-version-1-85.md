# 0004. Minimum supported Rust version 1.85

Date: 2026-11-06
Status: Accepted

## Context

Users need to know which Rust toolchains can build `rf-r`, and that this won't break silently.
The crate uses edition 2024, which needs Rust 1.85 or newer. The dev-only `criterion` 0.8.2 dependency needs Rust 1.86.

## Options considered

1. No declared MSRV: simplest, but users find out only when their build fails.
2. MSRV 1.85, the lowest version edition 2024 allows. Checked by building the library only.
3. MSRV 1.86, so that tests and benchmarks also run on the MSRV.

## Decision

Option 2. `rust-version = "1.85"` in `Cargo.toml`. The CI job `msrv` (and `make msrv`) runs
`cargo +1.85 check --locked --lib`. Tests, benchmarks and examples run on stable.

## Consequences

- Library users on Rust 1.85 or newer are supported. Cargo gives a clear error on older toolchains.
- Clippy warns when code in `src/` uses std APIs newer than 1.85.
- Contributors need Rust 1.86 or newer to run the tests.
- Raising the MSRV is a deliberate change that needs a new ADR.
