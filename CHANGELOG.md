# [0.0.3] - 2025-11-23

## Added

* **Test suite** — 80 new tests (148 total, +117%):

  * Integration tests (21) — full test fixtures
  * Parser error tests (15) — unhappy-path coverage
  * API error tests (10) — error-handling scenarios
  * Lexer tests (15) — token coverage
  * Resolver tests (18) — validation/error cases
  * Serialization tests (11) — value-type coverage

* **Benchmarks** — 12 professional Criterion benchmarks:

  * Targets: lexer, parser, end-to-end, scaling, and real-world scenarios
  * Includes statistical analysis, HTML report generation, and regression detection

* **Documentation**

  * Comprehensive docs for core components: `Lexer`, `Parser`, `Resolver`, `api`, and `error`
  * "HowTo" usage examples for `Lexer`, `Parser`, and `Resolver`
  * Intra-doc links to improve navigation

* **CI / Workflows**

  * Four optimized GitHub workflows: matrix testing (3 OS × 2 Rust toolchains), caching, and coverage
  * Linting checks added to CI, plus Clippy and rustfmt enforcement
  * Security audit and benchmark tracking integrated into CI

## Improved

* **Test coverage**: overall `43% → 45%`; lexer `42% → 53%`; serialization `45% → 88%`.
* **CI performance**: Rust caching yields ~3–5× faster runs; matrix strategy and codecov integration added.
* **Workflows**: improved lint/audit flows, daily security scans, and benchmark reporting (PR comments + GitHub Pages).

## Fixed

* Parser doctest syntax and doctests being triggered incorrectly.
* Compilation error caused by a missing `build` module reference.
* Incorrect parsing of enum values.
* `clippy` warning (`cmp_owned`) fixed in resolver tests.
* Resolver error message clarity and lexer test assertions.
* Coverage script filtering and GitHub workflow configuration issues.

## Compatibility

* **No API changes** — fully backward compatible with `v0.0.2`.
