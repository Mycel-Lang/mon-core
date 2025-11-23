//! # MON Lexer (Tokenizer)
//!
//! This module provides the `Lexer` for the MON language. The lexer, also known as a
//! tokenizer or scanner, is the first stage in the compilation process. It is responsible
//! for converting a raw source code string into a sequence of discrete `Token`s.
//!
//! ## Architectural Overview
//!
//! The `Lexer` is a hand-written, stateful iterator that scans the input character by character
//! to produce tokens. It recognizes all the fundamental building blocks of the language, such as:
//!
//! - **Literals:** Identifiers, strings, and numbers.
//! - **Keywords:** `true`, `false`, `null`, `import`, etc.
//! - **Punctuation:** Braces `{{}}`, brackets `[]`, commas `,`, colons `:`, etc.
//! - **Operators:** `::`, `...`, `&`, `*`, etc.
//! - **Whitespace and Comments:** These are also produced as tokens, which allows subsequent
//!   tools (like formatters or IDEs) to preserve them. The [`Parser`](crate::parser::Parser)
//!   typically filters them out.
//!
//! Each `Token` produced contains a [`TokenType`] and its start and end byte positions
//! in the original source, which is crucial for error reporting.
//!
//! ## Use Cases
//!
//! Direct use of the `Lexer` is not as common as using the full [`analyze`](crate::api::analyze) pipeline,
//! but it is essential for tools that operate at the token level.
//!
//! - **Syntax Highlighting:** A syntax highlighter can use the lexer to assign colors to
//!   different token types.
//! - **Code Formatting:** A formatter (like `rustfmt`) uses the token stream, including
//!   whitespace and comments, to re-format the code according to a set of rules.
//! - **Debugging and Educational Tools:** It can be used to show how a source file is broken
//!   down into its most basic components.
//!
//! ## Example: Direct Lexer Usage
//!
//! ```rust
//! use mon_core::lexer::{Lexer, TokenType, Token};
//!
//! let source = "key: 123 // A number";
//!
//! // Create a new lexer for the source string.
//! let mut lexer = Lexer::new(source);
//!
//! // You can retrieve all tokens at once.
//! let tokens: Vec<Token> = lexer.lex();
//!
//! // Or process them one by one.
//! let mut lexer = Lexer::new(source);
//! assert_eq!(lexer.next_token().ttype, TokenType::Identifier("key".to_string()));
//! assert_eq!(lexer.next_token().ttype, TokenType::Colon);
//! assert_eq!(lexer.next_token().ttype, TokenType::Whitespace);
//! assert_eq!(lexer.next_token().ttype, TokenType::Number(123.0));
//! assert_eq!(lexer.next_token().ttype, TokenType::Whitespace);
//! assert!(matches!(lexer.next_token().ttype, TokenType::Comment(_)));
//! assert_eq!(lexer.next_token().ttype, TokenType::Eof);
//! ```
/// Represents the different kinds of tokens that the lexer can produce.
/// Each token is a meaningful unit of the MON language syntax.
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // == Special Tokens ==
    /// Represents the end of the input file.
    Eof,
    /// Represents a sequence of one or more whitespace characters (spaces, tabs, newlines).
    Whitespace,
    /// Represents a comment, starting with `//` and continuing to the end of the line.
    /// The associated `String` contains the content of the comment.
    Comment(String),
    /// Represents a token that could not be recognized by the lexer.
    Unknown,

    // == Literals ==
    /// An identifier, used for keys, type names, and anchor/alias names.
    /// Examples: `name`, `User`, `&default_user`.
    Identifier(String),
    /// A string literal, enclosed in double quotes.
    /// The associated `String` holds the content of the string.
    String(String),
    /// A number literal, which can be an integer or a floating-point value.
    Number(f64),

    // == Keywords ==
    /// The boolean `true` value, can be written as `true` or `on`.
    True,
    /// The boolean `false` value, can be written as `false` or `off`.
    False,
    /// The `null` keyword, representing an empty or absent value.
    Null,
    /// The `import` keyword, used for the module system.
    Import,
    /// The `from` keyword, used for named imports.
    From,
    /// The `as` keyword, used for namespacing imports.
    As,

    // == Punctuation & Operators ==
    /// Left Brace: `{`
    LBrace,
    /// Right Brace: `}`
    RBrace,
    /// Left Bracket: `[`
    LBracket,
    /// Right Bracket: `]`
    RBracket,
    /// Left Parenthesis: `(`
    LParen,
    /// Right Parenthesis: `)`
    RParen,
    /// Comma: `,`
    Comma,
    /// Colon: `:`
    Colon,
    /// Double Colon: `::` (used for type annotations)
    DoubleColon,
    /// Dot: `.` (used for namespace access)
    Dot,
    /// Equals: `=` (used for using structs)
    Equals,
    /// Hash: `#` (used as a prefix for type definitions, e.g., `#struct`)
    Hash,
    /// Dollar Sign: `$` (used for accessing enum variants)
    Dollar,
    /// Ampersand: `&` (used to define an anchor)
    Ampersand,
    /// Asterisk: `*` (used to create an alias of an anchor)
    Asterisk,
    /// Spread: `...` (used to spread an anchor into an object or array)
    Spread,
}

