# MON (Mycel Object Notation) Core

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

## Development Conventions

* **Code Style:** Follow standard Rust conventions and formatting (`rustfmt`).
* **Testing:** Unit tests are located within the source files under a `#[cfg(test)]` module and in the `tests/`
  directory. Add tests for any new or modified functionality.
* **Documentation:** The `docs` directory contains the specification and user guide for the MON language. Changes to the
  language syntax or features should be reflected in these documents.