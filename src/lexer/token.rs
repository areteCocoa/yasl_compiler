/// lexer/token.rs
///
/// The token module contains all code for tokens storage as well as functionality with
/// the TokenBuilder to have a state machine which can create tokens based on input.

use std::fmt;

/// TokenType represents all the different types of tokens that can be used in YASL.
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
    Assign,

    // Comparators
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    EqualTo,
    NotEqualTo,

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
            &TokenType::Assign => write!(f, "EQUALS"),

            &TokenType::GreaterThan => write!(f, "GREATERTHAN"),
            &TokenType::LessThan => write!(f, "LESSTHAN"),
            &TokenType::GreaterThanOrEqual => write!(f, "GREATERTHANOREQUAL"),
            &TokenType::LessThanOrEqual => write!(f, "LESSTHANOREQUAL"),
            &TokenType::EqualTo => write!(f, "EQUALTO"),
            &TokenType::NotEqualTo => write!(f, "NOTEQUALTO="),

            &TokenType::EOFile => write!(f, "EOF"),
            &TokenType::Invalid => write!(f, "Invalid"),
        }
    }
}

/// KeywordType is an enum subset of TokenType used to store all the types of keywords.
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

/// Token is used to store information about a single token.
#[derive(Clone, PartialEq)]
pub struct Token {
    /// The type of token.
    token_type: TokenType,

    /// The line where the token starts.
    line: u32,

    /// The column where the token starts.
    column: u32,

    /// The lexeme associated with this token.
    lexeme: String
}

impl Token {
    /// Returns an empty, placeholder token.
    pub fn new() -> Token {
        Token {
            // Just testing, remove this before doing TODO
            line: 0,
            column: 0,
            lexeme: "".to_string(),
            token_type: TokenType::EOFile,
        }
    }

    /// Returns the token_type for this token.
    pub fn token_type(&self) -> TokenType {
        self.token_type.clone()
    }

    /// Sets the token_type for this token.
    pub fn set_token_type(&mut self, t: TokenType) {
        self.token_type = t;
    }

    /// Returns true if the token is of type t, false otherwise
    pub fn is_type(&self, t: TokenType) -> bool {
        self.token_type == t
    }

    /// Returns the lexeme associated with this token.
    pub fn lexeme(&self) -> String {
        self.lexeme.clone()
    }

    /// Sets the lexeme associated with this token.
    pub fn set_lexeme(&mut self, l: String) {
        self.lexeme = l;
    }

    /// Returns the line number for this token.
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Sets the line number for this token.
    pub fn set_line(&mut self, line: u32) {
        self.line = line;
    }

    /// Returns the column number for this token.
    pub fn column(&self) -> u32 {
        self.column
    }

    /// Sets the column number for this token.
    pub fn set_column(&mut self, column: u32) {
        self.column = column;
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}:{}", self.token_type, self.lexeme, self.line, self.column)
    }
}

/// TokenBuilder uses TokenState to push characters and returns tokens when they are
/// appropriately generated.
pub struct TokenBuilder {
    /// The current token builder state.
    token_state: TokenState,

    /// The starting line number for the current token.
    line: u32,

    /// The starting column number for the current token.
    column: u32,

    /// The current lexeme for the token.
    ///
    /// This is pushed onto as characters are input.
    lexeme: String,
}

impl TokenBuilder {
    /// Returns a new TokenBuilder given the line and column, initializing the state at Start.
    pub fn new(column: u32, line: u32) -> TokenBuilder {
        TokenBuilder {
            line: line,
            column: column,

            lexeme: String::new(),

            token_state: TokenState::Start,
        }
    }

    /// Returns true if the TokenBuilder is at the start state, false otherwise.
    pub fn is_start(&self) -> bool {
        match self.token_state {
            TokenState::Start => true,
            _ => false,
        }
    }

    /// Takes a character and pushes it to the lexeme and advances the state,
    /// returns true if it reaches a final (accepting or invalid) state as well as
    /// Some(t) where t is the generated token.
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

    /// Returns the final type for tokens, useful for keywords that can not be identified until
    /// they are completely finished.
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

    /// Returns the KeywordType given the input lexeme and returns Some(k) where k
    /// is the final state if it is a keyword and None otherwise.
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
            "boolean" => Some(Bool),
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

    /// Returns the line where the current token started.
    pub fn line(&mut self, line: u32) {
        self.line = line;
    }

    /// Returns the column where the current column started.
    pub fn column(&mut self, column: u32) {
        self.column = column;
    }

    /// Returns the current lexeme for this token builder.
    #[allow(dead_code)]
    pub fn lexeme(&mut self, lexeme: String) {
        self.lexeme = lexeme;
    }
}

/*
 * DFA code
 */

 /// TokenState represents a single state in the DFA.
 ///
 /// TokenState also contains functionality to advance along the DFA while consuming itself.
 #[derive(Clone)]
enum TokenState {
    Start, // 0

    Identifier, // 1

    Number, // 2

    String, // 3

    CommentCurly, // 5
    CommentSlashStart, // 6
    CommentSlash, // 7

    GTStart,
    LTStart,
    EqualStart,

    Accept(TokenAction, TokenType),
    Unaccepted,
}

/// Accepting actions for tokens, whether they should be accepted or if they should push the
/// cursor back one character and accept the token. Useful when a token can not end itself.
#[derive(Copy, Clone)]
enum TokenAction {
    Accept,
    AcceptPushback,
}

impl TokenState {
    /// Returns the next state given the current state and the input character.
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
                } else if input == '+' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Plus)
                } else if input == '-' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Minus)
                } else if input == '*' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Star)
                } else if input == '>' {
                    TokenState::GTStart
                } else if input == '<' {
                    TokenState::LTStart
                } else if input == '=' {
                    TokenState::EqualStart
                } else if input == ':' {
                    TokenState::Accept(TokenAction::Accept, TokenType::Colon)
                }else if input == '/' {
                    TokenState::CommentSlashStart
                } else if input == '{' {
                    TokenState::CommentCurly
                } else if input == '(' {
                    TokenState::Accept(TokenAction::Accept, TokenType::LeftParen)
                } else if input == ')' {
                    TokenState::Accept(TokenAction::Accept, TokenType::RightParen)
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

            TokenState::GTStart => {
                if input == '=' {
                    return TokenState::Accept(TokenAction::Accept, TokenType::GreaterThanOrEqual);
                } else {
                    return TokenState::Accept(TokenAction::AcceptPushback, TokenType::GreaterThan);
                }
            }

            TokenState::LTStart => {
                if input == '=' {
                    return TokenState::Accept(TokenAction::Accept, TokenType::LessThanOrEqual);
                } else if input == '>' {
                    return TokenState::Accept(TokenAction::Accept, TokenType::NotEqualTo);
                } else {
                    return TokenState::Accept(TokenAction::AcceptPushback, TokenType::LessThan);
                }
            }

            TokenState::EqualStart => {
                if input == '=' {
                    return TokenState::Accept(TokenAction::Accept, TokenType::EqualTo);
                } else {
                    return TokenState::Accept(TokenAction::AcceptPushback, TokenType::Assign);
                }
            }

            _ => {
                TokenState::Unaccepted
            }

        }
    }
}
