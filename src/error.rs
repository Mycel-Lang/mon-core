//! # Error Handling in `mon-core`
//!
//! This module defines the error types used throughout the `mon-core` library. The error
//! handling strategy is built around the [`miette`] crate for rich, diagnostic-style
//! error reporting, and [`thiserror`] for ergonomic error type definitions.
//!
//! ## Architectural Overview
//!
//! The error system is hierarchical:
//! 
//! 1.  **[`MonError`]**: This is the top-level, public-facing error enum. It wraps more
//!     specific error types from different stages of the compilation pipeline.
//!     Any function in the public API will typically return a `Result<T, MonError>`.
//! 
//! 2.  **Phase-Specific Errors:**
//!     - [`ParserError`]: Errors that occur during the lexing or parsing phase, such as
//!       syntax errors (e.g., an unexpected token).
//!     - [`ResolverError`]: Errors that occur during the semantic analysis phase. This includes
//!       failures like an import not being found, an anchor being undefined, or a circular dependency.
//! 
//! 3.  **[`ValidationError`]**: A specialized subset of resolver errors that occur specifically
//!     during type validation. This includes type mismatches, missing or extra fields in structs,
//!     and undefined enum variants.
//! 
//! ## Use Cases
//! 
//! When you use the `mon-core` library, you will primarily interact with `MonError`. You can
//! match on its variants (`Parser` or `Resolver`) to determine the source of the failure.
//! 
//! The rich diagnostic information provided by `miette` allows for printing user-friendly,
//! colorful error reports that point directly to the problematic code in the source file.
//! 
//! ## Example: Handling an Error
//! 
//! ```rust
//! use mon_core::api::analyze;
//! 
//! // This source code has a syntax error (a missing comma).
//! let source = "{ key1: \"value1\" key2: \"value2\" }";
//! 
//! let result = analyze(source, "bad.mon");
//! 
//! match result {
//!     Ok(_) => println!("This should not have succeeded!"),
//!     Err(err) => {
//!         // You can render the beautiful, diagnostic error report.
//!         // (Note: The actual output is a graphical report with colors and source snippets)
//!         println!("{:?}", err);
//! 
//!         // You can also programmatically inspect the error.
//!         match err {
//!             mon_core::error::MonError::Parser(p_err) => {
//!                 println!("A parser error occurred!");
//!             },
//!             _ => {}
//!         }
//!     }
//! }
//! ```
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;

/// The primary error type for the `mon-core` library.
#[derive(Error, Debug, Diagnostic, Clone)]
pub enum MonError {
    /// An error that occurred during the parsing phase.
    #[error(transparent)]
    #[diagnostic(transparent)]
    #[source_code]
    Parser(Box<ParserError>),

    /// An error that occurred during the resolution or validation phase.
    #[error(transparent)]
    #[diagnostic(transparent)]
    #[source_code]
    Resolver(Box<ResolverError>),
}

impl From<ParserError> for MonError {
    fn from(err: ParserError) -> Self {
        MonError::Parser(Box::new(err))
    }
}

impl From<ResolverError> for MonError {
    fn from(err: ResolverError) -> Self {
        MonError::Resolver(Box::new(err))
    }
}

