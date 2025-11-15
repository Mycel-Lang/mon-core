> MYCEL / MON is a hobby project. 

# MON (Mycel Object Notation) Core

[![Crates.io](https://img.shields.io/crates/v/mon-core.svg)](https://crates.io/crates/mon-core)
[![Docs.rs](https://docs.rs/mon-core/badge.svg)](https://docs.rs/mon-core)
[![License](https://img.shields.io/crates/l/mon-core.svg)](LICENSE)
[![CI](https://github.com/mycel-dot-org/mon/actions/workflows/rust.yml/badge.svg)](https://github.com/mycel-dot-org/mon/actions/workflows/rust.yml)

`mon-core` is the foundational Rust implementation for **MON (Mycel Object Notation)**, a human-friendly data notation
language designed for configuration, data exchange, and more. It prioritizes readability, structure, and reusability.

## Key Features

* **Human-Readable Syntax:** Clean, intuitive syntax with support for comments and unquoted keys.
* **Structured Data:** Organizes data in "labeled containers" (objects) with `key: value` pairs and arrays.
* **Data Reusability (DRY):** Features like **anchors (`&`)**, **aliases (`*`)**, and **spreads (`...*`)** to avoid
  repetition and promote modularity.
* **Robust Type System:**
    * **Type Definitions:** Supports custom `#struct` and `#enum` definitions.
    * **Validation:** Built-in type validation using `:: Type` annotations, ensuring data conforms to defined schemas.
    * **Collection Validation:** Advanced validation for arrays, including single-type, spread, tuple-like, and mixed
      collections.
* **Modularity & Imports:**
    * **Cross-File References:** Allows splitting data across multiple files using `import` statements.
    * **Namespaced Types:** Supports resolving and validating types defined in imported files, including namespaced
      references.
* **Rich Error Reporting:** Utilizes `miette` for clear, graphical diagnostics with source code snippets for parsing,
  resolution, and validation errors.

## Usage

To use `mon-core`, add it to your `Cargo.toml`:

```toml
[dependencies]
mon-core = "0.1.0" # Replace with the latest version
```

For a quick start, check out the [example](examples/simple.rs) in the `examples/` directory.

```rust
// examples/simple.rs
use mon_core::analyze;

    let mon_data = r#"
        user: {
            name: "John Doe",
            email: "john.doe@example.com"
        }
    "#;

    match analyze(mon_data, "example.mon") {
        Ok(result) => {
            let json_output = result.to_json().unwrap();
            println!("Successfully parsed MON to JSON:\n{}", json_output);
        }
        Err(e) => {
            eprintln!("Failed to parse MON: {:?}", e);
        }
    }
```

## Project Architecture (High-Level)

`mon-core` is built with a layered architecture to support advanced tooling like Language Server Protocol (LSP) and
compilers (`mycelc`). The core pipeline involves:

1. **Parser & AST/CST:** Transforms raw MON text into a syntax tree, designed to be error-tolerant for real-time
   feedback.
2. **Semantic Analyzer & Semantic Model:** Gives meaning to the syntax tree through validation, reference resolution (
   anchors, spreads, imports), and type checking.
3. **Query & Traversal API:** Provides an interface for external tools to navigate and inspect both the raw syntax tree
   and the fully resolved semantic model.

For a detailed breakdown of the architecture and future plans, please refer to `roadmap.md`.

## Building and Running

This is a standard Rust library project.

* **Build the project:**
  ```bash
  cargo build
  ```

* **Run tests:**
  ```bash
  cargo test
  ```

* **Check for compilation errors without building:**
  ```bash
  cargo check
  ```

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute,
report bugs, and suggest features. Adherence to our [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) is expected.

## Development Conventions

* **Code Style:** Follow standard Rust conventions and formatting (`rustfmt`).
* **Testing:** Unit tests are located within the source files under a `#[cfg(test)]` module and in the `tests/`
  directory. Add tests for any new or modified functionality.
* **Documentation:** The `docs` directory contains the specification and user guide for the MON language. Changes to the
  language syntax or features should be reflected in these documents.