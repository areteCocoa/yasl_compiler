/*
 * Token.rs
 *
 * Thomas Ring
 * August 30, 2016
 * token.rs
 *
 */

use std::fmt;

#[derive(Clone, PartialEq)]
pub enum TokenType {
    // Identifier
    Identifier,

    // Keyword
    Keyword(KeywordType),

    // Numbers
    Number,

    // String
    String,

    // Punctuation
    Semicolon,
    Colon,
    Period,
    Comma,
    LeftParen,
    RightParen,

    // Operators
    Plus,
    Minus,
    Star,
    Equals,

    // Misc
    EOFile,

    // Invalids
    Invalid
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &TokenType::Identifier => write!(f, "ID"),
            &TokenType::Keyword(ref k) => {
                write!(f, "{}", k)
            },
            &TokenType::Number => write!(f, "NUM"),
            &TokenType::String => {
                write!(f, "STRING")
            },

            &TokenType::Semicolon => write!(f, "SEMI"),
            &TokenType::Colon => write!(f, "COLON"),
            &TokenType::Period => write!(f, "PERIOD"),
            &TokenType::Comma => write!(f, "COMMA"),
            &TokenType::LeftParen => write!(f, "LPAREN"),
            &TokenType::RightParen => write!(f, "RPAREN"),

            &TokenType::Plus => write!(f, "PLUS"),
            &TokenType::Minus => write!(f, "MINUS"),
            &TokenType::Star => write!(f, "STAR"),
            &TokenType::Equals => write!(f, "EQUALS"),
            &TokenType::EOFile => write!(f, "EOF"),
            &TokenType::Invalid => write!(f, "Invalid"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum KeywordType {
    Program,
    Const,
    Begin,
    Print,
    End,
    Div,
    Mod,
    Var,
    Int,
    Bool,
    Proc,
    If,
    Then,
    Else,
    While,
    Do,
    Prompt,
    And,
    Or,
    Not,
    True,
    False,
}

impl fmt::Display for KeywordType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::KeywordType::*;
        match *self {
            Program => write!(f, "PROGRAM"),
            Const => write!(f, "CONST"),
            Begin => write!(f, "BEGIN"),
            Print => write!(f, "PRINT"),
            End => write!(f, "END"),
            Div => write!(f, "DIV"),
            Mod => write!(f, "MOD"),
            Var => write!(f, "VAR"),
            Int => write!(f, "INT"),
            Bool => write!(f, "BOOL"),
            Proc => write!(f, "PROC"),
            If => write!(f, "IF"),
            Then => write!(f, "THEN"),
            Else => write!(f, "ELSE"),
            While => write!(f, "WHILE"),
            Do => write!(f, "DO"),
            Prompt => write!(f, "PROMPT"),
            And => write!(f, "AND"),
            Or => write!(f, "OR"),
            Not => write!(f, "NOT"),
            True => write!(f, "TRUE"),
            False => write!(f, "FALSE"),
        }
    }
}

#[derive(Clone)]
pub struct Token {
    token_type: TokenType,
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

    pub fn token_type(&self) -> TokenType {
        self.token_type.clone()
    }

    pub fn set_token_type(&mut self, t: TokenType) {
        self.token_type = t;
    }

    pub fn is_type(&self, t: TokenType) -> bool {
        self.token_type == t
    }

    pub fn lexeme(&self) -> String {
        self.lexeme.clone()
    }

    pub fn set_lexeme(&mut self, l: String) {
        self.lexeme = l;
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn set_line(&mut self, line: u32) {
        self.line = line;
    }

    pub fn column(&self) -> u32 {
        self.column
    }

    pub fn set_column(&mut self, column: u32) {
        self.column = column;
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}:{}", self.token_type, self.lexeme, self.line, self.column)
    }
}

pub struct TokenBuilder {
    token_state: TokenState,

    line: u32,
    column: u32,
    lexeme: String,
}

impl TokenBuilder {
    pub fn new(column: u32, line: u32) -> TokenBuilder {
        TokenBuilder {
            line: line,
            column: column,

            lexeme: String::new(),

            token_state: TokenState::Start,
        }
    }

    pub fn is_start(&self) -> bool {
        match self.token_state {
            TokenState::Start => true,
            _ => false,
        }
    }

    // Takes a character and pushes it to the lexeme and advances the state,
    // returns true if it reaches a final (accepting or invalid) state
    pub fn push_char(&mut self, c: char) -> (Option<Token>, bool) {
        // Advance the state based on the character
        self.token_state = self.token_state.next_state(c);
        let mut pushback = false;

        match self.token_state {
            TokenState::Start => self.lexeme = String::new(),

            _ => {self.lexeme.push(c)}
        }

        // Check if the state is now at accepted
        // Cases in this block are also responsible for pushing the character
        // onto the lexeme because of how accepting states sometimes require
        // the character not be pushed.
        let token = match self.token_state {
            TokenState::Accept(action, _) => {
                let final_lexeme = match action {
                    TokenAction::AcceptPushback => {
                        pushback = true;
                        self.lexeme.pop();
                        self.lexeme.clone()
                    }
                    _ => {
                        self.lexeme.clone()
                    }
                };


                let result = match action {
                    TokenAction::Ignore => None,
                    _ => {
                        Some(Token {
                           token_type: self.final_type(),
                           line: self.line,
                           column: self.column,
                           // Check if the action requires that we push back
                           lexeme: final_lexeme
                       })
                    }
                };

                result
            },

            TokenState::Unaccepted => {
                let lexeme = self.lexeme.clone();

                println!("<YASLC/Lexer> Warning: Invalid token found at ({}, {}) with lexeme \"{}\".",
                    self.line, self.column, lexeme);

                Some(Token {
                    token_type: TokenType::Invalid,
                    line: self.line,
                    column: self.column,
                    lexeme: lexeme,
                })
            }

            // Not an accepting case, we have to push the
            _ => None,
        };



        // If we created a token we have to reset ourselves and increment values
        (token, pushback)
    }

