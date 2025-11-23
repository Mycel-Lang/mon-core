#![doc = include_str!("../README.md")]

//! # Crate Overview
//! 
//! `mon-core` is the foundational engine for the Mycel Object Notation (MON) language. It provides a complete pipeline
//! for parsing, analyzing, and serializing MON data, designed to be fast, robust, and developer-friendly. This crate
//! is the backbone of the Mycel ecosystem, powering tools like linters, language servers, and formatters.
//!
//! ## Architectural Overview
//! 
//! The library follows a standard compiler-front-end architecture, processing MON source code in several distinct stages:
//! 
//! 1.  **Lexical Analysis (`lexer`):** The source text is first fed into the [`lexer`], which scans the text and
//!     converts it into a stream of tokens (e.g., identifiers, keywords, symbols). This is the most basic
//!     level of tokenization.
//! 
//! 2.  **Parsing (`parser`):** The stream of tokens is then passed to the [`parser`], which enforces the MON
//!     language's grammar rules. It constructs an Abstract Syntax Tree (AST), a hierarchical representation
//!     of the code's structure, defined in the [`ast`] module.
//! 
//! 3.  **Semantic Analysis (`resolver`):** The raw AST is processed by the [`resolver`]. This is a crucial step
//!     where the meaning of the code is analyzed. The resolver handles imports, resolves aliases and spreads, 
//!     and validates data against type definitions (`#struct` and `#enum`).
//! 
//! 4.  **Public API (`api`):** The [`api`] module provides the primary, high-level interface for the library. 
//!     The [`analyze`] function wraps the entire Lexer -> Parser -> Resolver pipeline into a single, easy-to-use
//!     function, returning a fully resolved and validated result.
//! 
//! ## Use Cases
//! 
//! - **Configuration Loading:** Parse and validate `.mon` configuration files for an application.
//! - **Language Tooling:** Build linters, formatters, or a Language Server Protocol (LSP) implementation for MON.
//! - **Data Serialization:** Use the library to convert MON data into other formats like JSON or YAML.
//! 
//! ## Example: A Complete Analysis
//! 
//! This example shows how to use the high-level [`analyze`] function to process a MON string from start to finish.
//! 
//! ```rust
//! use mon_core::api::analyze;
//! 
//! # fn main() -> Result<(), mon_core::error::MonError> {
//! let source = r#" 
//! { 
//!     Config: #struct { port(Number) }, 
//!     my_config :: Config = { port: 8080 } 
//! } 
//! "#;
//! 
//! // The `analyze` function handles lexing, parsing, and resolving.
//! let result = analyze(source, "config.mon")?;
//! 
//! // You can serialize the validated data to JSON.
//! let json_output = result.to_json().unwrap();
//! println!("{}", json_output);
//! 
//! // The output will be a well-formatted JSON string.
//! assert!(json_output.contains("\"port\": 8080.0"));
//! # Ok(())
//! # }
//! ```
//! 
//! For more granular control over each stage of the process, you can use the components from the [`lexer`], [`parser`],
//! and [`resolver`] modules directly.

/// Provides the public API for interacting with the MON core library,
/// including parsing, analysis, and serialization functions.
pub mod api;
pub mod ast;
pub mod error;

pub mod lexer;
#[cfg(feature = "lsp")]
pub mod lsp;
#[cfg(feature = "lsp")]
pub mod utils;

pub mod parser;
pub mod resolver;
pub mod serialization;

pub use api::{analyze, AnalysisResult};