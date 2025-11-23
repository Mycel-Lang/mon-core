//! # MON Parser
//!
//! This module provides the `Parser` for the MON language. Its primary responsibility
//! is to transform a linear sequence of tokens from the [`lexer`](crate::lexer) into a
//! hierarchical Abstract Syntax Tree (AST), as defined in the [`ast`](crate::ast) module.
//!
//! ## Architectural Overview
//!
//! The `Parser` is a recursive descent parser. This parsing strategy uses a set of mutually
//! recursive functions to process the token stream, with each function typically corresponding
//! to a non-terminal symbol in the MON grammar. For example, `parse_object()`, `parse_array()`,
//! and `parse_member()` each handle a specific part of the language syntax.
//!
//! The parser's entry point is [`Parser::parse_document`], which orchestrates the parsing of
//! the entire document, including any top-level import statements and the root object.
//!
//! The parser does **not** perform semantic validation. It only checks for syntactic
//! correctness. For example, it will successfully parse `value: *non_existent_anchor`, but the
//! [`resolver`](crate::resolver) will later flag an error because the anchor does not exist.
//!
//! ## Use Cases
//!
//! Direct interaction with the parser is less common than using the top-level [`analyze`](crate::api::analyze)
//! function. However, it can be useful for:
//!
//! - **Syntax Tree Inspection:** Building tools that need to analyze the raw structure of a MON
//!   file without performing full semantic analysis.
//! - **Custom Analysis Pipelines:** Creating a custom analysis process where the AST needs to be
//!   inspected or transformed before being passed to the resolver.
//!
//! ## Example: Direct Parser Usage
//!
//! ```rust
//! use mon_core::parser::Parser;
//! use mon_core::error::MonError;
//!
//! # fn main() -> Result<(), MonError> {
//! let source = r#"
//! {
//!     // This is a syntactically correct MON file.
//!     key: "value",
//!     nested: { flag: on }
//! }
//! "#;
//!
//! // 1. Create a new parser for the source code.
//! let mut parser = Parser::new_with_name(source, "my_file.mon".to_string())?;
//!
//! // 2. Parse the source into a document (AST).
//! let document = parser.parse_document()?;
//!
//! // The `document` can now be inspected.
//! assert!(document.imports.is_empty());
//! // Further processing would be needed to make sense of the values.
//!
//! # Ok(())
//! # }
//! ```
use crate::ast::{
    EnumDef, FieldDef, ImportSpec, ImportSpecifier, ImportStatement, Member, MonDocument, MonValue,
    MonValueKind, Pair, StructDef, TypeDef, TypeDefinition, TypeSpec,
};
use crate::error::{MonError, ParserError};
use crate::lexer::{Lexer, Token, TokenType};
use miette::{GraphicalReportHandler, NamedSource, Report};
use std::panic::Location;
use std::sync::Arc;

/// A recursive descent parser for the MON language.
///
/// The `Parser` takes a stream of tokens from a [`Lexer`] and produces an
/// Abstract Syntax Tree (AST), represented by a [`MonDocument`]. It is responsible
/// for enforcing the grammatical structure of the MON language but does not perform
/// semantic analysis like validation or alias resolution (see [`crate::resolver::Resolver`]).
///
/// The main entry point is [`Parser::parse_document`], which parses the entire source.
///
/// # Example: How to use the Parser
///
/// You can use the `Parser` directly to get the raw AST of a MON file.
///
/// ```rust
/// use mon_core::parser::Parser;
/// use mon_core::error::MonError;
///
/// # fn main() -> Result<(), MonError> {
/// let source = r#"
/// import { MyType } from "./types.mon"
///
/// {
///     &my_anchor: { a: 1 },
///     value :: MyType = *my_anchor
/// }
/// "#;
///
/// // 1. Create a new parser for the source code.
/// let mut parser = Parser::new_with_name(source, "my_file.mon".to_string())?;
///
/// // 2. Parse the source into a document.
/// let document = parser.parse_document()?;
///
/// // The `document` now contains the raw AST, including imports and the unresolved root object.
/// assert_eq!(document.imports.len(), 1);
/// // Further processing would be needed by the resolver to handle the alias and validation.
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Parser<'a> {
    source: Arc<NamedSource<String>>,
    tokens: Vec<Token>,
    position: usize,
    source_text: &'a str,
}

