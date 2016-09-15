
/*
 * Token.rs
 *
 * Thomas Ring
 * August 30, 2016
 * token.rs
 *
 */

use std::fmt;

#[derive(Copy, Clone)]
pub enum TokenType {
    // Identifier
    Identifier,

    // Keyword
    Keyword,

    // Numbers
    Number,

    // String
    String,

    // Punctuation
    Semicolon,
    Plus,
    Minus,
    Star,

    // Misc
    EOFile,

    // Invalids
    Invalid
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenType::Identifier => write!(f, "ID"),
            TokenType::Keyword => {
                write!(f, "KEYWORD")
            },
            TokenType::Number => write!(f, "NUM"),
            TokenType::String => {
                write!(f, "STRING")
            },
            TokenType::Semicolon => write!(f, "SEMI"),
            TokenType::Plus => write!(f, "PLUS"),
            TokenType::Minus => write!(f, "MINUS"),
            TokenType::Star => write!(f, "STAR"),
            TokenType::EOFile => write!(f, "EOF"),
            TokenType::Invalid => write!(f, "Invalid"),
        }
    }
}

pub struct Token {
    pub token_type: TokenType,
    line: u32,
    column: u32,
    lexeme: String
}

impl Token {
    pub fn new() -> Token {
        Token {
            // Just testing, remove this before doing TODO
            line: 0,
            column: 0,
            lexeme: "".to_string(),
            token_type: TokenType::EOFile,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}:{}", self.token_type, self.lexeme, self.line, self.column)
    }
}

pub struct TokenBuilder {
    token_type: TokenType,
    line: u32,
    column: u32,
    lexeme: String,
}

impl TokenBuilder {
    pub fn new() -> TokenBuilder {
        TokenBuilder {
            token_type: TokenType::Invalid,
            line: 0,
            column: 0,
            lexeme: "".to_string(),
        }
    }

    pub fn token(self) -> Token {
        Token {
            token_type: self.token_type,
            line: self.line,
            column: self.column,
            lexeme: self.lexeme,
        }
    }

    pub fn token_type(mut self, t: TokenType) -> TokenBuilder {
        self.token_type = t;
        self
    }

    pub fn line(mut self, line: u32) -> TokenBuilder {
        self.line = line;
        self
    }

    pub fn column(mut self, column: u32) -> TokenBuilder {
        self.column = column;
        self
    }

    pub fn lexeme(mut self, lexeme: String) -> TokenBuilder {
        self.lexeme = lexeme;
        self
    }
}
