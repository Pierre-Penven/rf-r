# C4 level 1: System context

`rf-r` is a library linked into the developer's program. It has no runtime connections to other systems.

```mermaid
C4Context
  title System Context: rf-r
  Person(developer, "Application developer", "Writes link budget, sweep or simulation code in Rust")
  System(rfr, "rf-r", "Rust library for RF and satellite link budget calculations")
  System_Ext(github, "GitHub", "Hosts the source; runs CI")
  Rel(developer, rfr, "Calls functions of", "Rust API")
  Rel(developer, github, "Adds rf-r as a Cargo dependency from", "Git over HTTPS")
```

Elements:
- **Application developer**: anyone building RF tooling in Rust.
- **rf-r**: this crate.
- **GitHub**: source hosting and CI. Only involved at build time.