impl<'a> Parser<'a> {
    /// Creates a new `Parser` instance with a default file name "source.mon".
    ///
    /// This is a convenience method that calls [`Parser::new_with_name`].
    ///
    /// # Arguments
    ///
    /// * `source_text` - The MON source code as a string.
    ///
    /// # Errors
    ///
    /// Returns a [`MonError`] if lexing the source text fails.
    pub fn new(source_text: &'a str) -> Result<Self, MonError> {
        Self::new_with_name(source_text, "source.mon".to_string())
    }

    /// Creates a new `Parser` instance with a specified file name.
    ///
    /// The parser initializes a [`Lexer`] and filters out whitespace and comments.
    ///
    /// # Arguments
    ///
    /// * `source_text` - The MON source code as a string.
    /// * `name` - The name of the file being parsed, used for error reporting.
    ///
    /// # Errors
    ///
    /// Returns a [`MonError`] if lexing the source text fails.
    pub fn new_with_name(source_text: &'a str, name: String) -> Result<Self, MonError> {
        let source = Arc::new(NamedSource::new(name, source_text.to_string()));
        let mut lexer = Lexer::new(source_text);
        let tokens: Vec<Token> = lexer
            .lex()
            .into_iter()
            .filter(|t| !matches!(t.ttype, TokenType::Whitespace | TokenType::Comment(_)))
            .collect();

        Ok(Self {
            source,
            tokens,
            position: 0,
            source_text,
        })
    }

    // === Main Parsing Methods ===

    /// Parses the entire MON source into a [`MonDocument`].
    ///
    /// This method parses import statements first, followed by the root object,
    /// and ensures no unexpected tokens are left at the end of the file.
    ///
    /// # Errors
    ///
    /// Returns a [`MonError`] if parsing fails at any point.
    pub fn parse_document(&mut self) -> Result<MonDocument, MonError> {
        let mut imports: Vec<ImportStatement> = Vec::new();

        // consume zero-or-more import statements
        while self.check(&TokenType::Import) {
            let imp = self.parse_import_statement()?;
            imports.push(imp);
        }

        // After imports, we expect the root object.
        let root = self.parse_object()?;

        // After the root object, we expect the end of the file.
        self.expect(&TokenType::Eof)?;
        Ok(MonDocument { root, imports })
    }

