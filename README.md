
# MON Core (`mon-core`)

[![Crates.io](https://img.shields.io/crates/v/mon-core.svg)](https://crates.io/crates/mon-core)
[![Docs.rs](https://docs.rs/mon-core/badge.svg)](https://docs.rs/mon-core)
[![License](https://img.shields.io/crates/l/mon-core.svg)](./LICENSE)

`mon-core` is the reference Rust implementation of the **MON (Mycel Object Notation)** language — a human-focused configuration and data format designed to be readable, safe, and predictable.

This crate provides the parser, analyzer, validator, and core data model used by MON-based tooling, servers, CLIs, and compilers.

> for more information please look at out website: www.mycel-lang.org 

---

## Table of Contents

* [Overview](#overview)
* [Features](#features)
* [Example](#example)
* [Rust Quick Start](#rust-quick-start)
* [Error Handling](#error-handling)
* [Development](#development)
* [Roadmap](#roadmap)
* [License](#license)

---

## Overview

MON aims to replace overly rigid formats (JSON) and overly permissive ones (YAML) with a syntax that stays readable without giving up safety or predictability.

`mon-core` implements:

* A forgiving parser with clear, context-rich error messages
* A semantic analyzer with anchor/alias resolution
* Type checking via `#struct`, `#enum`, and validated bindings
* An internal IR suitable for compilers and higher-level tooling

If you want to embed MON into your Rust application or build tooling around the language, this is the crate.

For more information about mon [docs](docs/01_the_basic_structure.md) are here :D
---

## Features

* **Clean syntax:** unquoted keys, comments, trailing commas
* **Human-friendly booleans:** `on` / `off` as well as `true` / `false`
* **Anchors & aliases:** safe reuse with explicit copy semantics (`&name`, `*name`)
* **Deep merges:** `...*anchor` for structured overrides
* **Types built in:** `#struct`, `#enum`, and `::` for validation
* **Modular imports:** `import { A, B } from "./file.mon"`
* **Detailed errors:** location-aware, colorized, actionable

---

## Example

```mon
import { ServerSettings } from "./schemas.mon"

{
    &base: {
        host: "localhost",
        port: 8080,
    },

    User: #struct {
        id(Number),
        name(String),
        roles([String...]),
    },

    admin :: User = {
        id: 1,
        name: "Alice",
        roles: ["admin", "editor"],
    },

    dev: {
        ...*base,
        port: 9001,
        debug: on,
    },
}
```

---

## Rust Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
mon-core = "0.1"
```

Parse and analyze:

```rust
use mon_core::analyze;

fn main() {
    let text = r#"
        settings: {
            name: "Example",
            enabled: on,
        }
    "#;

    match analyze(text, "config.mon") {
        Ok(result) => {
            println!("JSON:\n{}", result.to_json().unwrap());
        }
        Err(err) => {
            eprintln!("MON error:\n{err}");
        }
    }
}
```

---

## Error Handling

MON is designed to fail loudly **and** helpfully.

Example error (format depends on your terminal capabilities):

```
error[E0012]: expected Number, got String
  --> config.mon:7:12
   |
 6 |   age: "twenty",
   |             ^^^ expected a Number here
```

Errors include:

* the source span
* the inferred and expected types
* suggestions when applicable

---

## Development

Build:

```bash
cargo build
```

Test:

```bash
cargo test --all-features
```

Checks:

```bash
cargo check
```

The project follows standard Rust layout.
Documentation lives in `docs/`. Any language or spec changes must be reflected there.

---

## Roadmap

* Improved parser recovery modes
* Type system stabilization
* Performance pass on alias/anchor resolution
* Better import graph validation
* Tooling support (formatter, LSP)

---

## License

Licensed under the MIT license.
See [`LICENSE`](./LICENSE) for details.


---
> for more information please look at out website: www.mycel-lang.org


Made with ❤️ 