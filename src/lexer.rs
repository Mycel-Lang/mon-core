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

/// A token with its type and position
#[derive(Debug, Clone)]
pub struct Token {
    pub ttype: TokenType,
    pub pos_start: usize,
    pub pos_end: usize,
}

impl Token {
    pub fn new(ttype: TokenType, pos_start: usize, pos_end: usize) -> Token {
        Token {
            ttype,
            pos_start,
            pos_end,
        }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            position: 0,
        }
    }

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
                    || (c == '-' && self.peek().map_or(false, |c| c.is_ascii_digit())) =>
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
        while let Some(c) = self.peek() {
            if *c == '"' {
                self.advance(); // Consume the closing quote
                return TokenType::String(value);
            }

            if *c == '\\' {
                self.advance(); // Consume the backslash
                if let Some(escaped_char) = self.advance() {
                    match escaped_char {
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        _ => {
                            value.push('\\');
                            value.push(escaped_char);
                        }
                    }
                } else {
                    return TokenType::Unknown; // Unclosed escape sequence
                }
            } else {
                value.push(self.advance().unwrap());
            }
        }
        TokenType::Unknown // Unclosed string
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tokens(input: &str, expected: Vec<TokenType>) {
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
        assert_tokens("", vec![TokenType::Eof]);
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
        assert_tokens(input, expected);
    }

    #[test]
    fn test_multi_char_operators() {
        let input = ":: ...";
        let expected = vec![TokenType::DoubleColon, TokenType::Spread, TokenType::Eof];
        assert_tokens(input, expected);
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
        assert_tokens(input, expected);
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
        assert_tokens(input, expected);
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
        assert_tokens(input, expected);
    }

    #[test]
    fn test_strings() {
        let input = r#"hello world" "" "another"#;
        let expected = vec![
            TokenType::String("hello world".to_string()),
            TokenType::String("".to_string()),
            TokenType::String("another".to_string()),
            TokenType::Eof,
        ];
        assert_tokens(input, expected);
    }

    #[test]
    fn test_strings_with_escapes() {
        let input = r#"hello \"world\"	\n\r"#;
        let expected = vec![
            TokenType::String(
                r#"hello "world"	
"#
                .to_string(),
            ),
            TokenType::Eof,
        ];
        assert_tokens(input, expected);
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
        print!("{}", input);
        assert_tokens(input, expected);
    }
}