    /// Object ::= "{" [ `MemberList` ] "}"
    /// `MemberList` ::= `Member { , Member } [ , ]`
    fn parse_object(&mut self) -> Result<MonValue, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(&TokenType::LBrace)?;
        let mut members = Vec::new();
        if !self.check(&TokenType::RBrace) {
            // Parse the first member
            members.push(self.parse_member()?);
            // Keep parsing members as long as they are preceded by a comma
            while self.match_token(&TokenType::Comma) {
                // If we match a comma but the next token is a brace, it's a trailing comma
                if self.check(&TokenType::RBrace) {
                    break;
                }
                members.push(self.parse_member()?);
            }
        }
        let end_token = self.current_token()?.clone();
        self.expect(&TokenType::RBrace)?;
        Ok(MonValue {
            kind: MonValueKind::Object(members),
            anchor: None, // Anchors are attached to values, not objects themselves
            pos_start: start_token.pos_start,
            pos_end: end_token.pos_end,
        })
    }

    /// Array ::= "[" [ `ValueList` ] "]"
    /// `ValueList` ::= `Value { , Value } [ , ]`
    fn parse_array(&mut self) -> Result<MonValue, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(&TokenType::LBracket)?;
        let mut values = Vec::new();
        if !self.check(&TokenType::RBracket) {
            loop {
                if self.check(&TokenType::Spread) {
                    let spread_start_token = self.current_token()?.clone();
                    let spread_name = self.parse_spread()?;
                    let spread_end_token = self.current_token_before_advance()?.clone(); // Get token before advance
                    values.push(MonValue {
                        kind: MonValueKind::ArraySpread(spread_name),
                        anchor: None,
                        pos_start: spread_start_token.pos_start,
                        pos_end: spread_end_token.pos_end,
                    });
                } else {
                    values.push(self.parse_value()?);
                }

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
                if self.check(&TokenType::RBracket) {
                    break; // Allow trailing comma
                }
            }
        }
        let end_token = self.current_token()?.clone();
        self.expect(&TokenType::RBracket)?;
        Ok(MonValue {
            kind: MonValueKind::Array(values),
            anchor: None,
            pos_start: start_token.pos_start,
            pos_end: end_token.pos_end,
        })
    }

    /// Value ::= Object | Array | Alias | `EnumValue` | Literal
    /// Attaches an anchor if one is present.
    fn parse_value(&mut self) -> Result<MonValue, MonError> {
        let anchor = self.parse_optional_anchor()?;

        let start_token = self.current_token()?.clone(); // Capture start token for pos_start

        let mut value = match &start_token.ttype.clone() {
            // Use start_token here
            TokenType::LBrace => self.parse_object(),
            TokenType::LBracket => self.parse_array(),
            TokenType::String(s) => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::String(s.clone()),
                    anchor: None,
                    pos_start: start_token.pos_start,
                    pos_end: start_token.pos_end,
                })
            }
            TokenType::Number(n) => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::Number(*n),
                    anchor: None,
                    pos_start: start_token.pos_start,
                    pos_end: start_token.pos_end,
                })
            }
            TokenType::True => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::Boolean(true),
                    anchor: None,
                    pos_start: start_token.pos_start,
                    pos_end: start_token.pos_end,
                })
            }
            TokenType::False => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::Boolean(false),
                    anchor: None,
                    pos_start: start_token.pos_start,
                    pos_end: start_token.pos_end,
                })
            }
            TokenType::Null => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::Null,
                    anchor: None,
                    pos_start: start_token.pos_start,
                    pos_end: start_token.pos_end,
                })
            }
            TokenType::Asterisk => self.parse_alias(),
            TokenType::Dollar => self.parse_enum_value(),
            _ => self.err_unexpected("a value"),
        }?;

        value.anchor = anchor;
        Ok(value)
    }

    /// Member ::= Pair | `TypeDefinition` | Spread
    fn parse_member(&mut self) -> Result<Member, MonError> {
        match self.current_token()?.ttype {
            TokenType::Spread => self.parse_spread().map(Member::Spread),
            // A TypeDefinition starts with an Identifier followed by a Colon and a Hash
            TokenType::Identifier(_)
                if self.peek_is(&TokenType::Colon) && self.peek_next_is(&TokenType::Hash) =>
            {
                self.parse_type_definition().map(Member::TypeDefinition)
            }
            // Otherwise, it's a regular pair
            _ => self.parse_pair().map(Member::Pair),
        }
    }

    /// Pair ::= `KeyPart` [ Validation ] ( ":" | "=" ) Value
    /// `KeyPart` ::= [ Anchor ] Key
    /// Key ::= Identifier | String
    fn parse_pair(&mut self) -> Result<Pair, MonError> {
        let mut anchor_from_key: Option<String> = None;

        // Handle the case where the key itself is an anchor, e.g., `&my_anchor: value`
        let key = if self.match_token(&TokenType::Ampersand) {
            let key_name = self.parse_key()?;
            anchor_from_key = Some(key_name.clone());
            key_name
        } else {
            self.parse_key()?
        };

        let validation = self.parse_optional_validation()?;

        if !self.match_token(&TokenType::Colon) && !self.match_token(&TokenType::Equals) {
            return self.err_unexpected("':' or '=' after key");
        }

        let mut value = self.parse_value()?;

        // If the key was an anchor, attach the anchor to the value.
        // This handles `&anchor: value`.
        // The `parse_value` function handles the `key: &anchor value` case on its own.
        if let Some(anchor_name) = anchor_from_key {
            value.anchor = Some(anchor_name);
        }

        Ok(Pair {
            key,
            value,
            validation,
        })
    }

    // === EBNF Sub-Rules ===

    /// Key ::= Identifier | String
    fn parse_key(&mut self) -> Result<String, MonError> {
        let token = self.current_token()?;
        let mut key_parts = Vec::new();

        match &token.ttype {
            TokenType::Identifier(s) | TokenType::String(s) => {
                key_parts.push(s.clone());
                self.advance();
            }
            _ => return self.err_unexpected("an identifier or string for a key"),
        }

        // Handle dotted keys like `schemas.User`
        while self.match_token(&TokenType::Dot) {
            let token = self.current_token()?;
            if let TokenType::Identifier(s) = &token.ttype {
                key_parts.push(s.clone());
                self.advance();
            } else {
                return self.err_unexpected("an identifier after a dot in a key");
            }
        }

        Ok(key_parts.join("."))
    }

    /// Anchor ::= "&" Identifier
    fn parse_optional_anchor(&mut self) -> Result<Option<String>, MonError> {
        if self.match_token(&TokenType::Ampersand) {
            let token = self.current_token()?;
            if let TokenType::Identifier(name) = &token.ttype {
                let name = name.clone();
                self.advance();
                Ok(Some(name))
            } else {
                self.err_unexpected("an identifier for the anchor name")
            }
        } else {
            Ok(None)
        }
    }

    /// Alias ::= "*" Identifier { "." Identifier }
    fn parse_alias(&mut self) -> Result<MonValue, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(&TokenType::Asterisk)?;
        let mut name = self.parse_key()?;
        let mut end_pos = self.current_token_before_advance()?.pos_end; // End of the first key part

        while self.match_token(&TokenType::Dot) {
            name.push('.');
            let key_part = self.parse_key()?;
            end_pos = self.current_token_before_advance()?.pos_end; // Update end_pos
            name.push_str(&key_part);
        }
        Ok(MonValue {
            kind: MonValueKind::Alias(name),
            anchor: None,
            pos_start: start_token.pos_start,
            pos_end: end_pos,
        })
    }

    /// Spread ::= "..." Alias
    fn parse_spread(&mut self) -> Result<String, MonError> {
        self.expect(&TokenType::Spread)?;
        let alias = self.parse_alias()?;
        if let MonValueKind::Alias(name) = alias.kind {
            Ok(name)
        } else {
            // This should be unreachable if parse_alias is correct
            self.err_unexpected("an alias after '...' ")
        }
    }

    /// `ImportStatement` ::= "import" ( `NamespaceImport` | `NamedImport` ) "from" String
    fn parse_import_statement(&mut self) -> Result<ImportStatement, MonError> {
        let start_token = self.current_token()?.clone(); // Capture start token for pos_start
        self.expect(&TokenType::Import)?;

        let spec = if self.match_token(&TokenType::Asterisk) {
            // NamespaceImport ::= "*" "as" Identifier
            self.expect(&TokenType::As)?;
            let name = self.parse_key()?;
            ImportSpec::Namespace(name)
        } else {
            // NamedImport ::= "{" [ ImportSpecifier { "," ImportSpecifier } [ "," ] ] "}"
            self.expect(&TokenType::LBrace)?;
            let mut specifiers = Vec::new();
            if !self.check(&TokenType::RBrace) {
                loop {
                    // ImportSpecifier ::= [ "&" ] Identifier
                    let is_anchor = self.match_token(&TokenType::Ampersand);
                    let name = self.parse_key()?;
                    specifiers.push(ImportSpecifier { name, is_anchor });
                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                    if self.check(&TokenType::RBrace) {
                        break;
                    }
                }
            }
            self.expect(&TokenType::RBrace)?;
            ImportSpec::Named(specifiers)
        };

        self.expect(&TokenType::From)?;
        let path_token = self.current_token()?.clone(); // Capture path token for pos_end
        let path = self.parse_key()?;

        Ok(ImportStatement {
            path,
            spec,
            pos_start: start_token.pos_start,
            pos_end: path_token.pos_end,
        })
    }

    /// `TypeDefinition` ::= Identifier ":" ( `StructDefinition` | `EnumDefinition` )
    fn parse_type_definition(&mut self) -> Result<TypeDefinition, MonError> {
        let name_token = self.current_token()?.clone();
        let name = self.parse_key()?;
        self.expect(&TokenType::Colon)?;
        let hash_token = self.current_token()?.clone();
        self.expect(&TokenType::Hash)?;

        let token = self.current_token()?;
        let (def_type, end_pos) = match &token.ttype {
            TokenType::Identifier(s) if s == "struct" => {
                self.advance();
                let mut struct_def = self.parse_struct_definition()?;
                let end_pos = struct_def.pos_end;
                struct_def.pos_start = hash_token.pos_start;
                Ok((TypeDef::Struct(struct_def), end_pos))
            }
            TokenType::Identifier(s) if s == "enum" => {
                self.advance();
                let mut enum_def = self.parse_enum_definition()?;
                let end_pos = enum_def.pos_end;
                enum_def.pos_start = hash_token.pos_start;
                Ok((TypeDef::Enum(enum_def), end_pos))
            }
            _ => self.err_unexpected("'struct' or 'enum' keyword"),
        }?;

        Ok(TypeDefinition {
            name,
            name_span: (
                name_token.pos_start,
                name_token.pos_end - name_token.pos_start,
            )
                .into(),
            def_type,
            pos_start: name_token.pos_start,
            pos_end: end_pos,
        })
    }

    /// `StructDefinition` ::= "{" [ `FieldList` ] "}"
    fn parse_struct_definition(&mut self) -> Result<StructDef, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(&TokenType::LBrace)?;
        let mut fields = Vec::new();
        if !self.check(&TokenType::RBrace) {
            loop {
                fields.push(self.parse_field_definition()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
                if self.check(&TokenType::RBrace) {
                    break;
                }
            }
        }
        let end_token = self.current_token()?.clone();
        self.expect(&TokenType::RBrace)?;
        Ok(StructDef {
            fields,
            pos_start: start_token.pos_start,
            pos_end: end_token.pos_end,
        })
    }

    /// `FieldDefinition` ::= Identifier "(" Type ")" [ "=" Value ]
    fn parse_field_definition(&mut self) -> Result<FieldDef, MonError> {
        let name = self.parse_key()?;
        self.expect(&TokenType::LParen)?;
        let type_spec = self.parse_type_spec()?;
        self.expect(&TokenType::RParen)?;

        let default_value = if self.match_token(&TokenType::Equals) {
            Some(self.parse_value()?)
        } else {
            None
        };

        Ok(FieldDef {
            name,
            type_spec,
            default_value,
        })
    }

    /// `EnumDefinition` ::= `{ [ Identifier { , Identifier } [ , ] ] }`
    fn parse_enum_definition(&mut self) -> Result<EnumDef, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(&TokenType::LBrace)?;
        let mut variants = Vec::new();
        if !self.check(&TokenType::RBrace) {
            loop {
                variants.push(self.parse_key()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
                if self.check(&TokenType::RBrace) {
                    break;
                }
            }
        }
        let end_token = self.current_token()?.clone();
        self.expect(&TokenType::RBrace)?;
        Ok(EnumDef {
            variants,
            pos_start: start_token.pos_start,
            pos_end: end_token.pos_end,
        })
    }

    /// Validation ::= "::" Type
    fn parse_optional_validation(&mut self) -> Result<Option<TypeSpec>, MonError> {
        if self.match_token(&TokenType::DoubleColon) {
            self.parse_type_spec().map(Some)
        } else {
            Ok(None)
        }
    }

    /// Type ::= `CollectionType` | Identifier | "String" | ...
    fn parse_type_spec(&mut self) -> Result<TypeSpec, MonError> {
        let start_token = self.current_token()?.clone();
        if self.check(&TokenType::LBracket) {
            // CollectionType ::= "[" Type [ "..." ] { "," Type [ "..." ] } "]"
            self.expect(&TokenType::LBracket)?;
            let mut types = Vec::new();
            if !self.check(&TokenType::RBracket) {
                loop {
                    let mut type_spec = self.parse_type_spec()?;
                    if self.match_token(&TokenType::Spread) {
                        let end_token = self.current_token_before_advance()?.clone();
                        let span = (
                            type_spec.get_span().offset(),
                            end_token.pos_end - type_spec.get_span().offset(),
                        )
                            .into();
                        type_spec = TypeSpec::Spread(Box::new(type_spec), span);
                    }
                    types.push(type_spec);

                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                    if self.check(&TokenType::RBracket) {
                        break;
                    }
                }
            }
            let end_token = self.current_token()?.clone();
            self.expect(&TokenType::RBracket)?;
            let span = (
                start_token.pos_start,
                end_token.pos_end - start_token.pos_start,
            )
                .into();
            Ok(TypeSpec::Collection(types, span))
        } else {
            // Simple Type
            let name = self.parse_key()?;
            let end_token = self.current_token_before_advance()?.clone();
            let span = (
                start_token.pos_start,
                end_token.pos_end - start_token.pos_start,
            )
                .into();
            Ok(TypeSpec::Simple(name, span))
        }
    }

    /// `EnumValue` ::= "$" Identifier "." Identifier
    fn parse_enum_value(&mut self) -> Result<MonValue, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(&TokenType::Dollar)?;

        // parse enum name as a single Identifier
        let enum_token = self.current_token()?.clone();
        let enum_name = if let TokenType::Identifier(s) = &enum_token.ttype {
            let s = s.clone();
            self.advance();
            s
        } else {
            return self.err_unexpected("an identifier for enum name");
        };

        self.expect(&TokenType::Dot)?;

        // parse variant name as a single Identifier
        let variant_token = self.current_token()?.clone();
        let variant_name = if let TokenType::Identifier(s) = &variant_token.ttype {
            let s = s.clone();
            self.advance();
            s
        } else {
            return self.err_unexpected("an identifier for enum variant");
        };

        Ok(MonValue {
            kind: MonValueKind::EnumValue {
                enum_name,
                variant_name,
            },
            anchor: None,
            pos_start: start_token.pos_start,
            pos_end: variant_token.pos_end,
        })
    }

    // === Tokenizer Helper Methods ===

    fn current_token(&self) -> Result<&Token, MonError> {
        self.tokens.get(self.position).ok_or_else(|| {
            let pos = self.source_text.len().saturating_sub(1);
            ParserError::UnexpectedEof {
                src: (*self.source).clone().into(), // ineficiency is my passion
                span: (pos, 0).into(),
            }
            .into()
        })
    }

    fn current_token_before_advance(&self) -> Result<&Token, MonError> {
        self.tokens
            .get(self.position.saturating_sub(1))
            .ok_or_else(|| {
                let pos = self.source_text.len().saturating_sub(1);
                ParserError::UnexpectedEof {
                    src: (*self.source).clone().into(),
                    span: (pos, 0).into(),
                }
                .into()
            })
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    #[track_caller]
    fn expect(&mut self, expected: &TokenType) -> Result<(), MonError> {
        let token = self.current_token()?.clone();
        if std::mem::discriminant(&token.ttype) == std::mem::discriminant(expected) {
            self.advance();
            Ok(())
        } else {
            self.err_unexpected(&format!("{expected:?}"))
        }
    }

    fn match_token(&mut self, ttype: &TokenType) -> bool {
        if self.check(ttype) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, ttype: &TokenType) -> bool {
        if let Ok(token) = self.current_token() {
            std::mem::discriminant(&token.ttype) == std::mem::discriminant(ttype)
        } else {
            false
        }
    }

    fn peek_is(&self, ttype: &TokenType) -> bool {
        if let Some(token) = self.tokens.get(self.position + 1) {
            std::mem::discriminant(&token.ttype) == std::mem::discriminant(ttype)
        } else {
            false
        }
    }

    fn peek_next_is(&self, ttype: &TokenType) -> bool {
        if let Some(token) = self.tokens.get(self.position + 2) {
            std::mem::discriminant(&token.ttype) == std::mem::discriminant(ttype)
        } else {
            false
        }
    }

    #[track_caller]
    fn err_unexpected<T>(&self, expected: &str) -> Result<T, MonError> {
        let token = self.current_token()?;
        print!("caller: {}", Location::caller());
        Err(ParserError::UnexpectedToken {
            src: (*self.source).clone().into(),
            span: (token.pos_start, token.pos_end - token.pos_start).into(),
            expected: expected.to_string(),
        }
        .into())
    }
}

