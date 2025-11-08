use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic, Clone)]
pub enum MonError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    #[source_code]
    Parser(#[from] ParserError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    #[source_code]
    Resolver(#[from] ResolverError),
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
#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Resolver Error")]
pub enum ResolverError {
    #[error("Module not found at path: {path}")]
    #[diagnostic(
        code(resolver::module_not_found),
        help("Check that the path is correct and the file exists.")
    )]
    ModuleNotFound {
        path: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("...referenced here")]
        span: SourceSpan,
    },

    #[error("Anchor '&{name}' not found")]
    #[diagnostic(
        code(resolver::anchor_not_found),
        help("Ensure the anchor is defined with '&{name}: ...' in the correct scope.")
    )]
    AnchorNotFound {
        name: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("This anchor was not found")]
        span: SourceSpan,
    },

    #[error("Cannot spread a non-object value")]
    #[diagnostic(
        code(resolver::spread_on_non_object),
        help("The '...*' operator can only be used on an object anchor inside another object.")
    )]
    SpreadOnNonObject {
        name: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("'{name}' does not point to an object")]
        span: SourceSpan,
    },

    #[error("Cannot spread a non-array value")]
    #[diagnostic(
        code(resolver::spread_on_non_array),
        help("The '...*' operator can only be used on an array anchor inside another array.")
    )]
    SpreadOnNonArray {
        name: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("'{name}' does not point to an array")]
        span: SourceSpan,
    },

    #[error("Circular dependency detected")]
    #[diagnostic(
        code(resolver::circular_dependency),
        help("The following import chain forms a loop: {cycle}")
    )]
    CircularDependency {
        cycle: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("Importing this module creates a cycle")]
        span: SourceSpan,
    },

    #[error(transparent)]
    #[diagnostic(transparent)]
    Validation(#[from] ValidationError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    WrappedParserError(#[from] ParserError),
}

#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Validation Error")]
pub enum ValidationError {
    #[error("Type mismatch for field '{field_name}'. Expected type {expected_type} but got {found_type}.")]
    #[diagnostic(
        code(validation::type_mismatch),
        help("Ensure the value's type matches the expected type in the struct or enum definition.")
    )]
    TypeMismatch {
        field_name: String,
        expected_type: String,
        found_type: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("Type mismatch here")]
        span: SourceSpan,
    },

    #[error("Missing required field '{field_name}' for struct '{struct_name}'.")]
    #[diagnostic(
        code(validation::missing_field),
        help("Add the missing field or make it optional in the struct definition.")
    )]
    MissingField {
        field_name: String,
        struct_name: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("Field '{field_name}' is missing here")]
        span: SourceSpan,
    },

    #[error("Found unexpected field '{field_name}' not defined in struct '{struct_name}'.")]
    #[diagnostic(
        code(validation::unexpected_field),
        help("Remove the unexpected field or add it to the struct definition.")
    )]
    UnexpectedField {
        field_name: String,
        struct_name: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("Unexpected field here")]
        span: SourceSpan,
    },

    #[error("Undefined type '{type_name}'.")]
    #[diagnostic(
        code(validation::undefined_type),
        help("Ensure the type is defined or imported correctly.")
    )]
    UndefinedType {
        type_name: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("Undefined type used here")]
        span: SourceSpan,
    },

    #[error("Variant '{variant_name}' is not defined in enum '{enum_name}'.")]
    #[diagnostic(
        code(validation::undefined_enum_variant),
        help("Ensure the enum variant exists in the enum definition.")
    )]
    UndefinedEnumVariant {
        variant_name: String,
        enum_name: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("Undefined enum variant used here")]
        span: SourceSpan,
    },

    #[error("Complex collection type validation not yet implemented for field '{field_name}'.")]
    #[diagnostic(
        code(validation::unimplemented_collection_validation),
        help("This type of collection validation is not yet supported.")
    )]
    UnimplementedCollectionValidation {
        field_name: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("Complex collection type here")]
        span: SourceSpan,
    },
}

impl From<MonError> for ResolverError {
    fn from(err: MonError) -> Self {
        match err {
            MonError::Parser(p_err) => ResolverError::WrappedParserError(p_err),
            MonError::Resolver(r_err) => r_err,
        }
    }
}