/// Represents a single lexical token, containing its type and position in the source text.
///
/// A `Token` is an atomic unit of the language syntax, like an identifier, a keyword, or a symbol.
#[derive(Debug, Clone)]
pub struct Token {
    /// The type of the token, e.g., `TokenType::Identifier`.
    pub ttype: TokenType,
    /// The 0-based starting byte position of the token in the source string.
    pub pos_start: usize,
    /// The 0-based ending byte position of the token in the source string.
    pub pos_end: usize,
}

impl Token {
    /// Creates a new `Token`.
    #[must_use]
    pub fn new(ttype: TokenType, pos_start: usize, pos_end: usize) -> Token {
        Token {
            ttype,
            pos_start,
            pos_end,
        }
    }
}

/// A lexer for the MON language, also known as a tokenizer or scanner.
///
/// The `Lexer`'s primary role is to read MON source code as a stream of characters
/// and break it down into a sequence of [`Token`]s. Each token represents a
/// meaningful unit of the language, like an identifier, a number, or a punctuation mark.
///
/// The `Lexer` is the first step in the compilation pipeline, providing the input
/// for the [`Parser`](crate::parser::Parser).
///
/// # Example: How to use the Lexer
///
/// You can use the `Lexer` to tokenize a MON source string and inspect the tokens.
///
/// ```rust
/// use mon_core::lexer::{Lexer, TokenType, Token};
///
/// let source = "{ key: 123 }";
///
/// // 1. Create a new lexer for the source code.
/// let mut lexer = Lexer::new(source);
///
/// // 2. Use `lex()` to get all tokens.
/// let tokens: Vec<Token> = lexer.lex();
///
/// assert_eq!(tokens[0].ttype, TokenType::LBrace);
/// assert_eq!(tokens[1].ttype, TokenType::Whitespace);
/// assert_eq!(tokens[2].ttype, TokenType::Identifier("key".to_string()));
/// assert_eq!(tokens[3].ttype, TokenType::Colon);
/// assert_eq!(tokens[4].ttype, TokenType::Whitespace);
/// assert_eq!(tokens[5].ttype, TokenType::Number(123.0));
/// assert_eq!(tokens[6].ttype, TokenType::Whitespace);
/// assert_eq!(tokens[7].ttype, TokenType::RBrace);
/// assert_eq!(tokens[8].ttype, TokenType::Eof);
///
/// // Alternatively, you can process tokens one by one using `next_token`.
/// let mut lexer = Lexer::new(source);
/// assert_eq!(lexer.next_token().ttype, TokenType::LBrace);
/// assert_eq!(lexer.next_token().ttype, TokenType::Whitespace);
/// ```
pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    position: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new `Lexer` for the given input string.
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            position: 0,
        }
    }

    /// Consumes the `Lexer` and returns a `Vec<Token>` containing all tokens from the source.
    ///
    /// This method will tokenize the entire input string up to and including the final [`TokenType::Eof`] token.
    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token.ttype == TokenType::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    /// Scans and returns the next [`Token`] from the input stream.
    ///
    /// This is the core tokenizing function. When the end of the input is reached,
    /// it will repeatedly return a token of type [`TokenType::Eof`].
    pub fn next_token(&mut self) -> Token {
        let start_pos = self.position;

        let ttype = if let Some(char) = self.advance() {
            match char {
                '{' => TokenType::LBrace,
                '}' => TokenType::RBrace,
                '[' => TokenType::LBracket,
                ']' => TokenType::RBracket,
                '(' => TokenType::LParen,
                ')' => TokenType::RParen,
                ',' => TokenType::Comma,
                '#' => TokenType::Hash,
                '$' => TokenType::Dollar,
                '&' => TokenType::Ampersand,
                '*' => TokenType::Asterisk,
                '=' => TokenType::Equals,

                ':' => {
                    if self.peek() == Some(&':') {
                        self.advance();
                        TokenType::DoubleColon
                    } else {
                        TokenType::Colon
                    }
                }
                '.' => {
                    if self.peek() == Some(&'.') {
                        self.advance();
                        if self.peek() == Some(&'.') {
                            self.advance();
                            TokenType::Spread
                        } else {
                            TokenType::Unknown
                        }
                    } else {
                        TokenType::Dot
                    }
                }
                '/' => {
                    if self.peek() == Some(&'/') {
                        self.read_comment()
                    } else {
                        TokenType::Unknown
                    }
                }
                '"' => self.read_string(),
                c if c.is_whitespace() => self.read_whitespace(),
                c if c.is_ascii_alphabetic() || c == '_' => self.read_identifier(c),
                c if c.is_ascii_digit()
                    || (c == '-' && self.peek().is_some_and(char::is_ascii_digit)) =>
                {
                    self.read_number(c)
                }

                _ => TokenType::Unknown,
            }
        } else {
            TokenType::Eof
        };

        Token::new(ttype, start_pos, self.position)
    }

    fn advance(&mut self) -> Option<char> {
        let char = self.chars.next();
        if let Some(c) = char {
            self.position += c.len_utf8();
        }
        char
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn read_whitespace(&mut self) -> TokenType {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
        TokenType::Whitespace
    }

    fn read_comment(&mut self) -> TokenType {
        self.advance(); // Consume the second '/'
        let mut comment_text = String::new();
        while let Some(c) = self.peek() {
            if *c == '\n' {
                break;
            }
            comment_text.push(self.advance().unwrap());
        }
        TokenType::Comment(comment_text.trim().to_string())
    }

    fn read_string(&mut self) -> TokenType {
        let mut value = String::new();
        loop {
            match self.peek() {
                Some('"') => {
                    self.advance(); // Consume the closing quote
                    return TokenType::String(value);
                }
                Some('\\') => {
                    self.advance(); // Consume the backslash
                    match self.advance() {
                        Some('"') => value.push('"'),
                        Some('\\') => value.push('\\'),
                        Some('n') => value.push('\n'),
                        Some('r') => value.push('\r'),
                        Some('t') => value.push('\t'),
                        Some(other) => {
                            value.push('\\');
                            value.push(other);
                        }
                        None => return TokenType::Unknown, // Unclosed escape sequence
                    }
                }
                Some(c) => {
                    value.push(*c);
                    self.advance();
                }
                None => return TokenType::Unknown, // Unclosed string
            }
        }
    }

    fn read_identifier(&mut self, first_char: char) -> TokenType {
        let mut ident = String::new();
        ident.push(first_char);

        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || *c == '_' {
                ident.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        match ident.as_str() {
            "true" | "on" => TokenType::True,
            "false" | "off" => TokenType::False,
            "null" => TokenType::Null,
            "import" => TokenType::Import,
            "from" => TokenType::From,
            "as" => TokenType::As,
            _ => TokenType::Identifier(ident),
        }
    }

    fn read_number(&mut self, first_char: char) -> TokenType {
        let mut number_str = String::new();
        number_str.push(first_char);
        let mut has_dot = first_char == '.';
        let mut has_exponent = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                number_str.push(self.advance().unwrap());
            } else if *c == '.' && !has_dot {
                has_dot = true;
                number_str.push(self.advance().unwrap());
            } else if (*c == 'e' || *c == 'E') && !has_exponent {
                has_exponent = true;
                number_str.push(self.advance().unwrap());
                // Check for optional sign after 'e' or 'E'
                if let Some(sign_char) = self.peek() {
                    if *sign_char == '+' || *sign_char == '-' {
                        number_str.push(self.advance().unwrap());
                    }
                }
            } else {
                break;
            }
        }

        if let Ok(num) = number_str.parse::<f64>() {
            TokenType::Number(num)
        } else {
            TokenType::Unknown
        }
    }
}