// internal debug function. I really can't stand bad strings
#[allow(dead_code)]
fn pretty_result(out: Result<MonDocument, MonError>) -> String {
    match out {
        Ok(doc) => format!("{doc:#?}"), // debug format for success
        Err(err) => {
            let report: Report = Report::new(err);
            let handler = GraphicalReportHandler::new(); // pretty ANSI colors
            let mut buffer = String::new();
            handler.render_report(&mut buffer, &*report).unwrap();
            buffer
        }
    }
}

#[cfg(test)]
#[allow(clippy::match_wildcard_for_single_variants)]
mod tests {
    use super::*;
    use miette::Report;

    impl Member {
        fn unwrap_pair(self) -> Pair {
            match self {
                Member::Pair(p) => p,
                _ => panic!("Expected Pair, got {self:?}"),
            }
        }
        fn unwrap_type_definition(self) -> TypeDefinition {
            match self {
                Member::TypeDefinition(td) => td,
                _ => panic!("Expected TypeDefinition, got {self:?}"),
            }
        }
    }

    impl MonValueKind {
        fn unwrap_object(self) -> Vec<Member> {
            match self {
                MonValueKind::Object(m) => m,
                _ => panic!("Expected Object, got {self:?}"),
            }
        }
        #[allow(dead_code)]
        fn unwrap_array(self) -> Vec<MonValue> {
            match self {
                MonValueKind::Array(v) => v,
                _ => panic!("Expected Array, got {self:?}"),
            }
        }
    }