/// An error that occurred during the parsing phase.
#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Parser Error")]
pub enum ParserError {
    /// An unexpected token was found.
    #[error("Unexpected token")]
    #[diagnostic(
        code(parser::unexpected_token),
        help("The parser found a token it did not expect in this position.")
    )]
    UnexpectedToken {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("Expected {expected}, but found this")]
        span: SourceSpan,
        expected: String,
    },

    /// The end of the file was reached unexpectedly.
    #[error("Unexpected end of file")]
    #[diagnostic(
        code(parser::unexpected_eof),
        help("The file ended unexpectedly. The parser expected more tokens.")
    )]
    UnexpectedEof {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("File ended unexpectedly here")]
        span: SourceSpan,
    },

    /// A specific token was expected but not found.
    #[error("Missing expected token")]
    #[diagnostic(
        code(parser::missing_expected_token),
        help("The parser expected a specific token that was not found.")
    )]
    MissingExpectedToken {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("Expected {expected} here")]
        span: SourceSpan,
        expected: String,
    },
}
/// An error that occurred during the resolution or validation phase.
#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Resolver Error")]
pub enum ResolverError {
    /// An imported module could not be found.
    #[error("Module not found at path: {path}")]
    #[diagnostic(
        code(resolver::module_not_found),
        help("Check that the path is correct and the file exists.")
    )]
    ModuleNotFound {
        path: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("...referenced here")]
        span: SourceSpan,
    },

    /// An anchor referenced by an alias or spread could not be found.
    #[error("Anchor '&{name}' not found")]
    #[diagnostic(
        code(resolver::anchor_not_found),
        help("Ensure the anchor is defined with '&{name}: ...' in the correct scope.")
    )]
    AnchorNotFound {
        name: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("This anchor was not found")]
        span: SourceSpan,
    },

    /// The spread operator (`...*`) was used on a value that is not an object.
    #[error("Cannot spread a non-object value")]
    #[diagnostic(
        code(resolver::spread_on_non_object),
        help("The '...*' operator can only be used on an object anchor inside another object.")
    )]
    SpreadOnNonObject {
        name: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("'{name}' does not point to an object")]
        span: SourceSpan,
    },

    /// The spread operator (`...*`) was used on a value that is not an array.
    #[error("Cannot spread a non-array value")]
    #[diagnostic(
        code(resolver::spread_on_non_array),
        help("The '...*' operator can only be used on an array anchor inside another array.")
    )]
    SpreadOnNonArray {
        name: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("'{name}' does not point to an array")]
        span: SourceSpan,
    },

    /// A circular dependency was detected in module imports.
    #[error("Circular dependency detected")]
    #[diagnostic(
        code(resolver::circular_dependency),
        help("The following import chain forms a loop: {cycle}")
    )]
    CircularDependency {
        cycle: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("Importing this module creates a cycle")]
        span: SourceSpan,
    },

    /// An error occurred during data validation against a schema.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Validation(#[from] ValidationError),

    /// A parser error that occurred while resolving an imported module.
    #[error(transparent)]
    #[diagnostic(transparent)]
    WrappedParserError(Box<ParserError>),
}

impl From<ParserError> for ResolverError {
    fn from(err: ParserError) -> Self {
        ResolverError::WrappedParserError(Box::new(err))
    }
}

/// An error that occurred during data validation against a schema.
#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Validation Error")]
pub enum ValidationError {
    /// A field's value did not match its declared type.
    #[error("Type mismatch for field '{field_name}'. Expected type {expected_type} but got {found_type}.")]
    #[diagnostic(
        code(validation::type_mismatch),
        help(
            "Ensure the value's type matches the expected type in the struct or enum definition."
        )
    )]
    TypeMismatch {
        field_name: String,
        expected_type: String,
        found_type: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("Type mismatch here")]
        span: SourceSpan,
    },

    /// A required field was missing from a struct.
    #[error("Missing required field '{field_name}' for struct '{struct_name}'.")]
    #[diagnostic(
        code(validation::missing_field),
        help("Add the missing field or make it optional in the struct definition.")
    )]
    MissingField {
        field_name: String,
        struct_name: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("Field '{field_name}' is missing here")]
        span: SourceSpan,
    },

    /// An unexpected field was found in a struct.
    #[error("Found unexpected field '{field_name}' not defined in struct '{struct_name}'.")]
    #[diagnostic(
        code(validation::unexpected_field),
        help("Remove the unexpected field or add it to the struct definition.")
    )]
    UnexpectedField {
        field_name: String,
        struct_name: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("Unexpected field here")]
        span: SourceSpan,
    },

    /// A type name was used that has not been defined or imported.
    #[error("Undefined type '{type_name}'.")]
    #[diagnostic(
        code(validation::undefined_type),
        help("Ensure the type is in scope or imported correctly.")
    )]
    UndefinedType {
        type_name: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("Undefined type used here")]
        span: SourceSpan,
    },

    /// A variant was used that is not defined in the corresponding enum.
    #[error("Variant '{variant_name}' is not defined in enum '{enum_name}'.")]
    #[diagnostic(
        code(validation::undefined_enum_variant),
        help("Ensure the enum variant exists in the enum definition.")
    )]
    UndefinedEnumVariant {
        variant_name: String,
        enum_name: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("Undefined enum variant used here")]
        span: SourceSpan,
    },

    /// A complex collection type was used that is not yet supported by the validator.
    #[error("Complex collection type validation not yet implemented for field '{field_name}'.")]
    #[diagnostic(
        code(validation::unimplemented_collection_validation),
        help("This type of collection validation is not yet supported.")
    )]
    UnimplementedCollectionValidation {
        field_name: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("Complex collection type here")]
        span: SourceSpan,
    },
}

impl From<MonError> for ResolverError {
    fn from(err: MonError) -> Self {
        match err {
            MonError::Parser(p_err) => ResolverError::WrappedParserError(p_err),
            MonError::Resolver(r_err) => *r_err,
        }
    }
}