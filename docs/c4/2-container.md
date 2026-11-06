# C4 level 2: Containers

`rf-r` is a single container: the `rf-r` library crate, compiled into the caller's binary.
Benchmarks and examples are development-only targets of the same crate.

```mermaid
C4Container
  title Containers: rf-r
  Person(developer, "Application developer", "Writes Rust code using rf-r")
  System_Boundary(rfr, "rf-r") {
    Container(lib, "rf_r library", "Rust crate (lib)", "Pure f64 RF and link budget functions")
    Container(example, "geo_link_budget", "Rust example binary", "Prints a full GEO Ku-band link budget")
    Container(bench, "rf_bench", "Criterion benchmark", "Measures hot-path latency")
  }
  Rel(developer, lib, "Calls", "Rust API")
  Rel(example, lib, "Calls", "Rust API")
  Rel(bench, lib, "Measures", "Rust API")
```

Elements:
- **rf_r library**: `src/`.
- **geo_link_budget**: `examples/geo_link_budget.rs`.
- **rf_bench**: `benches/rf_bench.rs`.