    fn parse_ok(source: &str) -> MonDocument {
        let mut parser = Parser::new_with_name(source, "test.mon".to_string()).unwrap();
        match parser.parse_document() {
            Ok(doc) => doc,
            Err(err) => {
                let report = Report::from(err);
                panic!("Parsing failed when it should have succeeded:\n{report:?}");
            }
        }
    }

    #[test]
    fn test_empty_object() {
        let doc = parse_ok("{}");
        assert_eq!(doc.root.kind, MonValueKind::Object(vec![]));
    }

    #[test]
    fn test_simple_pair() {
        let doc = parse_ok(r#"{ key: "value" }"#);
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 1);
        match &members[0] {
            Member::Pair(p) => {
                assert_eq!(p.key, "key");
                assert!(matches!(p.value.kind, MonValueKind::String(_)));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_anchor_and_alias() {
        let doc = parse_ok(r#"{ &anchor1 : 123, key2: *anchor1 }"#);
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 2);
        match &members[0] {
            Member::Pair(p) => {
                assert_eq!(p.key, "anchor1");
                assert_eq!(p.value.anchor, Some("anchor1".to_string()));
            }
            _ => panic!(),
        }
        match &members[1] {
            Member::Pair(p) => {
                assert_eq!(p.key, "key2");
                assert!(matches!(p.value.kind, MonValueKind::Alias(_)));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_spread() {
        let doc = parse_ok(r#"{ ...*my_anchor }"#);
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 1);
        match &members[0] {
            Member::Spread(name) => assert_eq!(name, "my_anchor"),
            _ => panic!(),
        }
    }

    #[test]
    fn test_namespace_import() {
        let doc = parse_ok(
            r###"import * as schemas from "./schemas.mon"
{
    a: 1
}"###,
        );
        assert!(!doc.imports.is_empty());
        match &doc.imports[0].spec {
            ImportSpec::Namespace(name) => assert_eq!(name, "schemas"),
            _ => panic!("Expected namespace import"),
        }
    }

    #[test]
    fn test_trailing_comma_in_object() {
        let doc = parse_ok("{ a: 1, b: 2, }");
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 2);
    }

    #[test]
    fn test_trailing_comma_in_array() {
        let doc = parse_ok("{arr: [ 1, 2, ]}");
        let members = doc.root.kind.unwrap_object();
        let pair = members[0].clone().unwrap_pair();
        let values = match pair.value.kind {
            MonValueKind::Array(v) => v,
            _ => panic!("Expected Array"),
        };
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_array_with_spread() {
        let doc = parse_ok("{arr: [ 1, ...*other, 3 ]}");
        let members = doc.root.kind.unwrap_object();
        let pair = members[0].clone().unwrap_pair();
        let values = match pair.value.kind {
            MonValueKind::Array(v) => v,
            _ => panic!("Expected Array"),
        };
        assert_eq!(values.len(), 3);
        assert!(matches!(values[0].kind, MonValueKind::Number(_)));
        assert!(matches!(values[1].kind, MonValueKind::ArraySpread(_)));
        assert!(matches!(values[2].kind, MonValueKind::Number(_)));
    }

    #[test]
    fn test_all_value_types() {
        let doc = parse_ok(
            r#"{ 
            s: "string",
            n: 123.45,
            b1: true,
            b2: false,
            nu: null,
            obj: {},
            arr: [],
            alias: *somewhere,
            enum_val: $MyEnum.Variant
        }"#,
        );
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 9);
        assert!(matches!(
            members[0].clone().unwrap_pair().value.kind,
            MonValueKind::String(_)
        ));
        assert!(matches!(
            members[1].clone().unwrap_pair().value.kind,
            MonValueKind::Number(_)
        ));
        assert!(matches!(
            members[2].clone().unwrap_pair().value.kind,
            MonValueKind::Boolean(true)
        ));
        assert!(matches!(
            members[3].clone().unwrap_pair().value.kind,
            MonValueKind::Boolean(false)
        ));
        assert!(matches!(
            members[4].clone().unwrap_pair().value.kind,
            MonValueKind::Null
        ));
        assert!(matches!(
            members[5].clone().unwrap_pair().value.kind,
            MonValueKind::Object(_)
        ));
        assert!(matches!(
            members[6].clone().unwrap_pair().value.kind,
            MonValueKind::Array(_)
        ));
        assert!(matches!(
            members[7].clone().unwrap_pair().value.kind,
            MonValueKind::Alias(_)
        ));
        assert!(matches!(
            members[8].clone().unwrap_pair().value.kind,
            MonValueKind::EnumValue { .. }
        ));
    }

    #[test]
    fn test_dotted_key() {
        let doc = parse_ok(r#"{ a.b.c: 1 }"#);
        let pair = doc.root.kind.unwrap_object().remove(0).unwrap_pair();
        assert_eq!(pair.key, "a.b.c");
    }

    #[test]
    fn test_string_key() {
        let doc = parse_ok(r#"{ "a-b-c": 1 }"#);
        let pair = doc.root.kind.unwrap_object().remove(0).unwrap_pair();
        assert_eq!(pair.key, "a-b-c");
    }

    #[test]
    fn test_pair_with_equals() {
        let doc = parse_ok(r#"{ key = 1 }"#);
        let pair = doc.root.kind.unwrap_object().remove(0).unwrap_pair();
        assert_eq!(pair.key, "key");
        assert!(matches!(pair.value.kind, MonValueKind::Number(_)));
    }

    #[test]
    fn test_named_imports() {
        let doc = parse_ok(
            r#"import { A, &B, C } from "./types.mon"
{
    x: 1
}"#,
        );
        assert!(!doc.imports.is_empty());
        match &doc.imports[0].spec {
            ImportSpec::Named(specifiers) => {
                assert_eq!(specifiers.len(), 3);
                assert_eq!(specifiers[0].name, "A");
                assert!(!specifiers[0].is_anchor);
                assert_eq!(specifiers[1].name, "B");
                assert!(specifiers[1].is_anchor);
                assert_eq!(specifiers[2].name, "C");
                assert!(!specifiers[2].is_anchor);
            }
            _ => panic!("Expected named import"),
        }
    }

    #[test]
    fn test_struct_type_definition() {
        let doc = parse_ok(
            r#"{
User: #struct {
    name(String),
    age(Number) = 30,
}
}"#,
        );
        let members = doc.root.kind.unwrap_object();
        let td = members[0].clone().unwrap_type_definition();
        match td.def_type {
            TypeDef::Struct(s) => {
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "name");
                assert!(s.fields[0].default_value.is_none());
                assert_eq!(s.fields[1].name, "age");
                assert!(s.fields[1].default_value.is_some());
            }
            _ => panic!("Expected struct definition"),
        }
    }

    #[test]
    fn test_enum_type_definition() {
        let doc = parse_ok(
            r#"{
Status: #enum { Active, Inactive, Pending }
}"#,
        );
        let members = doc.root.kind.unwrap_object();
        let td = members[0].clone().unwrap_type_definition();
        match td.def_type {
            TypeDef::Enum(e) => {
                assert_eq!(e.variants, vec!["Active", "Inactive", "Pending"]);
            }
            _ => panic!("Expected enum definition"),
        }
    }

    #[test]
    fn test_validation_on_pair() {
        let doc = parse_ok(r#"{ key :: Number = 42 }"#);
        let pair = doc.root.kind.unwrap_object().remove(0).unwrap_pair();

        //                                                              should have been 69
        assert_eq!(
            pair.validation.unwrap(),
            TypeSpec::Simple("Number".into(), (9, 6).into())
        )
    }

    #[test]
    fn test_nested_objects_and_arrays() {
        let doc = parse_ok(r#"{ obj: { a: 1, b: 2 }, arr: [1, 2, 3] }"#);
        let members = doc.root.kind.unwrap_object();
        let obj_val = members[0].clone().unwrap_pair().value;
        assert!(matches!(obj_val.kind, MonValueKind::Object(_)));
        let arr_val = members[1].clone().unwrap_pair().value;
        assert!(matches!(arr_val.kind, MonValueKind::Array(_)));
    }

    #[test]
    fn test_enum_value_in_object() {
        let doc = parse_ok(r#"{ status: $Status.Active }"#);
        let pair = doc.root.kind.unwrap_object().remove(0).unwrap_pair();
        match pair.value.kind {
            MonValueKind::EnumValue {
                enum_name,
                variant_name,
            } => {
                assert_eq!(enum_name, "Status");
                assert_eq!(variant_name, "Active");
            }
            _ => panic!("Expected enum value"),
        }
    }

    #[test]
    fn test_complex_document() {
        let doc = parse_ok(
            r#"
import { &A, B } from "./types.mon"

{
    &anchor1: { x: 1 },
    key2: *anchor1,
    list: [1, 2, 3, ...*B],
    status: $Status.Active
}"#,
        );
        assert_eq!(doc.imports.len(), 1);
        let members = doc.root.kind.unwrap_object();
        assert_eq!(members.len(), 4);
    }
}