    fn final_type(&self) -> TokenType {
        match self.token_state.clone() {
            TokenState::Accept(_, t) => {
                match t {
                    TokenType::Identifier => {
                        let l = self.lexeme.to_lowercase();
                        match self.keyword_for_token(&l) {
                            Some(s) => TokenType::Keyword(s),
                            None => TokenType::Identifier,
                        }
                    },
                    _ => t
                }
            },
            _ => TokenType::Invalid
        }
    }

    fn keyword_for_token(&self, lexeme: &str) -> Option<KeywordType> {
        use self::KeywordType::*;
        match lexeme {
            "program" => Some(Program),
            "const" => Some(Const),
            "begin" => Some(Begin),
            "print" => Some(Print),
            "end" => Some(End),
            "div" => Some(Div),
            "mod" => Some(Mod),
            "var" => Some(Var),
            "int" => Some(Int),
            "bool" => Some(Bool),
            "proc" => Some(Proc),
            "if" => Some(If),
            "then" => Some(Then),
            "else" => Some(Else),
            "while" => Some(While),
            "do" => Some(Do),
            "prompt" => Some(Prompt),
            "and" => Some(And),
            "or" => Some(Or),
            "not" => Some(Not),
            "true" => Some(True),
            "false" => Some(False),
            _ => None,
        }
    }

    // Consumption setter functions
    pub fn line(&mut self, line: u32) {
        self.line = line;
    }

    pub fn column(&mut self, column: u32) {
        self.column = column;
    }

    pub fn lexeme(&mut self, lexeme: String) {
        self.lexeme = lexeme;
    }
}

/*
 * DFA code
 */
 #[derive(Clone)]
enum TokenState {
    Start, // 0

    Identifier, // 1

    Number, // 2

    String, // 3

    CommentCurly, // 5
    CommentSlashStart, // 6
    CommentSlash, // 7

    Accept(TokenAction, TokenType),
    Unaccepted,
}

#[derive(Copy, Clone)]
enum TokenAction {
    Accept,
    AcceptPushback,
    Ignore,
}

impl TokenState {
    fn next_state(&self, input: char) -> TokenState {
        match *self {
            // Starting state
            TokenState::Start => {
                // Check for ignored characters first
                if input == '\r' || input == '\n' || input == ' ' {
                    TokenState::Start
                } else if input.is_alphabetic() {
                    TokenState::Identifier
                } else if let Some(input_digit) = input.to_digit(10) {
                    if input_digit == 0 {
                        TokenState::Accept(TokenAction::Accept, TokenType::Number)
                    } else {
                        TokenState::Number
                    }
                } else if input == '"' {
                    TokenState::String
                } else if input == '.' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Period)
                } else if input == ';' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Semicolon)
                } else if input == ',' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Comma)
                } else if input == '+' || input == '-' || input == '*' || input == '=' {
                    TokenState::Accept(TokenAction::Accept, match input {
                        '+' => TokenType::Plus,
                        '-' => TokenType::Minus,
                        '*' => TokenType::Star,
                        '=' => TokenType::Equals,
                        _ => TokenType::Invalid
                    })
                } else if input == ':' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Colon)
                }else if input == '/' {
                    TokenState::CommentSlashStart
                } else if input == '{' {
                    TokenState::CommentCurly
                }
                else {
                    let i = input as u8;
                    println!("<YASLC/Lexer> Internal warning: unrecognized character with ASCII value '{}' found.", i);
                    TokenState::Unaccepted
                }
            },

            TokenState::Identifier => {
                if input.is_alphabetic(){
                    TokenState::Identifier
                } else if let Some(_) = input.to_digit(10) {
                    TokenState::Identifier
                } else {
                    TokenState::Accept(TokenAction::AcceptPushback, TokenType::Identifier)
                }
            }

            TokenState::Number => {
                if let Some(_) = input.to_digit(10) {
                    TokenState::Number
                } else {
                    TokenState::Accept(TokenAction::AcceptPushback, TokenType::Number)
                }
            }

            TokenState::String => {
                if input == '"' {
                    TokenState::Accept(TokenAction::Accept, TokenType::String)
                } else {
                    TokenState::String
                }
            },

            TokenState::CommentCurly => {
                if input == '}' {
                    TokenState::Start
                } else {
                    TokenState::CommentCurly
                }
            },

            TokenState::CommentSlashStart => {
                if input == '/' {
                    TokenState::CommentSlash
                } else {
                    TokenState::Unaccepted
                }
            }

            TokenState::CommentSlash => {
                if input == '\n' {
                    TokenState::Start
                } else {
                    TokenState::CommentSlash
                }
            }

            _ => {
                TokenState::Unaccepted
            }

        }
    }
}
