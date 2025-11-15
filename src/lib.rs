#[doc = include_str!("../README.md")]
/// Provides the public API for interacting with the MON core library,
/// including parsing, analysis, and serialization functions.
pub mod api;
pub mod ast;
pub mod error;

pub mod lexer;
#[cfg(feature = "lsp")]
pub(crate) mod lsp;
#[cfg(feature = "lsp")]
pub(crate) mod utils;

pub mod parser;
pub mod resolver;
pub mod serialization;

pub use api::{analyze, AnalysisResult};
