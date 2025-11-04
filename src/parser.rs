use crate::ast::*;
use crate::error::{MonError, ParserError};
use crate::lexer::{Lexer, Token, TokenType};
use miette::NamedSource;
use std::panic::Location;
use std::sync::Arc;

/// A recursive descent parser for the MON language, built according to the EBNF grammar.
pub struct Parser<'a> {
    source: Arc<NamedSource<String>>,
    tokens: Vec<Token>,
    position: usize,
    source_text: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source_text: &'a str) -> Result<Self, MonError> {
        let source = Arc::new(NamedSource::new("source.mon", source_text.to_string()));
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

    /// Document ::= Object
    pub fn parse_document(&mut self) -> Result<MonDocument, MonError> {
        let root = self.parse_object()?;
        self.expect(TokenType::Eof)?;
        Ok(MonDocument { root })
    }

    /// Object ::= "{" [ MemberList ] "}"
    /// MemberList ::= Member { "," Member } [ "," ]
    fn parse_object(&mut self) -> Result<MonValue, MonError> {
        self.expect(TokenType::LBrace)?;
        let mut members = Vec::new();
        if !self.check(TokenType::RBrace) {
            loop {
                members.push(self.parse_member()?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
                if self.check(TokenType::RBrace) {
                    break; // Allow trailing comma
                }
            }
        }
        self.expect(TokenType::RBrace)?;
        Ok(MonValue {
            kind: MonValueKind::Object(members),
            anchor: None, // Anchors are attached to values, not objects themselves
        })
    }

    /// Array ::= "[" [ ValueList ] "]"
    /// ValueList ::= Value { "," Value } [ "," ]
    fn parse_array(&mut self) -> Result<MonValue, MonError> {
        self.expect(TokenType::LBracket)?;
        let mut values = Vec::new();
        if !self.check(TokenType::RBracket) {
            loop {
                values.push(self.parse_value()?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
                if self.check(TokenType::RBracket) {
                    break; // Allow trailing comma
                }
            }
        }
        self.expect(TokenType::RBracket)?;
        Ok(MonValue {
            kind: MonValueKind::Array(values),
            anchor: None,
        })
    }

    /// Value ::= Object | Array | Alias | EnumValue | Literal
    /// Attaches an anchor if one is present.
    fn parse_value(&mut self) -> Result<MonValue, MonError> {
        let anchor = self.parse_optional_anchor()?;

        let token = self.current_token()?;
        let mut value = match &token.ttype.clone() {
            TokenType::LBrace => self.parse_object(),
            TokenType::LBracket => self.parse_array(),
            TokenType::String(s) => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::String(s.clone()),
                    anchor: None,
                })
            }
            TokenType::Number(n) => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::Number(*n),
                    anchor: None,
                })
            }
            TokenType::True => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::Boolean(true),
                    anchor: None,
                })
            }
            TokenType::False => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::Boolean(false),
                    anchor: None,
                })
            }
            TokenType::Null => {
                self.advance();
                Ok(MonValue {
                    kind: MonValueKind::Null,
                    anchor: None,
                })
            }
            TokenType::Asterisk => self.parse_alias(),
            TokenType::Dollar => self.parse_enum_value(),
            _ => self.err_unexpected("a value"),
        }?;

        value.anchor = anchor;
        Ok(value)
    }

    /// Member ::= Pair | TypeDefinition | ImportStatement | Spread
    fn parse_member(&mut self) -> Result<Member, MonError> {
        match self.current_token()?.ttype {
            TokenType::Import => self.parse_import_statement().map(Member::Import),
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
        let anchor = self.parse_optional_anchor()?;
        let key = self.parse_key()?;
        let validation = self.parse_optional_validation()?;

        if !self.match_token(TokenType::Colon) && !self.match_token(TokenType::Equals) {
            return self.err_unexpected("':' or '=' after key");
        }

        let mut value = self.parse_value()?;
        // The anchor belongs to the value, not the key part.
        if anchor.is_some() {
            value.anchor = anchor;
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
        match &token.ttype {
            TokenType::Identifier(s) | TokenType::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => self.err_unexpected("an identifier or string for a key"),
        }
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

    /// Alias ::= "*" Identifier
    fn parse_alias(&mut self) -> Result<MonValue, MonError> {
        self.expect(TokenType::Asterisk)?;
        let token = self.current_token()?;
        if let TokenType::Identifier(name) = &token.ttype {
            let name = name.clone();
            self.advance();
            Ok(MonValue {
                kind: MonValueKind::Alias(name),
                anchor: None,
            })
        } else {
            self.err_unexpected("an identifier for the alias name")
        }
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
        let path = self.parse_key()?;
        Ok(ImportStatement { path, spec })
    }

    /// TypeDefinition ::= Identifier ":" ( StructDefinition | EnumDefinition )
    fn parse_type_definition(&mut self) -> Result<TypeDefinition, MonError> {
        let name = self.parse_key()?;
        self.expect(TokenType::Colon)?;
        self.expect(TokenType::Hash)?;

        let token = self.current_token()?;
        let def_type = match &token.ttype {
            TokenType::Identifier(s) if s == "struct" => {
                self.advance();
                self.parse_struct_definition().map(TypeDef::Struct)
            }
            TokenType::Identifier(s) if s == "enum" => {
                self.advance();
                self.parse_enum_definition().map(TypeDef::Enum)
            }
            _ => self.err_unexpected("'struct' or 'enum' keyword"),
        }?;

        Ok(TypeDefinition { name, def_type })
    }

    /// StructDefinition ::= "{" [ FieldList ] "}"
    fn parse_struct_definition(&mut self) -> Result<StructDef, MonError> {
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
        self.expect(TokenType::RBrace)?;
        Ok(StructDef { fields })
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
        self.expect(TokenType::RBrace)?;
        Ok(EnumDef { variants })
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
        if self.check(TokenType::LBracket) {
            // CollectionType ::= "[" Type [ "..." ] { "," Type [ "..." ] } "]"
            self.expect(TokenType::LBracket)?;
            // This is a simplified implementation for collection types.
            // A full implementation would need to handle the recursive structure.
            let type_name = self.parse_key()?;
            let _is_spread = self.match_token(TokenType::Spread);
            self.expect(TokenType::RBracket)?;
            Ok(TypeSpec::Collection(vec![TypeSpec::Simple(type_name)]))
        } else {
            // Simple Type
            self.parse_key().map(TypeSpec::Simple)
        }
    }

    /// EnumValue ::= "$" Identifier "." Identifier
    fn parse_enum_value(&mut self) -> Result<MonValue, MonError> {
        self.expect(TokenType::Dollar)?;
        let enum_name = self.parse_key()?;
        self.expect(TokenType::Dot)?;
        let variant_name = self.parse_key()?;
        Ok(MonValue {
            kind: MonValueKind::EnumValue {
                enum_name,
                variant_name,
            },
            anchor: None,
        })
    }

    // === Tokenizer Helper Methods ===

    fn current_token(&self) -> Result<&Token, MonError> {
        self.tokens.get(self.position).ok_or_else(|| {
            let pos = self.source_text.len().saturating_sub(1);
            ParserError::UnexpectedEof {
                src: (*self.source).clone(),
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

    fn expect(&mut self, expected: TokenType) -> Result<(), MonError> {
        let token = self.current_token()?;
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
            token.ttype == ttype
        } else {
            false
        }
    }

    #[track_caller]
    fn err_unexpected<T>(&self, expected: &str) -> Result<T, MonError> {
        let token = self.current_token().unwrap(); // Should be safe if we got here
        print!("caller: {}", Location::caller());
        Err(ParserError::UnexpectedToken {
            src: (*self.source).clone(),
            span: (token.pos_start, token.pos_end - token.pos_start).into(),
            expected: expected.to_string(),
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Report;

    fn parse_ok(source: &str) -> MonDocument {
        let mut parser = Parser::new(source).unwrap();
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
        let doc = parse_ok(r#"{ import * as my_schemas from "./schemas.mon" }"#);
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 1);
        match &members[0] {
            Member::Import(i) => {
                assert_eq!(i.path, "./schemas.mon");
                assert!(matches!(i.spec, ImportSpec::Namespace(_)));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_named_import() {
        let doc = parse_ok(r#"{ import { User, &Template } from "./file.mon" }"#);
        let members = match doc.root.kind {
            MonValueKind::Object(m) => m,
            _ => panic!(),
        };
        assert_eq!(members.len(), 1);
        match &members[0] {
            Member::Import(i) => {
                assert_eq!(i.path, "./file.mon");
                match &i.spec {
                    ImportSpec::Named(specs) => {
                        assert_eq!(specs.len(), 2);
                        assert_eq!(specs[0].name, "User");
                        assert_eq!(specs[0].is_anchor, false);
                        assert_eq!(specs[1].name, "Template");
                        assert_eq!(specs[1].is_anchor, true);
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
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
}