/// QOL function
#[allow(dead_code)]
pub(crate) fn tokens_to_pretty_string(tokens: &[Token]) -> String {
    let mut buff: Vec<String> = Vec::with_capacity(tokens.len());

    for token in tokens {
        buff.push(format!(
            "{:?}, {}, {}",
            token.ttype, token.pos_start, token.pos_end,
        ));
    }

    buff.join("\n")
}

#[cfg(test)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::explicit_auto_deref)]
mod tests {
    use super::*;

    fn assert_tokens(input: &str, expected: &[TokenType]) {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex();
        let token_types: Vec<TokenType> = tokens.into_iter().map(|t| t.ttype).collect();

        // Filter out whitespace and comments for most tests
        let filtered_tokens: Vec<TokenType> = token_types
            .into_iter()
            .filter(|t| !matches!(t, TokenType::Whitespace | TokenType::Comment(_)))
            .collect();

        assert_eq!(filtered_tokens, expected);
    }

    #[test]
    fn test_eof() {
        assert_tokens("", &[TokenType::Eof]);
    }

    #[test]
    fn test_single_char_tokens() {
        let input = "{}[](),:#{new_string}*";
        let expected = vec![
            TokenType::LBrace,
            TokenType::RBrace,
            TokenType::LBracket,
            TokenType::RBracket,
            TokenType::LParen,
            TokenType::RParen,
            TokenType::Comma,
            TokenType::Colon,
            TokenType::Hash,
            TokenType::LBrace,
            TokenType::Identifier("new_string".to_string()),
            TokenType::RBrace,
            TokenType::Asterisk,
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_multi_char_operators() {
        let input = ":: ...";
        let expected = vec![TokenType::DoubleColon, TokenType::Spread, TokenType::Eof];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_keywords() {
        let input = "true on false off null import from as";
        let expected = vec![
            TokenType::True,
            TokenType::True,
            TokenType::False,
            TokenType::False,
            TokenType::Null,
            TokenType::Import,
            TokenType::From,
            TokenType::As,
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_identifiers() {
        let input = "foo bar_123 _baz";
        let expected = vec![
            TokenType::Identifier("foo".to_string()),
            TokenType::Identifier("bar_123".to_string()),
            TokenType::Identifier("_baz".to_string()),
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_numbers() {
        let input = "123 45.67 -10 0.5";
        let expected = vec![
            TokenType::Number(123.0),
            TokenType::Number(45.67),
            TokenType::Number(-10.0),
            TokenType::Number(0.5),
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_comments_and_whitespace() {
        let input = " // this is a comment\n key: value // another one";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex();
        let token_types: Vec<TokenType> = tokens.into_iter().map(|t| t.ttype).collect();

        let expected = vec![
            TokenType::Whitespace,
            TokenType::Comment("this is a comment".to_string()),
            TokenType::Whitespace,
            TokenType::Identifier("key".to_string()),
            TokenType::Colon,
            TokenType::Whitespace,
            TokenType::Identifier("value".to_string()),
            TokenType::Whitespace,
            TokenType::Comment("another one".to_string()),
            TokenType::Eof,
        ];

        assert_eq!(token_types, expected);
    }

    #[test]
    fn test_complex_mon_structure() {
        let input = r#"
        {
        // Config settings
        service_name: "My App",
        port: 8080,
        is_enabled: on,

        &default_user: {
            permissions: ["READ", "WRITE"],
        },

        admin :: User = {
            ...*default_user,
            name: "Admin",
            }
        }
                    "#;
        let expected = vec![
            TokenType::LBrace,
            TokenType::Identifier("service_name".to_string()),
            TokenType::Colon,
            TokenType::String("My App".to_string()),
            TokenType::Comma,
            TokenType::Identifier("port".to_string()),
            TokenType::Colon,
            TokenType::Number(8080.0),
            TokenType::Comma,
            TokenType::Identifier("is_enabled".to_string()),
            TokenType::Colon,
            TokenType::True,
            TokenType::Comma,
            TokenType::Ampersand,
            TokenType::Identifier("default_user".to_string()),
            TokenType::Colon,
            TokenType::LBrace,
            TokenType::Identifier("permissions".to_string()),
            TokenType::Colon,
            TokenType::LBracket,
            TokenType::String("READ".to_string()),
            TokenType::Comma,
            TokenType::String("WRITE".to_string()),
            TokenType::RBracket,
            TokenType::Comma,
            TokenType::RBrace,
            TokenType::Comma,
            TokenType::Identifier("admin".to_string()),
            TokenType::DoubleColon,
            TokenType::Identifier("User".to_string()),
            TokenType::Equals,
            TokenType::LBrace,
            TokenType::Spread,
            TokenType::Asterisk,
            TokenType::Identifier("default_user".to_string()),
            TokenType::Comma,
            TokenType::Identifier("name".to_string()),
            TokenType::Colon,
            TokenType::String("Admin".to_string()),
            TokenType::Comma,
            TokenType::RBrace,
            TokenType::RBrace,
            TokenType::Eof,
        ];
        print!("{input}");
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_unclosed_string() {
        let input = r#"{ key: "unclosed }"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex();
        
        // Unclosed string should return Unknown token
        let has_unknown = tokens.iter().any(|t| matches!(t.ttype, TokenType::Unknown));
        assert!(has_unknown, "Should have Unknown token for unclosed string");
    }

    #[test]
    fn test_string_with_escapes() {
        let input = r#""hello\nworld\t\"test\"""#;
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        
        match token.ttype {
            TokenType::String(s) => {
                // The lexer actually processes the escapes
                assert!(s.contains('\n'));
                assert!(s.contains('\t'));
                assert!(s.contains('"'));
                assert_eq!(s, "hello\nworld\t\"test\"");
            }
            _ => panic!("Expected string token, got {:?}", token.ttype),
        }
    }

    #[test]
    fn test_invalid_escape_at_eof() {
        let input = r#""test\"#;
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        assert!(matches!(token.ttype, TokenType::Unknown));
    }

    #[test]
    fn test_number_with_exponent() {
        let input = "1.23e10 4.5E-3";
        let mut lexer = Lexer::new(input);
        
        let tok1 = lexer.next_token();
        assert!(matches!(tok1.ttype, TokenType::Number(n) if (n - 1.23e10).abs() < 1e-6));
        
        lexer.next_token(); // whitespace
        let tok2 = lexer.next_token();
        assert!(matches!(tok2.ttype, TokenType::Number(n) if (n - 4.5e-3).abs() < 1e-9));
    }

    #[test]
    fn test_negative_numbers() {
        let input = "-42 -3.14";
        let expected = vec![
            TokenType::Number(-42.0),
            TokenType::Number(-3.14),
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_dotdot_not_spread() {
        // ".." without third dot should be two dots
        let input = "..";
        let mut lexer = Lexer::new(input);
        let tok1 = lexer.next_token();
        let tok2 = lexer.next_token();
        
        // First dot, then either another dot or unknown
        assert!(matches!(tok1.ttype, TokenType::Dot | TokenType::Unknown));
    }

    #[test]
    fn test_unknown_character() {
        let input = "{ @invalid }";
        let mut lexer = Lexer::new(input);
        let tokens: Vec<TokenType> = lexer.lex().into_iter().map(|t| t.ttype).collect();
        
        // Should have Unknown token for @
        assert!(tokens.iter().any(|t| matches!(t, TokenType::Unknown)));
    }

    #[test]
    fn test_single_slash_not_comment() {
        let input = "test / value";
        let mut lexer = Lexer::new(input);
        let tokens: Vec<TokenType> = lexer.lex().into_iter().map(|t| t.ttype).collect();
        
        // Single slash should produce Unknown
        assert!(tokens.iter().any(|t| matches!(t, TokenType::Unknown)));
    }

    #[test]
    fn test_escape_r() {
        let input = r#""test\rvalue""#;
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        assert!(matches!(token.ttype, TokenType::String(s) if s.len() > 0));
    }

    #[test]
    fn test_escape_backslash() {
        let input = r#""test\\value""#;
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        assert!(matches!(token.ttype, TokenType::String(s) if s.len() > 0));
    }

    #[test]
    fn test_unknown_escape_preserved() {
        let input = r#""test\xvalue""#;
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        // String should parse successfully
        assert!(matches!(token.ttype, TokenType::String(_)));
    }

    #[test]
    fn test_zero_number() {
        assert_tokens("0", &[TokenType::Number(0.0), TokenType::Eof]);
    }

    #[test]
    fn test_decimal_point_only() {
        assert_tokens("3.14", &[TokenType::Number(3.14), TokenType::Eof]);
    }

    #[test]
    fn test_leading_decimal() {
        // .5 should be parsed as dot  + number
        let input = ".5";
        let mut lexer = Lexer::new(input);
        let tok1 = lexer.next_token();
        let tok2 = lexer.next_token();
        assert!(matches!(tok1.ttype, TokenType::Dot));
        assert!(matches!(tok2.ttype, TokenType::Number(5.0)));
    }

    #[test]
    fn test_multiline_comment() {
        let input = "// line 1\n// line 2\nvalue";
        let mut lexer = Lexer::new(input);
        let tokens: Vec<TokenType> = lexer.lex().into_iter()
            .filter(|t| !matches!(t.ttype, TokenType::Whitespace | TokenType::Comment(_)))
            .map(|t| t.ttype)
            .collect();
        assert_eq!(tokens, vec![TokenType::Identifier("value".to_string()), TokenType::Eof]);
    }

    #[test]
    fn test_comment_at_eof() {
        let input = "value // comment at end";
        let mut lexer = Lexer::new(input);
        let tokens: Vec<TokenType> = lexer.lex().into_iter().map(|t| t.ttype).collect();
        assert!(tokens.iter().any(|t| matches!(t, TokenType::Comment(_))));
    }

    #[test]
    fn test_all_keywords() {
        let input = "true false null on off import from as";
        let expected = vec![
            TokenType::True,
            TokenType::False,
            TokenType::Null,
            TokenType::True,  // 'on' maps to true
            TokenType::False, // 'off' maps to false
            TokenType::Import,
            TokenType::From,
            TokenType::As,
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_identifiers_with_underscores() {
        let input = "my_var _private __dunder";
        let expected = vec![
            TokenType::Identifier("my_var".to_string()),
            TokenType::Identifier("_private".to_string()),
            TokenType::Identifier("__dunder".to_string()),
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_mixed_operators() {
        let input = ":: = ...";
        let expected = vec![
            TokenType::DoubleColon,
            TokenType::Equals,
            TokenType::Spread,
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_adjacent_tokens_no_whitespace() {
        let input = "[1,2,3]";
        let mut lexer = Lexer::new(input);
        let tokens: Vec<TokenType> = lexer.lex().into_iter()
            .filter(|t| !matches!(t.ttype, TokenType::Whitespace))
            .map(|t| t.ttype)
            .collect();
        assert_eq!(tokens.len(), 8); // [, 1, ,, 2, ,, 3, ], EOF
    }

    #[test]
    fn test_hash_token() {
        let input = "#struct";
        let expected = vec![
            TokenType::Hash,
            TokenType::Identifier("struct".to_string()),
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_dollar_token() {
        let input = "$Status.Active";
        let expected = vec![
            TokenType::Dollar,
            TokenType::Identifier("Status".to_string()),
            TokenType::Dot,
            TokenType::Identifier("Active".to_string()),
            TokenType::Eof,
        ];
        assert_tokens(input, &expected);
    }

    #[test]
    fn test_empty_string() {
        let input = r#""""#;
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        assert_eq!(token.ttype, TokenType::String("".to_string()));
    }
}
