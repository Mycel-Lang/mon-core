use crate::ast::*;
use crate::error::{MonError, ParserError};
use crate::lexer::{Lexer, Token, TokenType};
use miette::{GraphicalReportHandler, NamedSource, Report};
use std::panic::Location;
use std::sync::Arc;

/// A recursive descent parser for the MON language, built according to the EBNF grammar.
#[derive(Debug)]
pub struct Parser<'a> {
    source: Arc<NamedSource<String>>,
    tokens: Vec<Token>,
    position: usize,
    source_text: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source_text: &'a str) -> Result<Self, MonError> {
        Self::new_with_name(source_text, "source.mon".to_string())
    }

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

    ///    Document ::= { ImportStatement } Object
    pub fn parse_document(&mut self) -> Result<MonDocument, MonError> {
        let mut imports: Vec<ImportStatement> = Vec::new();

        // consume zero-or-more import statements
        while self.check(TokenType::Import) {
            let imp = self.parse_import_statement()?;
            imports.push(imp);
            // DON'T call self.advance() here — parse_import_statement already advances
        }

        // After imports, we expect the root object.
        let root = self.parse_object()?;

        // After the root object, we expect the end of the file.
        self.expect(TokenType::Eof)?;
        Ok(MonDocument { root, imports })
    }

    /// Object ::= "{" [ MemberList ] "}"
    /// MemberList ::= Member { "," Member } [ "," ]
    fn parse_object(&mut self) -> Result<MonValue, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(TokenType::LBrace)?;
        let mut members = Vec::new();
        if !self.check(TokenType::RBrace) {
            // Parse the first member
            members.push(self.parse_member()?);
            // Keep parsing members as long as they are preceded by a comma
            while self.match_token(TokenType::Comma) {
                // If we match a comma but the next token is a brace, it's a trailing comma
                if self.check(TokenType::RBrace) {
                    break;
                }
                members.push(self.parse_member()?);
            }
        }
        let end_token = self.current_token()?.clone();
        self.expect(TokenType::RBrace)?;
        Ok(MonValue {
            kind: MonValueKind::Object(members),
            anchor: None, // Anchors are attached to values, not objects themselves
            pos_start: start_token.pos_start,
            pos_end: end_token.pos_end,
        })
    }

    /// Array ::= "[" [ ValueList ] "]"
    /// ValueList ::= Value { "," Value } [ "," ]
    fn parse_array(&mut self) -> Result<MonValue, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(TokenType::LBracket)?;
        let mut values = Vec::new();
        if !self.check(TokenType::RBracket) {
            loop {
                if self.check(TokenType::Spread) {
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

                if !self.match_token(TokenType::Comma) {
                    break;
                }
                if self.check(TokenType::RBracket) {
                    break; // Allow trailing comma
                }
            }
        }
        let end_token = self.current_token()?.clone();
        self.expect(TokenType::RBracket)?;
        Ok(MonValue {
            kind: MonValueKind::Array(values),
            anchor: None,
            pos_start: start_token.pos_start,
            pos_end: end_token.pos_end,
        })
    }

    /// Value ::= Object | Array | Alias | EnumValue | Literal
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

    /// Member ::= Pair | TypeDefinition | Spread
    fn parse_member(&mut self) -> Result<Member, MonError> {
        match self.current_token()?.ttype {
            TokenType::Spread => self.parse_spread().map(Member::Spread),
            // A TypeDefinition starts with an Identifier followed by a Colon and a Hash
            TokenType::Identifier(_)
                if self.peek_is(TokenType::Colon) && self.peek_next_is(TokenType::Hash) =>
            {
                self.parse_type_definition().map(Member::TypeDefinition)
            }
            // Otherwise, it's a regular pair
            _ => self.parse_pair().map(Member::Pair),
        }
    }

    /// Pair ::= KeyPart [ Validation ] ( ":" | "=" ) Value
    /// KeyPart ::= [ Anchor ] Key
    /// Key ::= Identifier | String
    fn parse_pair(&mut self) -> Result<Pair, MonError> {
        let mut anchor_from_key: Option<String> = None;

        // Handle the case where the key itself is an anchor, e.g., `&my_anchor: value`
        let key = if self.match_token(TokenType::Ampersand) {
            let key_name = self.parse_key()?;
            anchor_from_key = Some(key_name.clone());
            key_name
        } else {
            self.parse_key()?
        };

        let validation = self.parse_optional_validation()?;

        if !self.match_token(TokenType::Colon) && !self.match_token(TokenType::Equals) {
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
        while self.match_token(TokenType::Dot) {
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
        if self.match_token(TokenType::Ampersand) {
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
        self.expect(TokenType::Asterisk)?;
        let mut name = self.parse_key()?;
        let mut end_pos = self.current_token_before_advance()?.pos_end; // End of the first key part

        while self.match_token(TokenType::Dot) {
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
        self.expect(TokenType::Spread)?;
        let alias = self.parse_alias()?;
        if let MonValueKind::Alias(name) = alias.kind {
            Ok(name)
        } else {
            // This should be unreachable if parse_alias is correct
            self.err_unexpected("an alias after '...'")
        }
    }

    /// ImportStatement ::= "import" ( NamespaceImport | NamedImport ) "from" String
    fn parse_import_statement(&mut self) -> Result<ImportStatement, MonError> {
        let start_token = self.current_token()?.clone(); // Capture start token for pos_start
        self.expect(TokenType::Import)?;

        let spec = if self.match_token(TokenType::Asterisk) {
            // NamespaceImport ::= "*" "as" Identifier
            self.expect(TokenType::As)?;
            let name = self.parse_key()?;
            ImportSpec::Namespace(name)
        } else {
            // NamedImport ::= "{" [ ImportSpecifier { "," ImportSpecifier } [ "," ] ] "}"
            self.expect(TokenType::LBrace)?;
            let mut specifiers = Vec::new();
            if !self.check(TokenType::RBrace) {
                loop {
                    // ImportSpecifier ::= [ "&" ] Identifier
                    let is_anchor = self.match_token(TokenType::Ampersand);
                    let name = self.parse_key()?;
                    specifiers.push(ImportSpecifier { name, is_anchor });
                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                    if self.check(TokenType::RBrace) {
                        break;
                    }
                }
            }
            self.expect(TokenType::RBrace)?;
            ImportSpec::Named(specifiers)
        };

        self.expect(TokenType::From)?;
        let path_token = self.current_token()?.clone(); // Capture path token for pos_end
        let path = self.parse_key()?;

        Ok(ImportStatement {
            path,
            spec,
            pos_start: start_token.pos_start,
            pos_end: path_token.pos_end,
        })
    }

    /// TypeDefinition ::= Identifier ":" ( StructDefinition | EnumDefinition )
    fn parse_type_definition(&mut self) -> Result<TypeDefinition, MonError> {
        let name_token = self.current_token()?.clone();
        let name = self.parse_key()?;
        self.expect(TokenType::Colon)?;
        let hash_token = self.current_token()?.clone();
        self.expect(TokenType::Hash)?;

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

    /// StructDefinition ::= "{" [ FieldList ] "}"
    fn parse_struct_definition(&mut self) -> Result<StructDef, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(TokenType::LBrace)?;
        let mut fields = Vec::new();
        if !self.check(TokenType::RBrace) {
            loop {
                fields.push(self.parse_field_definition()?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
                if self.check(TokenType::RBrace) {
                    break;
                }
            }
        }
        let end_token = self.current_token()?.clone();
        self.expect(TokenType::RBrace)?;
        Ok(StructDef {
            fields,
            pos_start: start_token.pos_start,
            pos_end: end_token.pos_end,
        })
    }

    /// FieldDefinition ::= Identifier "(" Type ")" [ "=" Value ]
    fn parse_field_definition(&mut self) -> Result<FieldDef, MonError> {
        let name = self.parse_key()?;
        self.expect(TokenType::LParen)?;
        let type_spec = self.parse_type_spec()?;
        self.expect(TokenType::RParen)?;

        let default_value = if self.match_token(TokenType::Equals) {
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

    /// EnumDefinition ::= "{" [ Identifier { "," Identifier } [ "," ] ] "}"
    fn parse_enum_definition(&mut self) -> Result<EnumDef, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(TokenType::LBrace)?;
        let mut variants = Vec::new();
        if !self.check(TokenType::RBrace) {
            loop {
                variants.push(self.parse_key()?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
                if self.check(TokenType::RBrace) {
                    break;
                }
            }
        }
        let end_token = self.current_token()?.clone();
        self.expect(TokenType::RBrace)?;
        Ok(EnumDef {
            variants,
            pos_start: start_token.pos_start,
            pos_end: end_token.pos_end,
        })
    }

    /// Validation ::= "::" Type
    fn parse_optional_validation(&mut self) -> Result<Option<TypeSpec>, MonError> {
        if self.match_token(TokenType::DoubleColon) {
            self.parse_type_spec().map(Some)
        } else {
            Ok(None)
        }
    }

    /// Type ::= CollectionType | Identifier | "String" | ...
    fn parse_type_spec(&mut self) -> Result<TypeSpec, MonError> {
        let start_token = self.current_token()?.clone();
        if self.check(TokenType::LBracket) {
            // CollectionType ::= "[" Type [ "..." ] { "," Type [ "..." ] } "]"
            self.expect(TokenType::LBracket)?;
            let mut types = Vec::new();
            if !self.check(TokenType::RBracket) {
                loop {
                    let mut type_spec = self.parse_type_spec()?;
                    if self.match_token(TokenType::Spread) {
                        let end_token = self.current_token_before_advance()?.clone();
                        let span = (
                            type_spec.get_span().offset(),
                            end_token.pos_end - type_spec.get_span().offset(),
                        )
                            .into();
                        type_spec = TypeSpec::Spread(Box::new(type_spec), span);
                    }
                    types.push(type_spec);

                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                    if self.check(TokenType::RBracket) {
                        break;
                    }
                }
            }
            let end_token = self.current_token()?.clone();
            self.expect(TokenType::RBracket)?;
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

    /// EnumValue ::= "$" Identifier "." Identifier
    fn parse_enum_value(&mut self) -> Result<MonValue, MonError> {
        let start_token = self.current_token()?.clone();
        self.expect(TokenType::Dollar)?;
        let enum_name = self.parse_key()?;
        self.expect(TokenType::Dot)?;
        let variant_name = self.parse_key()?;
        let end_token = self.current_token_before_advance()?.clone(); // End of the variant_name

        Ok(MonValue {
            kind: MonValueKind::EnumValue {
                enum_name,
                variant_name,
            },
            anchor: None,
            pos_start: start_token.pos_start,
            pos_end: end_token.pos_end,
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
    fn expect(&mut self, expected: TokenType) -> Result<(), MonError> {
        let token = self.current_token()?.clone();
        if std::mem::discriminant(&token.ttype) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            self.err_unexpected(&format!("{:?}", expected))
        }
    }

    fn match_token(&mut self, ttype: TokenType) -> bool {
        if self.check(ttype) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, ttype: TokenType) -> bool {
        if let Ok(token) = self.current_token() {
            std::mem::discriminant(&token.ttype) == std::mem::discriminant(&ttype)
        } else {
            false
        }
    }

    fn peek_is(&self, ttype: TokenType) -> bool {
        if let Some(token) = self.tokens.get(self.position + 1) {
            std::mem::discriminant(&token.ttype) == std::mem::discriminant(&ttype)
        } else {
            false
        }
    }

    fn peek_next_is(&self, ttype: TokenType) -> bool {
        if let Some(token) = self.tokens.get(self.position + 2) {
            std::mem::discriminant(&token.ttype) == std::mem::discriminant(&ttype)
        } else {
            false
        }
    }

    #[track_caller]
    fn err_unexpected<T>(&self, expected: &str) -> Result<T, MonError> {
        let token = self.current_token()?; // Should be safe if we got here
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
        Ok(doc) => format!("{:#?}", doc), // debug format for success
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
mod tests {
    use super::*;
    use miette::Report;
    use std::fs;

    fn parse_ok(source: &str) -> MonDocument {
        let mut parser = Parser::new_with_name(source, "test.mon".to_string()).unwrap();
        match parser.parse_document() {
            Ok(doc) => doc,
            Err(err) => {
                let report = Report::from(err);
                print!("{:?}", report);

                panic!("{:#}", report);
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
        let doc = parse_ok(r#"import * as my_schemas from "./schemas.mon" {}"#);
        assert_eq!(doc.imports.len(), 1);
        let i = &doc.imports[0];
        assert_eq!(i.path, "./schemas.mon");
        assert!(matches!(i.spec, ImportSpec::Namespace(ref s) if s == "my_schemas"));

        // Root object should be empty
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!("Root was not an object"),
        };
        assert!(members.is_empty());
    }

    #[test]
    fn test_named_import() {
        let doc = parse_ok(r#"import { User, &Template } from "./file.mon" {}"#);
        assert_eq!(doc.imports.len(), 1);
        let i = &doc.imports[0];
        assert_eq!(i.path, "./file.mon");
        match &i.spec {
            ImportSpec::Named(specs) => {
                assert_eq!(specs.len(), 2);
                assert_eq!(specs[0].name, "User");
                assert!(!specs[0].is_anchor);
                assert_eq!(specs[1].name, "Template");
                assert!(specs[1].is_anchor);
            }
            _ => panic!(),
        }
        // Root object should be empty
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!("Root was not an object"),
        };
        assert!(members.is_empty());
    }

    #[test]
    fn test_enum_definition() {
        let doc = parse_ok(r#"{ Status: #enum { Active, Inactive } }"#);
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 1);
        match &members[0] {
            Member::TypeDefinition(t) => {
                assert_eq!(t.name, "Status");
                assert!(matches!(t.def_type, TypeDef::Enum(_)));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_struct_definition() {
        let doc = parse_ok(r#"{ User: #struct { id(Number), name(String) = "Guest" } }"#);
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 1);
        match &members[0] {
            Member::TypeDefinition(t) => {
                assert_eq!(t.name, "User");
                match &t.def_type {
                    TypeDef::Struct(s) => {
                        assert_eq!(s.fields.len(), 2);
                        assert_eq!(s.fields[0].name, "id");
                        assert!(s.fields[1].default_value.is_some());
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_validation_pair() {
        let doc = parse_ok(r#"{ my_user :: User = { name: "Alice" } }"#);
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 1);
        match &members[0] {
            Member::Pair(p) => {
                assert_eq!(p.key, "my_user");
                assert!(p.validation.is_some());
                assert!(matches!(p.value.kind, MonValueKind::Object(_)));
            }
            _ => panic!(),
        }
    }

    #[test]
    #[ignore]
    fn visual_conformation_from_golden() {
        let contents = fs::read_to_string("tests/ok/golden.mon").unwrap();
        let parsed = Parser::new_with_name(&contents, "test.mon".to_string())
            .unwrap()
            .parse_document();

        print!("parsed: \n{}", pretty_result(parsed))
    }

    #[test]
    fn test_all_mon_files() {
        let tests_dir = "./tests";
        let entries = fs::read_dir(tests_dir).expect("Failed to read tests directory");

        for entry in entries {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "mon") {
                println!("Parsing file: {:?}", path);
                let source = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Failed to read file: {:?}", path));

                let mut parser = Parser::new_with_name(&source, path.to_str().unwrap().to_string())
                    .expect("Lexer failed");

                if let Err(err) = parser.parse_document() {
                    panic!("Failed to parse {:?}. Error: {:#?}", path, Report::new(err));
                }
            }
        }
    }
}
