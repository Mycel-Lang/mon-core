use std::fmt::Display;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic, Clone)]
pub enum MonError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    #[source_code]
    Parser(#[from] ParserError),
}

#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Parser Error")]
pub enum ParserError {
    #[error("Unexpected token")]
    #[diagnostic(
        code(parser::unexpected_token),
        help("The parser found a token it did not expect in this position.")
    )]
    UnexpectedToken {
        #[source_code]
        src: NamedSource<String>,
        #[label("Expected {expected}, but found this")]
        span: SourceSpan,
        expected: String,
    },

    #[error("Unexpected end of file")]
    #[diagnostic(
        code(parser::unexpected_eof),
        help("The file ended unexpectedly. The parser expected more tokens.")
    )]
    UnexpectedEof {
        #[source_code]
        src: NamedSource<String>,
        #[label("File ended unexpectedly here")]
        span: SourceSpan,
    },

    #[error("Missing expected token")]
    #[diagnostic(
        code(parser::missing_expected_token),
        help("The parser expected a specific token that was not found.")
    )]
    MissingExpectedToken {
        #[source_code]
        src: NamedSource<String>,
        #[label("Expected {expected} here")]
        span: SourceSpan,
        expected: String,
    },
}
